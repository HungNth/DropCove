# 02: Notification-area Resident Access and Exit

Status: resolved
Type: task
Version: 0.0.4
Parent: [Specification](../spec.md)
Blocked by: 01
Implementation approval: approved by user on 2026-10-13

## What to build

A resident user can show DropCove from its notification-area icon and terminate it only through Exit DropCove. While the Shelf is visible, foreground changes repair its topmost order without taking keyboard focus.

## Acceptance criteria

- [x] The primary Resident Instance creates exactly one notification-area icon and removes it during explicit process exit/teardown.
- [x] Left click and double-click route through the same always-show coordinator operation and place/activate the Shelf on the cursor monitor.
- [x] The initial right-click menu exposes Exit DropCove; the Settings item is added only when ticket 03 provides a real Settings window, never as a no-op control.
- [x] Exit DropCove cleanly closes transient windows, native hooks/drop targets and persistence before terminating the primary process.
- [x] Closing or hiding the Shelf leaves the tray icon and resident process available.
- [x] The tray adapter exposes resident warning notifications for later settings/hotkey/startup failures without requiring a fatal startup dialog.
- [x] Foreground-window events reassert topmost ordering only while the Shelf is visible, using no-activate z-order repair so the foreground application keeps keyboard focus.
- [x] Foreground/display handling is event-driven; no cursor/window polling loop is introduced.
- [x] Tray activation, normal launch and secondary activation share placement/show behavior rather than three divergent implementations.
- [x] Failure to initialize the tray is reported at the resident startup boundary; it does not silently leave an inaccessible hidden resident.

## Testing seam

Coordinator tests use deterministic tray/foreground/window adapters to prove always-show routing, hide-versus-exit lifetime and no-activate repair intent. Actual Windows smoke exercises the real notification-area icon/menu, left/double-click access, explicit Exit and foreground changes against known owned windows.

## Demo path

Hide the Shelf, show it from the notification-area icon, switch to another application and observe DropCove remain topmost without taking focus, then choose Exit DropCove and observe the owned resident terminate cleanly.

## Non-goals

The Settings menu item, warning causes, global hotkey and sign-in startup are delivered by tickets 03–04. No Edge Rail is created.

## Comments

- Implemented `resident_tray.rs` with the installed Tauri `TrayIconBuilder`, `MenuBuilder` and native tray events. It requires an icon and rejects duplicate registration; menu/build/icon errors propagate through startup. Exit/warning entry points belong to the coordinator; Settings is not a no-op menu item. Added maintained `tauri-plugin-notification` 2.6.0 (Rust-only): installed Tauri exposes no warning notification API or public native icon identifier. [Official notification docs](https://v2.tauri.app/plugin/notification/) require installed-app verification on Windows.
- Implemented UI-thread-owned `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` with explicit unhook/subclass removal before exit. A thread-local owner avoids the raw pointer/free-after-shutdown defect in the first adapter draft. Native display changes and Tauri scale changes request existing frontend reflow without showing or activating. Exit waits for pending Shelf/native interactions, closes the real SQLite store for every retained clone, unregisters drop targets, removes the tray and requests process exit; no memory-database replacement or unsafe Tauri state removal is used.
- Built via cargo-xwin with `tauri/custom-protocol`; the newly compiled integrated `dropcove_lib-1a9ebbf412a1544f.exe` reported **47 passed, 0 failed**. Regressions cover hidden/visible foreground policy, busy/failed/retried exit, and actual SQLite close/reopen retention with stale clones rejected. Constant/wiring-only tests were removed. Frontend type/build and Windows NSIS packaging also passed through the configured `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc`; metadata stays 0.0.3 and this is smoke packaging, not the 0.0.4 release. Existing external CRT PDB linker warnings were non-fatal.
- Real Windows smoke: one actual Shell icon named DropCove; physical left-click and double-click restored the same hidden Shelf HWND. Native right-click popup visibly contained only Exit DropCove. Against an owned foreground probe, removing Shelf topmost then changing foreground restored topmost and z-order while the probe retained foreground/focus; a Hidden Shelf was neither shown nor reordered. Clicking the freshly observed Exit popup in one native interaction terminated PID 8092 with code 0, removed all owned windows and SQLite WAL/SHM; restart PID 19848 restored the exact durable Temporary item snapshot. Early double-click/Exit probes used stale/dismissed popup coordinates or did not cause a foreground transition and were not counted as product results.
- Installed smoke used an owned current-user 0.0.3 installation after proving no existing DropCove installation/shortcut. A reversible owned SQLite table rename caused a recoverable tray-show error without ending residency. Windows `ToastNotificationManager` reported notifications Enabled and recorded the actual `DropCove warning` toast with `failed to get shelf snapshot: database error: no such table: shelf_items` in native OS history. A transient banner/Notification Center UI row was not observed, so delivery evidence is OS toast history, not a claimed banner screenshot. Restored the schema, proved same-profile installed secondary exit 0, and exercised a native display-change message while Hidden without focus/visibility change. Installed Exit returned 0 and closed WAL. Cleared only the verified single owned warning entry, exited known processes, uninstalled the owned app (exit 0), verified removal of its registration/shortcut, restored cursor, and removed owned profiles/fixtures/WebView directories. Ambient state and the existing `build/DropCove-Setup-0.0.3.exe` deliverable were untouched. Live mixed-DPI/display transitions and final 0.0.4 release remain unverified.

## Answer

Ticket 02 is resolved: real tray access/Exit, native foreground repair without focus theft, durable exit/restart retention and installed Windows warning delivery passed the evidence above. Ticket 03 is unblocked; version metadata remains 0.0.3 and no commit was created.

