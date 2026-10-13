# Release DropCove 0.0.2

Status: resolved
Type: task
Version: 0.0.2
Parent: `../spec.md`
Blocked by: 01, 02, 03

## Required behavior

Close the approved version as one installable, evidence-backed delivery:

- Exercise the integrated Drop Shelf workflow from storage-item drag-in through persistence, restore, item drag-out, whole-batch preparation, and accepted or canceled completion.
- Confirm all `0.0.2` acceptance criteria in the parent spec on Windows 11 x64 at normal integrity level.
- Synchronize version `0.0.2` across package, crate, Tauri configuration, executable metadata, and installer naming.
- Build with cargo-xwin and produce `DropCove-Setup-0.0.2.exe` through the current-user NSIS configuration.
- Install and launch the packaged artifact rather than relying only on development mode.
- Update `docs/ROADMAP.md` with the actual Implemented and Verification evidence and change `0.0.2` to `Completed` only after every release criterion passes.
- Leave `0.0.3` and later versions unimplemented.

## Acceptance criteria

1. The parent spec demo succeeds from the installed `0.0.2` artifact.
2. Project manifests and produced artifact names all report `0.0.2`.
3. Restart persistence, duplicate no-op, item completion, batch availability preparation, cancellation retention, and source-object safety are observed through the packaged app.
4. `docs/ROADMAP.md` records only exercised evidence and lists any approved limitation explicitly.
5. No `0.0.3` scope is included in source or release notes.

## Testing seam

- Run the focused permanent checks created by tickets 01–03 once after integration.
- Use the installed Windows application for the release smoke; tests alone do not satisfy this ticket.
- Retain no throwaway fixtures, scripts, or test-profile data after verification.

## Demo path

Run the complete demo from `../spec.md` using `DropCove-Setup-0.0.2.exe`, then show the updated `0.0.2` Implemented and Verification sections in `docs/ROADMAP.md`.

## Comments

- Parent specification demo path and release acceptance were exercised directly from the packaged and installed artifact `DropCove-Setup-0.0.2.exe`.
- Version `0.0.2` synchronized across `VERSION`, `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Artifact built with cargo-xwin targeting `x86_64-pc-windows-msvc` and packaged as an NSIS `currentUser` installer.
- Installed to `%LOCALAPPDATA%\DropCove\DropCove.exe` with registry display version `0.0.2`.
- Exercised behaviors in installed app:
  1. Intake of file and folder items into one ordered Shelf Batch.
  2. Duplicate batch silent no-op.
  3. Restart and persistence of order, items, and sizing.
  4. Missing cleanup before drag-out.
  5. Modal confirmation when unavailable items remain.
  6. Consumption of only the Available Temporary participant after accepted Copy, with Unavailable reference retained.
- Reran the 11-test native core suite after integration: all passed.
- Clean uninstallation verified; `DropCove.exe` removed from install location.
- Final cleanup criterion passed: the throwaway `smoke-native.ps1` was removed, the installed-demo fixtures were deleted, and the final `Test-Path` check for the owned smoke directory returned `False` after its remaining log handles were released.
- Version `0.0.3` remains unstarted.
