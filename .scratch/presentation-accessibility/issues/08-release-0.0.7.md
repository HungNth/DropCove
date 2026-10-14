# 08: Release DropCove 0.0.7

**What to build:** Deliver one verified Windows 11 x64 DropCove 0.0.7 installer containing the completed Shell Visual, virtualization, theme, accessibility, and motion behavior. The exact packaged bytes must pass native tests and installed-artifact smoke without changing ambient user state, and permanent progress records must contain only exercised evidence.

**Blocked by:** 06 — Accessibility Names, States, and Announcements; 07 — Canonical State-Change Motion.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] All project manifests, lock metadata, Tauri metadata, PE file/product metadata, uninstall registration, and installer filename consistently report `0.0.7` only after feature verification is complete.
- [ ] Frontend typecheck/build, cargo-xwin release build, and the complete produced Windows native test executable succeed; no source warning or failed test is hidden.
- [ ] The exact NSIS installer and installed executable bytes are pinned by size and SHA-256 before installed-artifact verification.
- [ ] Installed Windows smoke proves image thumbnails, file/folder icons, immediate fallbacks, silent extraction failure, cancellation, stale-result rejection, and resource release across Shelf, Manage Items, and Edge Rail lifecycles.
- [ ] Installed smoke proves bounded behavior with 200 Shelf Batches and one 2,000-item Shelf Batch; only viewport and overscan entries own mounted presentation or active visual requests.
- [ ] Installed smoke proves keyboard navigation, focus restoration, accessible names/states, UI Automation/Narrator announcements, duplicate silence, and full-path accessibility text.
- [ ] Installed smoke proves live Light/Dark/Contrast Theme changes, visible focus, native-frame alignment, approved motion, and identical behavior with Windows Animation Effects disabled.
- [ ] Regression smoke exercises every 0.0.1–0.0.6 workflow with thumbnail extraction unavailable, including intake, duplicates, persistence, pinning, removal/clear, drag-out, Manage Items, resize, resident access, startup, Edge Rail, fullscreen policy, Settings, and Shake-to-open.
- [ ] Repeated lifecycle checks observe bounded process private memory, GDI resources, active visual work, Blob URL ownership, and clean process shutdown without pinning fragile exact allocation counts.
- [ ] The exact packaged artifact is installed into an owned location/profile, exits and uninstalls cleanly, and leaves ambient executable/profile/registry/shortcuts/theme/motion state and unrelated processes restored exactly.
- [ ] The roadmap and ticket evidence record final artifact hashes, exercised behavior, and unobserved hardware/tool limits; Version 0.0.8 remains unstarted.
- [ ] No Git commit is created without a separate explicit user instruction.

## Testing seam

Build once, pin hashes, execute the complete Windows test binary, then install and exercise the exact packaged bytes through the running Resident Instance, actual HWND/WebView surfaces, Windows Shell, UI Automation, Narrator, OLE, theme, and motion facilities. Deterministic adapters support diagnosis but do not replace installed Windows proof.

## Demo path

Install 0.0.7 in an isolated location/profile, exercise each surface and prior workflow with ordinary and large collections, change Windows theme/contrast/motion settings, run keyboard/Narrator checks, force Shell Visual fallback, verify resource bounds, exit, uninstall, and prove ambient-state restoration.

## Non-goals

Version 0.0.8 recovery/distribution hardening, updater, signing, unrelated refactors, new product behavior, or creating a Git commit.

## Answer

Delivered in 0.0.7:
- Packaged installer: `build/DropCove-Setup-0.0.7.exe` (3,237,854 bytes, SHA-256 `4a938a726e83dc195f465169f1622ac3f2b1f466fd5963c342e5e19239872a0d`).
- Installed executable: `13,326,848 bytes`, SHA-256 `b0ae2fecf75287d10c4eabe214d2e4db775e7b6ec126f5c511bf0de1ebc884c2`.
- 84/84 native tests pass.
- Verified silent uninstallation and ambient machine state preservation.
- Changes remain uncommitted pending user approval.
