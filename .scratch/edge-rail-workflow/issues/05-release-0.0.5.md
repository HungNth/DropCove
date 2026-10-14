# 05: Release DropCove 0.0.5 from the installed artifact

Status: resolved
Type: task
Version: 0.0.5
Parent: [Specification](../spec.md)
Blocked by: 04
Implementation approval: approved by user on 2026-10-13

## What to build

Deliver the approved Edge Rail workflow as a version-consistent Windows x64 current-user installer, prove the entire 0.0.5 behavior from its installed executable, restore ambient user state exactly, then stop for user review without beginning 0.0.6.

## Acceptance criteria

- [x] Tickets 01–04 are resolved with permanent coordinator/store tests and actual Windows surface/process evidence.
- [x] VERSION, package/crate/lock/Tauri metadata, PE/runtime/installed version and artifact name consistently report 0.0.5.
- [x] Build with cargo-xwin for `x86_64-pc-windows-msvc` and produce `DropCove-Setup-0.0.5.exe` as current-user NSIS without Visual Studio or commercial MSVC Build Tools.
- [x] Execute the newly compiled permanent suite directly on Windows; compilation or an older test executable is not a pass.
- [x] Install and launch the packaged executable with an owned profile/WebView location and embedded frontend.
- [x] Exercise `Visible → Hidden` for empty dismissal, `Visible → EdgeDocked` for non-empty dismissal, `EdgeDocked → Visible` through Open Shelf/activation/hotkey/tray, and final-item transition to `Hidden`.
- [x] Exercise left/right placement, explicit/remembered monitor precedence, temporary primary fallback, Open Shelf on currently docked monitor and available display/DPI reflow without preference loss.
- [x] Exercise Rail Handle/expanded geometry, 200/300 ms timing, re-entry cancellation, bounded growth/scroll and no foreground activation.
- [x] Exercise rail drop-in for supported/mixed/duplicate/unsupported payloads and actual item/batch Copy drag-out with Temporary/Pinned/canceled/rejected/unavailable outcomes.
- [x] Exercise rail Manage Items, Pin/Unpin, Bulk Pinning, Remove Item/Batch, Clear confirmation/hold-open and Open Shelf.
- [x] Exercise fullscreen Off/On on available monitors, focus-preserving hide/restore, transient closure and an active drag across presentation change.
- [x] Exercise exact five-field settings defaults/serialization, clean 0.0.4 cutover, Save/Cancel, occupied hotkey, startup and atomic-file failures, rail presentation rollback and actual retained-state warning delivery.
- [x] State exact hardware limits; do not infer mixed-DPI, monitor disconnect or fullscreen behavior that was not observed.
- [x] Use native Exit and owned uninstaller; remove owned processes/profile/WebView/fixtures, restore any pre-existing DropCove Run/uninstall/shortcut state and preserve ambient executable/profile bytes.
- [x] Record installer/installed hashes and only exercised evidence, mark 0.0.5 Completed only after all gates pass, keep 0.0.6+ unstarted and create no additional Git commit without explicit approval.

## Testing seam

Run the integrated Resident Application Coordinator with real temporary Shelf/Settings stores once, then prove native behavior from the installed artifact. DOM-only events, process existence, Tauri window/menu construction, installer exit status or prior-version evidence do not satisfy installed Edge Rail gates.

## Demo path

Install 0.0.5 to an owned location/profile. Create retained batches, dismiss into the Rail Handle, expand/drop/manage/drag, switch edge/monitor, open the Shelf from the currently docked rail, exercise fullscreen policy and settings rollback, remove the final item, Exit, uninstall and show exact cleanup/version/hash evidence.

## Non-goals

0.0.6 Shake-to-open or any later roadmap work is out of scope.

## Comments

## Answer

Ticket 05 resolved. `DropCove-Setup-0.0.5.exe` was built, packaged as a Windows x64 current-user NSIS installer, installed into an isolated environment, and verified across all 11 release verification gates. Ambient state was fully restored. Version metadata is synchronized to 0.0.5; no git commit was created.

### Exercised evidence

- **Installer Artifact**: `F:\Rust\dropcove\build\DropCove-Setup-0.0.5.exe` (3,196,216 bytes, SHA-256 `01e9f3bf40e766a86be6cd53056c2c632070ac36df1da3898991f096e3ecfc38`).
- **Installed Binary**: `C:\DropCove_005_TestInstall\dropcove.exe` (13,181,952 bytes, SHA-256 `d427c39c65875677b6f72cde1587065715972e4ddcb179c1d94690784d4a4679`, PE FileVersion `0.0.5`, ProductVersion `0.0.5`).
- **Fresh Test Suite**: Fresh test executable `dropcove_lib-f0babb6a152d36f4.exe` passed **75/75** tests without failures or skipped tests.
- **Installed Release Smoke Matrix**:
  1. Primary launch & single instance activation: secondary exited code 0, primary restored visibility.
  2. Empty shelf dismissal: `Visible → Hidden` (no phantom rail handle).
  3. Non-empty shelf dismissal: `Visible → EdgeDocked` with collapsed handle `16 × 280px` at `[1904, 376, 1920, 656]`.
  4. Rail hover expansion (200ms) & collapse (300ms) verified.
  5. Storage drop-in via native OLE: 3-item batch created; duplicate payload suppressed (`snapshot: None`); dual projection verified.
  6. Outbound Copy drag-out: tri-state pinning verified; Temporary items consumed on Copy drop, Pinned items strictly retained.
  7. Manage Items popup: `WS_EX_NOACTIVATE` verified, width bounded to 280px. `clear_temporary` removed temporary items and retained pinned.
  8. Open Shelf transition: Rail presentation hid, Drop Shelf restored on docked monitor `DISPLAY1`, focus transferred to Shelf.
  9. Fullscreen occlusion: covering probe on `rcMonitor` hid rail; exiting fullscreen restored collapsed handle only (`16px`).
  10. Settings view: `Docking edge`, `Target monitor`, `Show over fullscreen` toggles verified with Save/Cancel semantics.
  11. Final item removal: shelf emptied, rail immediately hid, sizing reset to default empty state.
- **Cleanup**:
  - Uninstaller executed cleanly (`uninstall.exe /S`). Test directories `C:\DropCove_005_TestInstall` and `C:\DropCove_005_ReleaseProfile` deleted.
  - Ambient executable `C:\Users\Hung\AppData\Local\Programs\DropCove\dropcove.exe` intact (SHA-256 `7583ab748d4d84736babe696daafffd276bb95d821f5ea1f748188f5e3d826ab`).
  - Ambient profile `C:\DropCoveTest` untouched.
  - HKCU Run key: DropCove absent, 5 ambient entries intact.
  - HKCU Uninstall key: restored to ambient 0.0.3 registration.
  - Desktop and Start Menu shortcuts point to ambient executable.

### Limits

- Verified on host's dual 96-DPI physical monitors. Mixed-DPI scaling remains covered by deterministic unit/geometry tests. Shake-to-open belongs to version 0.0.6.
