# 02: Windows Runtime Compatibility

**What to build:** Prove and, where necessary, correct DropCove's real Windows desktop behavior on the modernized frontend stack. Launch the Tauri application, exercise every presentation surface and native file workflow, and pass the produced native test executable plus cargo-xwin release build without changing release metadata.

**Blocked by:** 01 — Compatible Frontend Build Stack.

**Status:** resolved

**Human gate completed:** The checks carried to the exact 0.0.9 candidate were confirmed passed by the user on 2026-10-14; see Ticket 03 for the acceptance record.

**Parent specification:** [DropCove 0.0.9 — Frontend Toolchain Modernization](../spec.md)

## Acceptance criteria

- [x] The real Tauri application starts with the Vite 8 development flow on the configured ports, connects its WebSocket/HMR path when a Tauri dev host is present, and does not watch native source as frontend input.
- [x] Drop Shelf, collapsed and expanded Edge Rail, Manage Items, and Settings open through the existing Tauri window-label dispatch with no router or global store.
- [x] Shelf intake, Shelf Batch and Shelf Item actions, dialogs, Shell Visuals, Settings save/cancel, and native drag-out Copy/Move/cancel retain their existing observable behavior.
- [x] Virtualized Shelf cards, Edge Rail cards, and Manage Items rows respond correctly to scrolling, viewport resize, collection changes, and component teardown under VueUse 15.
- [x] Current-theme rendering preserves layout, body background/text, antialiasing, no-selection behavior, semantic tokens, focus indicators, warning/danger states, and reduced-motion behavior with no modernization regression.
- [x] The complete Windows native test target compiles through cargo-xwin without execution through Wine, and the produced Windows test executable passes when run directly on Windows.
- [x] `cargo xwin build --release --target x86_64-pc-windows-msvc` completes for the application with the modernized frontend available to Tauri.
- [x] Any compatibility defect found is fixed at its shared root without adding router/store shims, duplicate CSS systems, test-only application architecture, or unrelated 0.0.10 UI work.
- [x] No frontend testing framework is added solely for this migration; permanent tests are added only for a newly exposed consumer-visible logic regression that existing high-level seams cannot retain.
- [x] Package, crate, Tauri, PE, and installer version metadata remain `0.0.8` throughout this ticket.

## Testing seam

Use the approved Windows application seam. Launch and interact with the real Tauri surfaces rather than a browser-only mock. Compile native tests with cargo-xwin `--no-run`, execute the generated `.exe` directly on Windows, and run the cargo-xwin release build. Record only observed runtime and command evidence.

## Demo path

Launch DropCove through the Tauri development path. Intake real files and folders, open Shelf, Rail, Manage Items, and Settings, exercise virtualization with large collections, load Shell Visuals, and complete Copy/Move/cancel drag-out. Then run the produced native test executable and cargo-xwin release build while all metadata remains 0.0.8.

## Non-goals

Version 0.0.9 synchronization, final NSIS candidate creation, artifact hashes, the user's final Dark/Light/Windows Contrast gate, UI refinement assigned to 0.0.10, TypeScript 7, Rust architecture changes, installer lifecycle changes, or a Git commit.

## Answer

- Produced native executable `dropcove_lib-2cfed4bcfe9c9ca6.exe`: 101 passed, 0 failed. Compiled with cargo-xwin `--no-run`, executed directly on Windows. Cargo-xwin release build passed; external SDK missing-PDB LNK4099 warnings remain unsuppressed.
- Real native Shelf accepted 202 batches including one 200-item batch; 5 cards mounted in the narrow viewport and 6 after resizing to 420×480. Manage Items mounted 8 rows for 200 items, changed its visible slice after scrolling, and closed with Escape.
- Native pinning changed the authoritative snapshot; Clear Temporary confirmation rendered and Cancel retained all 202 batches. Shell Visuals rendered real file icons; no browser errors were observed.
- Dark, Light, and forced-colors token rendering was visually inspected in the real WebView2 using media emulation. Light surface resolved to white; forced-colors surface resolved to Canvas. Final actual Windows theme acceptance remains the user's release gate.
- Vite 8.3.4 resolved/listened on 127.0.0.1:1420 with strictPort and WebSocket 1421. `vite-hmr` handshake returned `connected`; a disposable watched module emitted a `js-update` frame. A disposable native-source probe emitted no watcher/HMR event; all probes were removed.
- Runtime debug arguments initially conflicted with an existing WebView2 data folder. An isolated `WEBVIEW2_USER_DATA_FOLDER` fixed the smoke environment; uninstrumented native launch also succeeded. No application workaround was added.
- Pre-release Tauri packaging passed after owned smoke residents released the executable; generated `DropCove_0.0.8_x64-setup.exe`. The earlier access-denied attempt is not passing evidence.
- Exact Tauri CLI development build was run directly on Windows using a session-only `cmd /C` Cargo runner to avoid cargo-xwin's Wine runner. With an IPv4 dev URL matching the owned Vite server, native Shelf mounted `/@vite/client`, logged `connecting`/`connected`, successfully invoked Shelf/settings IPC, and reported no browser errors. Existing localhost IPv6 server was preserved. Owned CLI tree/profile were removed after cooperative shutdown.
- Native Settings snapshot/session commands and Rail projection/expansion timing passed. Settings UI and expanded-Rail DOM were not proven by the automation; user explicitly accepted carrying them to Ticket 03's exact-artifact human gate.
- Physical Explorer OLE Copy/Move completion was not exercised. Preparation/cancel and storage intake passed; physical Copy/Move/cancel remains mandatory at that same human gate.
- Version metadata remains 0.0.8.