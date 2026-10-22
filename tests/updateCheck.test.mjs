import assert from "node:assert/strict";
import test from "node:test";
import { checkForUpdates } from "../src/updateCheck.ts";

const REPO = "https://github.com/HungNth/DropCove";
const PAGE = `${REPO}/releases/tag/`;

function release(tag, extra = {}) {
  return { tag_name: tag, draft: false, prerelease: false, html_url: `${PAGE}${tag}`, ...extra };
}

function respond(status, body) {
  return async () => ({ ok: status >= 200 && status < 300, status, json: async () => body });
}

const check = (version, fetchImpl, repo = REPO) => checkForUpdates(version, repo, fetchImpl);

test("an empty release list has no release", async () => {
  assert.deepEqual(await check("0.0.20", respond(200, [])), { kind: "no-release" });
});

test("drafts, pre-releases and non-semver tags never qualify", async () => {
  const body = [
    release("v9.9.9", { draft: true }),
    release("v8.8.8", { prerelease: true }),
    release("nightly"),
    release("v1.2"),
    release("v1.2.3-beta"),
  ];
  assert.deepEqual(await check("0.0.20", respond(200, body)), { kind: "no-release" });
});

test("a higher release is available with its normalized version and page", async () => {
  const result = await check("0.0.20", respond(200, [release("v0.0.21")]));
  assert.deepEqual(result, { kind: "available", version: "0.0.21", url: `${PAGE}v0.0.21` });
});

test("the highest version wins regardless of order, comparing numerically", async () => {
  const body = [release("0.0.9"), release("v0.0.10"), release("0.0.2")];
  const result = await check("0.0.1", respond(200, body));
  assert.deepEqual(result, { kind: "available", version: "0.0.10", url: `${PAGE}v0.0.10` });
});

test("an equal or newer running version is up to date", async () => {
  assert.deepEqual(await check("0.0.20", respond(200, [release("v0.0.20")])), { kind: "up-to-date" });
  assert.deepEqual(await check("0.1.0", respond(200, [release("v0.0.20")])), { kind: "up-to-date" });
});

test("a release page outside the repository releases path is invalid data", async () => {
  const body = [release("v0.0.21", { html_url: "https://evil.example/releases/tag/v0.0.21" })];
  assert.deepEqual(await check("0.0.20", respond(200, body)), { kind: "failed" });
  const lookalike = [release("v0.0.21", { html_url: "https://github.com/HungNth/DropCove-evil/releases/tag/v0.0.21" })];
  assert.deepEqual(await check("0.0.20", respond(200, lookalike)), { kind: "failed" });
});

test("HTTP 403 and 429 are rate limits", async () => {
  assert.deepEqual(await check("0.0.20", respond(403, {})), { kind: "rate-limited" });
  assert.deepEqual(await check("0.0.20", respond(429, {})), { kind: "rate-limited" });
});

test("other statuses, rejections, and malformed bodies fail", async () => {
  assert.deepEqual(await check("0.0.20", respond(500, [])), { kind: "failed" });
  assert.deepEqual(await check("0.0.20", async () => { throw new TypeError("offline"); }), { kind: "failed" });
  assert.deepEqual(await check("0.0.20", respond(200, { message: "nope" })), { kind: "failed" });
  assert.deepEqual(await check("0.0.20", respond(200, [{ tag_name: "v0.0.21" }])), { kind: "failed" });
  assert.deepEqual(await check("0.0.20", async () => ({ ok: true, status: 200, json: async () => { throw new SyntaxError("bad"); } })), { kind: "failed" });
});

test("a non-GitHub repository URL fails without a request", async () => {
  let called = false;
  const spy = async () => { called = true; return { ok: true, status: 200, json: async () => [] }; };
  assert.deepEqual(await check("0.0.20", spy, "https://example.com/HungNth/DropCove"), { kind: "failed" });
  assert.equal(called, false);
});

test("the request targets the releases API with the GitHub Accept header", async () => {
  let seen;
  await check("0.0.20", async (url, init) => { seen = { url, init }; return { ok: true, status: 200, json: async () => [] }; });
  assert.equal(seen.url, "https://api.github.com/repos/HungNth/DropCove/releases");
  assert.equal(seen.init.headers.Accept, "application/vnd.github+json");
});

test("a request with no response aborts after the timeout and fails", async () => {
  const hang = (_url, init) => new Promise((_resolve, reject) => {
    init.signal.addEventListener("abort", () => reject(init.signal.reason));
  });
  const started = Date.now();
  assert.deepEqual(await checkForUpdates("0.0.20", REPO, hang, 30), { kind: "failed" });
  assert.ok(Date.now() - started < 2000);
});
