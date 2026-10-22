# 02: Packaging and binary PE icon resource validation

**What to build:**
Build the Windows x64 NSIS installer candidate `DropCove-Setup-0.0.21.exe` using cargo-xwin/tauri bundler. Inspect and validate the embedded PE binary icon resources (`RT_GROUP_ICON`) of `DropCove-Setup-0.0.21.exe`, `DropCove.exe`, and the uninstaller binary `uninstall.exe`, confirming the Nullsoft default icon is completely replaced with the DropCove Brand Visual.

**Parent specification:** `.scratch/application-branding-installer-icon-unification/spec.md`

**Blocked by:** 01-icon-asset-unification-bundler-configuration

**Status:** resolved

## Acceptance criteria

- [x] Windows x64 NSIS candidate builds through cargo-xwin and is copied to the release output.
- [x] Setup embeds the Brand Visual in every entry of its icon group.
- [x] The exact installed application embeds the Brand Visual in every entry of its icon group.
- [x] The generated and installed uninstaller embeds the Brand Visual in every entry of its icon group.
- [x] The complete freshly compiled native suite passes: 124/124; frontend suite passes: 13/13.

## Testing seam

Automated PE header parser script inspecting `RT_GROUP_ICON` and `RT_ICON` entries in `DropCove-Setup-0.0.21.exe`, `DropCove.exe`, and `uninstall.exe`.

## Demo path

Run PE icon resource validator script against all 3 executables and output the icon group structures and SHA-256 hashes proving they match `icon.ico`.

## Comments

- Built with the existing Tauri cargo-xwin runner and installer-copy script. Existing SDK missing-PDB LNK4099 warnings remain nonfatal and unsuppressed.
- Parsed each `RT_GROUP_ICON` entry, followed its resource ID to `RT_ICON`, checked size/dimensions/depth, and asserted its complete payload equals the corresponding source ICO entry. All six layers match for Setup, installed application, and installed uninstaller; resource IDs differ but payloads do not.
- Exact artifact sizes and complete SHA-256 hashes are recorded in the 0.0.21 roadmap evidence.
- The initial inspection left a PE memory map open, locking an executable and invalidating an early uninstall smoke. Closing analysis handles resolved this harness issue. Subsequent standard uninstall removed the complete owned installation without manual deletion.
