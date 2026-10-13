# 01: Profile-scoped Resident Instance and Activation

Status: resolved
Type: task
Version: 0.0.4
Parent: [Specification](../spec.md)
Blocked by: none
Implementation approval: approved by user on 2026-10-13

## What to build

A normal launch creates one profile-scoped Resident Instance; later launches of the same resolved profile signal it to show the existing Drop Shelf and exit. The Shelf can be hidden without ending residency, with all durable references retained.

## Acceptance criteria

- [x] Resolve/normalize the DropCove profile before acquiring a single-instance identity or opening profile persistence.
- [x] One primary process owns the profile database and resident coordinator; a secondary process never opens a second Shelf state owner or creates global integrations.
- [x] A same-profile secondary launch sends one bounded Activation Request, exits successfully after delivery, and causes the primary to show and activate its existing Shelf.
- [x] Failure to contact an existing primary never falls through to a competing second profile owner; the secondary reports a bounded actionable failure.
- [x] Distinct explicitly owned profiles can run independently without sharing instance identity or activation channels.
- [x] Normal and secondary activation always show rather than toggle; repeated requests are idempotent and may reposition to the current cursor monitor.
- [x] Shelf close/Hide transitions `Visible` to `Hidden` without exiting the primary process, closing the profile database, or changing retained Shelf references.
- [x] In 0.0.4, hiding a non-empty Shelf uses the approved temporary `Hidden` transition; no unfinished Edge Rail or `EdgeDocked` surface is created.
- [x] Show placement selects the monitor containing the cursor, converts retained logical sizing through that monitor's DPI, and clamps complete native bounds to its work area.
- [x] Hidden/Visible presentation remains transient; process restart restores durable Shelf content without persisting a display-state flag.
- [x] The resident coordinator exposes a small interface for launch, show and hide; presentation/adapters do not become durable state owners. Warning publication and Exit are accepted in ticket 02; open Settings is accepted in ticket 03.

## Testing seam

Permanent tests drive profile-scoped primary/secondary outcomes, idempotent activation, Hidden/Visible transitions, cursor-monitor placement requests and retained Shelf state through the Resident Application Coordinator with deterministic single-instance/window adapters. Actual Windows smoke launches two executables with one owned profile and observes one durable owner plus activation of the primary.

## Demo path

Launch with an owned profile, add references, hide the Shelf, launch the executable again with the same profile, and observe the existing process show the same Shelf on the cursor monitor while the secondary exits. Launch a second owned profile and observe independent residency.

## Non-goals

Notification-area access, Settings, global hotkey, Start with Windows, Edge Rail and Shake-to-open belong to later tickets/versions.

## Comments

- Implementation approved on 2026-10-13. Added `windows_instance.rs` and `resident.rs`, integrated pre-Tauri lease acquisition and `CloseRequested::prevent_close()` plus hide; the native drop target remains registered while hidden. Settings/tray controls were not stubbed.
- Profile identity uses canonical directory volume/file identity, not lossy path lowercasing or a truncated path hash. A session-scoped mutex is retained with `OwnedHandle`; native receiver readiness is published after setup and successful activation is acknowledged synchronously on its UI thread.
- Build passed with `cargo xwin build --release --target x86_64-pc-windows-msvc --features tauri/custom-protocol`. The initial build without custom-protocol loaded the development URL and is not counted as embedded-frontend evidence. Existing external CRT PDB linker warnings were non-fatal; application-code compiler warnings were removed.
- Newly compiled integrated Windows suite `dropcove_lib-4e072da51a6f1af0.exe`: **44 passed, 0 failed**. Includes the complete-frame boundary regression and profile alias/lease release test.
- Owned native smoke: simultaneous primary/uppercase-profile secondary startup succeeded; secondary exit 0, one primary HWND remained. Real `WM_CLOSE` hid rather than destroyed that HWND. Subsequent normal launch using a `..` profile alias showed it, retained the exact Temporary item/sizing snapshot, and foreground PID matched primary PID 16540. Repeat activation stayed Visible; separate profile PID 23500 owned its own Shelf/database.
- `GetWindowRect` / `GetClientRect` proved the first placement exceeded work area by 16px horizontally and 9px vertically. After fixing native frame accounting and applying outer bounds, bottom-right outer rect was `[1724,843,1920,1032]`, client 180×180, inside work rect `[0,0,1920,1032]`. After saving 300×220 client sizing, top-left activation was outer `[0,0,316,229]`; restart restored the exact item snapshot and 300×220 client dimensions within the work area.
- An owned mutex without a ready receiver produced the documented 10-second startup error; the rejected secondary exited 1 after its owned dialog was dismissed and never created `shelf.db`. No competing owner or WebView initialization was used as fallback.
- Embedded Shelf was observed through owned WebView CDP only after native bounds verification; its retained file card and controls rendered. Closed managed tabs, stopped/waited only owned processes, restored cursor, and removed owned profiles/fixtures/WebView directories. Ambient user state was untouched.
- Live native DPI was 96 (scale 1); mixed-DPI/multi-monitor hardware transitions were not exercised. Deterministic placement tests cover 144 DPI. No 0.0.4 installer or version bump is claimed.
- **Planning blocker:** the final acceptance criterion requires open Settings, warning publication and exit in ticket 01, but this ticket's Non-goals assigns Settings/notification-area access to later slices. The approved breakdown puts warning/Exit in 02 and Settings in 03. Those entry points are not implemented as no-ops. User decision needed: stage that criterion with its owning tickets, or expand ticket 01's scope before resolving it.
- User approved staging the final interface criterion with its owning tickets. Warning/Exit acceptance remains in 02 and Settings in 03; no version-level behavior was removed.

## Answer

Ticket 01 is resolved: profile-scoped activation and resident hide/show passed the build, 44-test integrated suite and owned native smoke documented above. Interface staging was explicitly approved; ticket 02 is unblocked. Version metadata remains 0.0.3.

