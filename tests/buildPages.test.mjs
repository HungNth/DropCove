import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { buildPagesSite } from "../scripts/build-pages.mjs";

const REPO_URL = "https://github.com/HungNth/DropCove";

function makeRelease({
  tag = "v0.1.0",
  draft = false,
  prerelease = false,
  assetName = "DropCove-Setup-0.1.0.zip",
  downloadUrl = `https://github.com/HungNth/DropCove/releases/download/${tag}/${assetName}`,
} = {}) {
  return {
    tag_name: tag,
    draft,
    prerelease,
    assets: assetName ? [{ name: assetName, browser_download_url: downloadUrl }] : [],
  };
}

function withTempOutDir(fn) {
  const outDir = fs.mkdtempSync(path.join(os.tmpdir(), "dropcove-pages-"));
  try {
    return fn(outDir);
  } finally {
    fs.rmSync(outDir, { recursive: true, force: true });
  }
}

test("builds landing page with resolved release version, direct ZIP link, copy, and local assets", async () => {
  await withTempOutDir(async (outDir) => {
    const result = await buildPagesSite({
      sourceDir: path.resolve("website"),
      outDir,
      repositoryUrl: REPO_URL,
      fetchImpl: async (url) => {
        assert.equal(url, "https://api.github.com/repos/HungNth/DropCove/releases?per_page=30");
        return {
          ok: true,
          status: 200,
          json: async () => [makeRelease()],
        };
      },
    });

    assert.equal(result.version, "0.1.0");
    assert.equal(result.downloadUrl, "https://github.com/HungNth/DropCove/releases/download/v0.1.0/DropCove-Setup-0.1.0.zip");

    const html = fs.readFileSync(path.join(outDir, "index.html"), "utf8");
    assert.ok(html.includes(result.downloadUrl));
    assert.ok(html.includes("0.1.0"));
    assert.ok(html.includes("Windows 11"));
    assert.ok(html.includes("build 22000"));
    assert.ok(html.includes("Microsoft Visual C++ Redistributable"));
    assert.ok(html.includes("Download ZIP"));
    assert.ok(html.includes("Extract"));
    assert.ok(html.includes("Run the installer"));
    assert.ok(html.includes("./styles.css"));
    assert.ok(html.includes("./app-icon.svg"));
    assert.ok(!html.includes("{{"));
    assert.ok(!/<script\b/i.test(html));
    assert.ok(html.includes("hero-showcase"));
    assert.ok(html.includes('aria-labelledby="showcase-caption"'));
    assert.ok(html.includes('aria-label="DropCove Drop Shelf demonstration"'));
    assert.ok(html.includes('poster="./assets/shelf-preview.png"'));
    assert.ok(html.includes("./assets/dropcove-demo.webm"));
    assert.ok(html.includes("showcase-fallback-img"));
    assert.ok(/<video\b[^>]*\bautoplay\b[^>]*\bloop\b[^>]*\bmuted\b[^>]*\bplaysinline\b/i.test(html));

    const css = fs.readFileSync(path.join(outDir, "styles.css"), "utf8");
    assert.ok(css.includes("prefers-reduced-motion"));
    assert.ok(css.includes(".showcase-video"));
    assert.ok(css.includes(".showcase-fallback-img"));

    assert.ok(fs.existsSync(path.join(outDir, "styles.css")));
    assert.ok(fs.existsSync(path.join(outDir, "app-icon.svg")));
    assert.ok(fs.existsSync(path.join(outDir, "assets", "shelf-preview.png")));
    assert.ok(fs.existsSync(path.join(outDir, "assets", "dropcove-demo.webm")));
    assert.ok(fs.existsSync(path.join(outDir, ".nojekyll")));
  });
});

test("selects highest stable release even when drafts, prereleases, or malformed tags are present", async () => {
  await withTempOutDir(async (outDir) => {
    const releases = [
      makeRelease({ tag: "v9.9.9", draft: true, assetName: "DropCove-Setup-9.9.9.zip" }),
      makeRelease({ tag: "v8.8.8", prerelease: true, assetName: "DropCove-Setup-8.8.8.zip" }),
      makeRelease({ tag: "nightly", assetName: "DropCove-Setup-nightly.zip" }),
      makeRelease({ tag: "0.1.0", assetName: "DropCove-Setup-0.1.0.zip" }),
      makeRelease({ tag: "v0.1.0", assetName: "DropCove-Setup-0.1.0.zip" }),
      makeRelease({ tag: "v0.2.0", assetName: "DropCove-Setup-0.2.0.zip" }),
    ];

    const result = await buildPagesSite({
      sourceDir: path.resolve("website"),
      outDir,
      repositoryUrl: REPO_URL,
      fetchImpl: async () => ({ ok: true, status: 200, json: async () => releases }),
    });

    assert.equal(result.version, "0.2.0");
    assert.equal(result.downloadUrl, "https://github.com/HungNth/DropCove/releases/download/v0.2.0/DropCove-Setup-0.2.0.zip");
  });
});

test("can target an explicit release tag without guessing another release", async () => {
  await withTempOutDir(async (outDir) => {
    const result = await buildPagesSite({
      sourceDir: path.resolve("website"),
      outDir,
      repositoryUrl: REPO_URL,
      tag: "v0.1.0",
      fetchImpl: async (url) => {
        assert.equal(url, "https://api.github.com/repos/HungNth/DropCove/releases/tags/v0.1.0");
        return { ok: true, status: 200, json: async () => makeRelease() };
      },
    });

    assert.equal(result.version, "0.1.0");
  });
});

test("rejects missing ZIP asset, mismatched asset name, draft/prerelease explicit tag, malformed tag, and outside download host", async () => {
  const cases = [
    [makeRelease({ assetName: "" })],
    [makeRelease({ assetName: "DropCove-Setup-0.1.0.exe" })],
    [makeRelease({ tag: "0.1.0", assetName: "DropCove-Setup-0.1.0.zip" })],
    [makeRelease({ tag: "v0.1.0", assetName: "DropCove-Setup-0.1.0.zip", downloadUrl: "https://evil.example/DropCove-Setup-0.1.0.zip" })],
    [makeRelease({ draft: true })],
    [makeRelease({ prerelease: true })],
    [],
  ];

  for (const body of cases) {
    await withTempOutDir(async (outDir) => {
      await assert.rejects(
        buildPagesSite({
          sourceDir: path.resolve("website"),
          outDir,
          repositoryUrl: REPO_URL,
          fetchImpl: async () => ({ ok: true, status: 200, json: async () => body }),
        }),
      );
      assert.equal(fs.existsSync(path.join(outDir, "index.html")), false);
    });
  }

  await withTempOutDir(async (outDir) => {
    await assert.rejects(
      buildPagesSite({
        sourceDir: path.resolve("website"),
        outDir,
        repositoryUrl: REPO_URL,
        tag: "v0.1.0",
        fetchImpl: async () => ({
          ok: true,
          status: 200,
          json: async () => makeRelease({ prerelease: true }),
        }),
      }),
    );
  });
});
