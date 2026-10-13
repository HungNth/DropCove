# 06: Release DropCove 0.0.3 from the installed artifact

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 03, 04, 05
Implementation approval: approved by the user on 2026-10-13

## What to build

Deliver the complete approved 0.0.3 workflow as a version-consistent Windows x64 current-user installer with exercised evidence, then stop for user review rather than starting 0.0.4.

## Acceptance criteria

- [x] Tickets 01–05 are resolved with their permanent checks and actual-surface smoke evidence.
- [x] Canonical version, package/crate/Tauri metadata, installed app version, and artifact name consistently report 0.0.3.
- [x] Build with cargo-xwin for x86_64-pc-windows-msvc and produce DropCove-Setup-0.0.3.exe using current-user NSIS packaging without Visual Studio/MSVC Build Tools.
- [x] Run the focused permanent core suite after integration; compilation without execution is not counted as a pass.
- [x] Install and launch the actual packaged executable using an explicitly owned test profile and embedded frontend, not stale dev assets or a source-build substitute.
- [x] Exercise Pin/Unpin, mixed Bulk Pinning, item/batch Copy and cancellation, confirmed Clear, explicit removal/source safety, management pointer/keyboard/focus behavior, and resize/restore/reset through the installed app.
- [x] Observe persistence of lifecycle and sizing across process restart, and retention of references after rejected/canceled or failed transfer.
- [x] Record only exercised results in roadmap Implemented/Verification and ticket Comments; 0.0.3 becomes Completed only after all gates pass.
- [x] Release owned process/input/drive/file handles and remove owned throwaway scripts/fixtures/test profiles; preserve ambient user profiles and do not terminate unrelated processes by executable name.
- [x] Keep 0.0.4 and later versions unstarted and create no Git commit without explicit approval.

## Testing seam

Integrate the existing Core Shelf State/real-SQLite checks once, then prove the installed Windows surfaces with native interactions. Mock event echoes, DOM-only snapshots, unexecuted test binaries, and installer exit status alone do not establish completion.

## Demo path

Install 0.0.3, manage a mixed batch using pointer and keyboard, Pin some items, Copy item and batch references, confirm Clear, restart to verify retained lifecycle/sizing, perform native manual resize and reset, then show the completed roadmap evidence and clean test-data teardown.

## Comments

- Explicitly requested pre-release implementation commit completed: `e2fefcc` — `feat: complete shelf lifecycle, bulk pinning and native sizing` (20 files, tickets 03–05). Earlier baseline is `0cee503`. Release metadata and completion-document changes made afterward remain uncommitted; no additional commit authorization inferred.
- Canonical VERSION, package.json, Cargo.toml, Cargo.lock workspace package, tauri.conf.json, installed PE FileVersion/ProductVersion, Tauri runtime version and HKCU uninstall DisplayVersion all report 0.0.3. Existing version synchronization/copy scripts were reused.
- Built via `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc`, then `node scripts/copy-installer.cjs`. Artifact: `build/DropCove-Setup-0.0.3.exe` (2.80 MiB), Windows x64 NSIS currentUser packaging. SHA-256: `2d385ca2ac09d60f7d069868131fa1fe8ea3569433aac416bb11b343380508c5`.
- Frontend type/build passed during packaging. cargo-xwin compiled the integrated Windows suite with `--no-run`; the newly reported 0.0.3 test executable `dropcove_lib-f21b90718d330f42.exe` was executed directly and reported **34 passed, 0 failed**. A prior old-version test executable was not counted as 0.0.3 suite evidence. Existing external CRT PDB linker warnings were non-fatal.
- No existing DropCove installation was found in HKCU/HKLM uninstall registration before smoke. Ran the produced installer with `/S /D=<owned temp installation>`, exit 0, and launched that installed dropcove.exe (not the source-build executable) with an owned --profile and owned WebView data directory. Installed PE/runtime/registry version 0.0.3 verified; process token Mandatory Integrity RID 8192 proved Medium integrity.
- Actual Windows Forms FileDrop source delivered one ordered installed-app batch containing file/file/folder, Temporary by default; duplicate native intake was a no-op. Installed management popup was independent 280px width, with path/folder presentation. Enter and Space opening, the complete eight-control forward Tab cycle, reverse wrap, mixed→all-Pinned→all-Temporary transitions, X/N summary, and exact-origin focus restoration were exercised.
- Actual installed-app item OLE Copy retained a Pinned file; controlled physical native Escape and rejected self-drop retained Temporary references and management; accepted Temporary item Copy consumed only its participant with matching destination/source bytes. Removing the owned source before drag reported the native storage-path failure while retaining the reference; the fixture was restored.
- Initial native intake/foreground probes lacked the required restored/owned input state and were not counted as product results. After restoring owned windows, native intake passed. The controlled cancellation probe recorded the foreground PID matching installed app PID 32552 immediately before real native Escape, and subsequent reference/popup retention passed; browser focus alone was not accepted as OLE focus evidence.
- Whole-batch native Copy closed management and retained the all-Pinned file/folder while copying the folder's nested source bytes. A later mixed-batch Copy retained only the Pinned file while consuming its Temporary folder reference. Single-card Unpin followed by accepted item Copy consumed its reference and reset sizing. Explicit pinned Remove Batch preserved all source bytes/folder contents.
- Installed confirmation defaulted to Cancel and cancel left the exact snapshot unchanged; confirmed Clear preserved only the exact Pinned reference, hid the all-Pinned footer and left the source folder intact. Real East width-only resize persisted width 400 with manual height NULL; South resize persisted manual height 280. Process restart restored the identical lifecycle/400 × 280 snapshot. Final removal reset the installed Shelf to 180 × 180.
- Closed managed CDP tabs, terminated/waited only known owned installed-app/harness processes, released native input and SQLite handles, then ran the owned uninstall.exe `/S _?=<owned installation>`. Uninstall exited 0, removed the installed executable and its owned HKCU product registration. Removed owned installation/profile/fixtures/WebView/copies and the temporary `smoke-release-drag.ps1`; final owned-directory existence was false. Ambient Shelf profiles/recovery data were untouched. The deliverable installer remains in build/.
- Known distribution/visual limits: installer signature is NotSigned (development release; signing/update hardening remains deferred). Available native smoke DPI scale was 1; live mixed-DPI/multi-monitor transitions were not exercised. No unavailable hardware coverage is inferred. 0.0.4 and later remain unstarted; version is not advanced beyond 0.0.3.
