# 06: Release DropCove 0.0.3 from the installed artifact

Status: ready-for-agent
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 03, 04, 05
Implementation approval: approved by the user on 2026-10-13

## What to build

Deliver the complete approved 0.0.3 workflow as a version-consistent Windows x64 current-user installer with exercised evidence, then stop for user review rather than starting 0.0.4.

## Acceptance criteria

- [ ] Tickets 01–05 are resolved with their permanent checks and actual-surface smoke evidence.
- [ ] Canonical version, package/crate/Tauri metadata, installed app version, and artifact name consistently report 0.0.3.
- [ ] Build with cargo-xwin for x86_64-pc-windows-msvc and produce DropCove-Setup-0.0.3.exe using current-user NSIS packaging without Visual Studio/MSVC Build Tools.
- [ ] Run the focused permanent core suite after integration; compilation without execution is not counted as a pass.
- [ ] Install and launch the actual packaged executable using an explicitly owned test profile and embedded frontend, not stale dev assets or a source-build substitute.
- [ ] Exercise Pin/Unpin, mixed Bulk Pinning, item/batch Copy and cancellation, confirmed Clear, explicit removal/source safety, management pointer/keyboard/focus behavior, and resize/restore/reset through the installed app.
- [ ] Observe persistence of lifecycle and sizing across process restart, and retention of references after rejected/canceled or failed transfer.
- [ ] Record only exercised results in roadmap Implemented/Verification and ticket Comments; 0.0.3 becomes Completed only after all gates pass.
- [ ] Release owned process/input/drive/file handles and remove owned throwaway scripts/fixtures/test profiles; preserve ambient user profiles and do not terminate unrelated processes by executable name.
- [ ] Keep 0.0.4 and later versions unstarted and create no Git commit without explicit approval.

## Testing seam

Integrate the existing Core Shelf State/real-SQLite checks once, then prove the installed Windows surfaces with native interactions. Mock event echoes, DOM-only snapshots, unexecuted test binaries, and installer exit status alone do not establish completion.

## Demo path

Install 0.0.3, manage a mixed batch using pointer and keyboard, Pin some items, Copy item and batch references, confirm Clear, restart to verify retained lifecycle/sizing, perform native manual resize and reset, then show the completed roadmap evidence and clean test-data teardown.

## Comments
