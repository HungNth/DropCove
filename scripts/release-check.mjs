import { execFileSync } from "node:child_process";
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

function listZipEntries(zipPath) {
  const buf = fs.readFileSync(zipPath);
  let eocdOffset = -1;
  for (let i = buf.length - 22; i >= Math.max(0, buf.length - 65_557); i -= 1) {
    if (buf.readUInt32LE(i) === 0x06054b50) {
      eocdOffset = i;
      break;
    }
  }
  if (eocdOffset < 0) {
    throw new Error(`Invalid ZIP archive (missing central directory): ${zipPath}`);
  }

  const totalEntries = buf.readUInt16LE(eocdOffset + 10);
  const centralSize = buf.readUInt32LE(eocdOffset + 12);
  const centralOffset = buf.readUInt32LE(eocdOffset + 16);
  const entries = [];
  let cursor = centralOffset;
  const end = centralOffset + centralSize;

  while (cursor < end) {
    if (buf.readUInt32LE(cursor) !== 0x02014b50) {
      throw new Error(`Invalid ZIP central directory entry in ${zipPath}`);
    }
    const nameLen = buf.readUInt16LE(cursor + 28);
    const extraLen = buf.readUInt16LE(cursor + 30);
    const commentLen = buf.readUInt16LE(cursor + 32);
    const name = buf.toString("utf8", cursor + 46, cursor + 46 + nameLen);
    entries.push(name);
    cursor += 46 + nameLen + extraLen + commentLen;
  }

  if (entries.length !== totalEntries) {
    throw new Error(`ZIP entry count mismatch in ${zipPath}`);
  }
  return entries;
}

export function verifyZipContents({ zipPath, version }) {
  if (!fs.existsSync(zipPath)) {
    throw new Error(`Release ZIP not found: ${zipPath}`);
  }
  const expectedExe = `DropCove-Setup-${version}.exe`;
  const entries = listZipEntries(zipPath);
  if (entries.length !== 1 || entries[0] !== expectedExe) {
    throw new Error(
      `Release ZIP must contain only [${expectedExe}] at root; found [${entries.join(", ")}]`,
    );
  }
  return entries;
}

export function packageInstallerZip({ rootDir = process.cwd(), version }) {
  const expectedExe = `DropCove-Setup-${version}.exe`;
  const exePath = path.join(rootDir, "build", expectedExe);
  const zipPath = path.join(rootDir, "build", `DropCove-Setup-${version}.zip`);
  if (!fs.existsSync(exePath)) {
    throw new Error(`Installer EXE not found: ${exePath}`);
  }
  fs.rmSync(zipPath, { force: true });

  execFileSync("powershell.exe", [
    "-NoProfile",
    "-Command",
    "Compress-Archive -Path $env:DC_EXE -DestinationPath $env:DC_ZIP -Force",
  ], {
    env: { ...process.env, DC_EXE: exePath, DC_ZIP: zipPath },
    stdio: "inherit",
  });

  verifyZipContents({ zipPath, version });
  return { exePath, zipPath };
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
  } else if (command === "package-zip") {
    const { zipPath } = packageInstallerZip({ rootDir: opts.rootDir, version });
    console.log(`Verified archive ${zipPath} containing only DropCove-Setup-${version}.exe`);
  } else {
    throw new Error(`Usage: node scripts/release-check.mjs <verify|package-zip> [--tag vX.Y.Z]`);
  }
}
