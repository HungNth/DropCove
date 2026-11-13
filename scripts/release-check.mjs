import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
const SEMVER_PATTERN = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const TAG_PATTERN = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

export function verifyManifestVersions(rootDir = process.cwd()) {
  const version = fs.readFileSync(path.join(rootDir, "VERSION"), "utf8").trim();
  if (!SEMVER_PATTERN.test(version)) {
    throw new Error(`VERSION must match MAJOR.MINOR.PATCH, got: ${version}`);
  }

  const pkg = JSON.parse(fs.readFileSync(path.join(rootDir, "package.json"), "utf8"));
  if (pkg.version !== version) {
    throw new Error(`package.json version (${pkg.version}) does not match VERSION (${version})`);
  }

  const tauriConf = JSON.parse(
    fs.readFileSync(path.join(rootDir, "src-tauri", "tauri.conf.json"), "utf8"),
  );
  if (tauriConf.version !== version) {
    throw new Error(
      `src-tauri/tauri.conf.json version (${tauriConf.version}) does not match VERSION (${version})`,
    );
  }

  const cargoToml = fs.readFileSync(path.join(rootDir, "src-tauri", "Cargo.toml"), "utf8");
  const cargoMatch = /^version\s*=\s*"([^"]+)"/m.exec(cargoToml);
  if (!cargoMatch || cargoMatch[1] !== version) {
    throw new Error(
      `src-tauri/Cargo.toml version (${cargoMatch?.[1] ?? "missing"}) does not match VERSION (${version})`,
    );
  }

  const cargoLock = fs.readFileSync(path.join(rootDir, "src-tauri", "Cargo.lock"), "utf8");
  const lockMatch = /\[\[package\]\]\s*\r?\nname = "dropcove"\s*\r?\nversion = "([^"]+)"/.exec(
    cargoLock,
  );
  if (!lockMatch || lockMatch[1] !== version) {
    throw new Error(
      `src-tauri/Cargo.lock dropcove version (${lockMatch?.[1] ?? "missing"}) does not match VERSION (${version})`,
    );
  }

  return version;
}

export function verifyReleaseTag(tag, expectedVersion) {
  const cleanTag = String(tag || "").trim();
  const match = TAG_PATTERN.exec(cleanTag);
  if (!match) {
    throw new Error(`Release tag must match vMAJOR.MINOR.PATCH, got: ${cleanTag}`);
  }
  const tagVersion = `${match[1]}.${match[2]}.${match[3]}`;
  if (tagVersion !== expectedVersion) {
    throw new Error(`Release tag ${cleanTag} does not match VERSION ${expectedVersion}`);
  }
  return { tag: cleanTag, version: tagVersion };
}

export function verifyInstallerExe({ rootDir = process.cwd(), version }) {
  const expectedExe = `DropCove-Setup-${version}.exe`;
  const exePath = path.join(rootDir, "build", expectedExe);
  if (!fs.existsSync(exePath)) {
    throw new Error(`Installer EXE not found: ${exePath}`);
  }
  const header = Buffer.alloc(2);
  const fd = fs.openSync(exePath, "r");
  let bytesRead = 0;
  try {
    bytesRead = fs.readSync(fd, header, 0, 2, 0);
  } finally {
    fs.closeSync(fd);
  }
  if (bytesRead < 2 || header[0] !== 0x4d || header[1] !== 0x5a) {
    throw new Error(`Installer EXE is empty or missing PE MZ header: ${exePath}`);
  }
  return exePath;
}
function parseCli(argv) {
  const [command, ...rest] = argv;
  const opts = { rootDir: process.cwd(), tag: "" };
  for (let i = 0; i < rest.length; i += 1) {
    const arg = rest[i];
    if (arg === "--root") opts.rootDir = path.resolve(rest[++i]);
    else if (arg === "--tag") opts.tag = rest[++i];
    else throw new Error(`Unknown argument: ${arg}`);
  }
  return { command, opts };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { command, opts } = parseCli(process.argv.slice(2));
  const version = verifyManifestVersions(opts.rootDir);

  if (command === "verify") {
    const checked = verifyReleaseTag(opts.tag || `v${version}`, version);
    console.log(`Verified DropCove version ${checked.version} and tag ${checked.tag}`);
  } else if (command === "verify-exe") {
    const exePath = verifyInstallerExe({ rootDir: opts.rootDir, version });
    console.log(`Verified installer ${exePath}`);
  } else {
    throw new Error(`Usage: node scripts/release-check.mjs <verify|verify-exe> [--tag vX.Y.Z]`);
  }
}
