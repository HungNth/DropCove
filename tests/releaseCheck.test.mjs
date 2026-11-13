import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  verifyInstallerExe,
  verifyManifestVersions,
  verifyReleaseTag,
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

test("verifies DropCove-Setup-<version>.exe exists with PE MZ header and rejects missing or non-PE files", () => {
  const root = makeTempRoot("0.1.0");
  try {
    assert.throws(() => verifyInstallerExe({ rootDir: root, version: "0.1.0" }), /not found/);

    const exePath = path.join(root, "build", "DropCove-Setup-0.1.0.exe");
    fs.writeFileSync(exePath, "");
    assert.throws(() => verifyInstallerExe({ rootDir: root, version: "0.1.0" }), /empty|PE|MZ/i);

    fs.writeFileSync(exePath, "PK\x03\x04not-an-exe");
    assert.throws(() => verifyInstallerExe({ rootDir: root, version: "0.1.0" }), /PE|MZ/i);

    fs.writeFileSync(exePath, "MZ-dummy-installer-bytes");
    assert.equal(verifyInstallerExe({ rootDir: root, version: "0.1.0" }), exePath);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
