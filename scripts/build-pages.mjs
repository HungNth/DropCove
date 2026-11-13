import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const REPO_PATTERN = /^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git|\/)?$/;
const TAG_PATTERN = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

function parseRepo(repositoryUrl) {
  const match = REPO_PATTERN.exec(String(repositoryUrl || "").trim());
  if (!match) {
    throw new Error(`Invalid GitHub repository URL: ${repositoryUrl}`);
  }
  return { owner: match[1], repo: match[2] };
}

function parseReleaseTag(tagName) {
  const match = TAG_PATTERN.exec(String(tagName || "").trim());
  if (!match) return null;
  return {
    tag: match[0],
    version: `${match[1]}.${match[2]}.${match[3]}`,
    parts: [Number(match[1]), Number(match[2]), Number(match[3])],
  };
}

function compareSemver(a, b) {
  for (let i = 0; i < 3; i += 1) {
    if (a.parts[i] !== b.parts[i]) return a.parts[i] - b.parts[i];
  }
  return 0;
}

function validateRelease(release, owner, repo) {
  if (!release || release.draft === true || release.prerelease === true) return null;
  const parsed = parseReleaseTag(release.tag_name);
  if (!parsed) return null;

  const expectedAssetName = `DropCove-Setup-${parsed.version}.zip`;
  const assets = Array.isArray(release.assets) ? release.assets : [];
  const asset = assets.find((entry) => entry && entry.name === expectedAssetName);
  if (!asset) return null;

  const expectedUrl = `https://github.com/${owner}/${repo}/releases/download/${parsed.tag}/${expectedAssetName}`;
  if (asset.browser_download_url !== expectedUrl) return null;

  return {
    tag: parsed.tag,
    version: parsed.version,
    parts: parsed.parts,
    assetName: expectedAssetName,
    downloadUrl: expectedUrl,
  };
}

async function fetchJson(url, fetchImpl, token) {
  const headers = {
    Accept: "application/vnd.github+json",
    "User-Agent": "DropCove-Pages-Builder",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  const response = await fetchImpl(url, { headers });
  if (!response || !response.ok) {
    throw new Error(`GitHub API request failed (${response ? response.status : "no response"}): ${url}`);
  }
  return response.json();
}

export async function resolvePublishedRelease({
  repositoryUrl,
  tag = "",
  fetchImpl = globalThis.fetch,
  token = process.env.GITHUB_TOKEN || "",
}) {
  const { owner, repo } = parseRepo(repositoryUrl);
  const requestedTag = String(tag || "").trim();

  if (requestedTag) {
    if (!parseReleaseTag(requestedTag)) {
      throw new Error(`Release tag must match vMAJOR.MINOR.PATCH: ${requestedTag}`);
    }
    const release = await fetchJson(
      `https://api.github.com/repos/${owner}/${repo}/releases/tags/${encodeURIComponent(requestedTag)}`,
      fetchImpl,
      token,
    );
    const validated = validateRelease(release, owner, repo);
    if (!validated) {
      throw new Error(`Release ${requestedTag} is missing a valid DropCove-Setup-<version>.zip asset or is not stable`);
    }
    return validated;
  }

  const releases = await fetchJson(
    `https://api.github.com/repos/${owner}/${repo}/releases?per_page=30`,
    fetchImpl,
    token,
  );
  if (!Array.isArray(releases)) {
    throw new Error("Unexpected GitHub releases response shape");
  }

  let best = null;
  for (const release of releases) {
    const candidate = validateRelease(release, owner, repo);
    if (!candidate) continue;
    if (!best || compareSemver(candidate, best) > 0) {
      best = candidate;
    }
  }
  if (!best) {
    throw new Error("No stable GitHub Release with a matching DropCove-Setup-<version>.zip asset was found");
  }
  return best;
}

export async function buildPagesSite({
  sourceDir,
  outDir,
  repositoryUrl,
  tag = "",
  fetchImpl = globalThis.fetch,
  token = process.env.GITHUB_TOKEN || "",
}) {
  const resolved = await resolvePublishedRelease({ repositoryUrl, tag, fetchImpl, token });
  const { owner, repo } = parseRepo(repositoryUrl);
  const repoUrl = `https://github.com/${owner}/${repo}`;
  const licenseUrl = `${repoUrl}/blob/main/LICENSE`;
  const templatePath = path.join(sourceDir, "index.html");
  const template = fs.readFileSync(templatePath, "utf8");

  const html = template
    .replaceAll("{{VERSION}}", resolved.version)
    .replaceAll("{{TAG}}", resolved.tag)
    .replaceAll("{{ASSET_NAME}}", resolved.assetName)
    .replaceAll("{{DOWNLOAD_URL}}", resolved.downloadUrl)
    .replaceAll("{{REPO_URL}}", repoUrl)
    .replaceAll("{{LICENSE_URL}}", licenseUrl);

  if (html.includes("{{")) {
    throw new Error("Unreplaced template placeholder remains in generated index.html");
  }

  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "index.html"), html, "utf8");
  fs.writeFileSync(path.join(outDir, ".nojekyll"), "", "utf8");

  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    if (entry.name === "index.html") continue;
    fs.cpSync(path.join(sourceDir, entry.name), path.join(outDir, entry.name), { recursive: true });
  }

  return resolved;
}

function readDefaultRepoUrl() {
  const cargoPath = path.resolve(process.cwd(), "src-tauri", "Cargo.toml");
  if (fs.existsSync(cargoPath)) {
    const cargo = fs.readFileSync(cargoPath, "utf8");
    const match = /^repository\s*=\s*"([^"]+)"/m.exec(cargo);
    if (match) return match[1];
  }
  return "https://github.com/HungNth/DropCove";
}

function parseCliArgs(argv) {
  const args = {
    sourceDir: path.resolve(process.cwd(), "website"),
    outDir: path.resolve(process.cwd(), "_site"),
    repositoryUrl: readDefaultRepoUrl(),
    tag: "",
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--source") args.sourceDir = path.resolve(argv[++i]);
    else if (arg === "--out") args.outDir = path.resolve(argv[++i]);
    else if (arg === "--repo") args.repositoryUrl = argv[++i];
    else if (arg === "--tag") args.tag = argv[++i];
    else throw new Error(`Unknown argument: ${arg}`);
  }
  return args;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const options = parseCliArgs(process.argv.slice(2));
  const result = await buildPagesSite(options);
  console.log(`Built DropCove Pages site for ${result.tag} -> ${result.downloadUrl}`);
}
