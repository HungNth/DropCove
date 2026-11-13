import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  packageInstallerZip,
  verifyManifestVersions,
  verifyReleaseTag,
  verifyZipContents,
} from "../scripts/release-check.mjs";

function makeTempRoot(version = "0.1.0") {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "dropcove-rel-"));
  fs.mkdirSync(path.join(root, "src-tauri"), { recursive: true });
  fs.mkdirSync(path.join(root, "build"), { recursive: true });
  fs.writeFileSync(path.join(root, "VERSION"), `${version}\n`);
  fs.writeFileSync(path.join(root, "package.json"), JSON.stringify({ name: "dropcove", version }, null, 2));
  fs.writeFileSync(
    path.join(root, "src-tauri", "Cargo.toml"),
    `[package]\nname = "dropcove"\nversion = "${version}"\n`,
  );
  fs.writeFileSync(
    path.join(root, "src-tauri", "Cargo.lock"),
    `version = 4\n\n[[package]]\nname = "dropcove"\nversion = "${version}"\n`,
  );
  fs.writeFileSync(
    path.join(root, "src-tauri", "tauri.conf.json"),
    JSON.stringify({ productName: "DropCove", version }, null, 2),
  );
  return root;
}

test("accepts synthetic v<VERSION> tag matching canonical VERSION and synchronized manifests", () => {
  const root = makeTempRoot("0.1.0");
  try {
    const version = verifyManifestVersions(root);
    assert.equal(version, "0.1.0");
    assert.deepEqual(verifyReleaseTag("v0.1.0", version), {
      tag: "v0.1.0",
      version: "0.1.0",
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("rejects malformed tags and version mismatches", () => {
  assert.throws(() => verifyReleaseTag("0.1.0", "0.1.0"));
  assert.throws(() => verifyReleaseTag("v0.1", "0.1.0"));
  assert.throws(() => verifyReleaseTag("v0.1.0-rc.1", "0.1.0"));
  assert.throws(() => verifyReleaseTag("v0.2.0", "0.1.0"));

  const root = makeTempRoot("0.1.0");
  try {
    fs.writeFileSync(
      path.join(root, "src-tauri", "Cargo.lock"),
      `version = 4\n\n[[package]]\nname = "dropcove"\nversion = "0.0.28"\n`,
    );
    assert.throws(() => verifyManifestVersions(root), /Cargo\.lock/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("packages ZIP containing only DropCove-Setup-<version>.exe at archive root and rejects extra entries", () => {
  const root = makeTempRoot("0.1.0");
  try {
    const exePath = path.join(root, "build", "DropCove-Setup-0.1.0.exe");
    const zipPath = path.join(root, "build", "DropCove-Setup-0.1.0.zip");
    fs.writeFileSync(exePath, "MZ-dummy-installer-bytes");

    packageInstallerZip({ rootDir: root, version: "0.1.0" });
    assert.ok(fs.existsSync(zipPath));
    assert.deepEqual(verifyZipContents({ zipPath, version: "0.1.0" }), [
      "DropCove-Setup-0.1.0.exe",
    ]);

    const extraFile = path.join(root, "build", "extra.txt");
    fs.writeFileSync(extraFile, "unwanted");
    const badZipPath = path.join(root, "build", "bad.zip");
    execFileSync("powershell.exe", [
      "-NoProfile",
      "-Command",
      `Compress-Archive -Path '${exePath}', '${extraFile}' -DestinationPath '${badZipPath}' -Force`,
    ]);
    assert.throws(() => verifyZipContents({ zipPath: badZipPath, version: "0.1.0" }));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
