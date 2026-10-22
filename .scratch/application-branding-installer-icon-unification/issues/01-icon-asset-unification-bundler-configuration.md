# 01: Icon asset unification and bundler configuration

**What to build:**
Generate unified multi-resolution Windows icon assets (`icon.ico` containing 16x16, 24x24, 32x32, 48x48, 64x64, 256x256 layers and bundle PNG assets) directly from the Brand Visual vector source (`app-icon.svg`), discarding non-Windows artifacts. Explicitly configure both `installerIcon` and `uninstallerIcon` in `tauri.conf.json` under `bundle.windows.nsi`. Synchronize version `0.0.21` across `package.json`, `Cargo.toml`, and `tauri.conf.json`.

**Parent specification:** `.scratch/application-branding-installer-icon-unification/spec.md`

**Blocked by:** None (can start immediately)

**Status:** resolved

## Acceptance criteria

- [x] `src-tauri/icons/icon.ico` contains all standard Windows resolutions: 16x16, 24x24, 32x32, 48x48, 64x64, 256x256 with 32-bit RGBA color depth derived from `app-icon.svg`.
- [x] Windows/Tauri bundle PNG assets in `src-tauri/icons/` (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`) match the Brand Visual.
- [x] Non-Windows assets remain unchanged; generated Android/iOS/ICNS changes were discarded.
- [x] NSIS packaging explicitly defines both `installerIcon` and `uninstallerIcon` using the unified ICO.
- [x] VERSION, package, Rust crate/lock, and Tauri metadata are synchronized to `0.0.21`.
- [x] Frontend production build and `cargo xwin check` targeting Windows x64 pass.

## Testing seam

Automated inspection of `src-tauri/icons/icon.ico` binary headers verifying resolution entries and color depth, followed by clean frontend/Tauri compilation.

## Demo path

Run binary ICO inspection script confirming all 6 standard Windows icon dimensions and verify synchronized version `0.0.21` across manifests.

## Comments

- Regenerated assets with the pinned Tauri CLI. Windows ICO and PNG outputs were already byte-identical to the canonical vector's generated assets; no image replacement was necessary. Only the missing installer/uninstaller configuration needed changing.
- ICO inspection verified all six dimensions and 32-bit depth. Frontend typechecking/Vite build passed. Ordinary `cargo check` selected an unsuitable `link.exe`; the required cargo-xwin check passed without installing Visual Studio.
- Existing native window and tray icon loading paths are reused unchanged. No dependency or permanent test was added for configuration wiring; artifact and installed-runtime checks cover the observable result.
