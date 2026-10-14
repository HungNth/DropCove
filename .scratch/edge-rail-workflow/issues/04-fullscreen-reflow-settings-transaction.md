# 04: Fullscreen, Display Reflow and Settings Transaction

Status: resolved
Type: task
Version: 0.0.5
Parent: [Specification](../spec.md)
Blocked by: 03
Implementation approval: approved by user on 2026-10-13

## What to build

Finish Edge Rail runtime policy around fullscreen, foreground, display/DPI changes and settings application. Make Show over fullscreen applications an actionable setting only now that its runtime owner exists, and extend the existing settings transaction so hotkey, startup and rail configuration remain one rollback-safe application.

## Acceptance criteria

- [x] Settings now exposes Show over fullscreen applications, default Off, alongside the ticket 01 Docking edge and Target monitor controls.
- [x] All settings remain drafts until Save; Cancel/window close changes no hotkey, Run registration, JSON, rail placement or fullscreen behavior.
- [x] Save coordinates candidate hotkey, startup registration, edge/monitor/fullscreen configuration, active rail presentation and atomic JSON through one settings transaction.
- [x] Failure at any application or persistence step restores prior file/native/presentation state where possible. Incomplete rollback queries and reports actual retained hotkey, startup and rail state.
- [x] With fullscreen overlay Off, a foreground window that geometrically covers the selected monitor hides requested rail presentation; application identity/name is irrelevant.
- [x] Fullscreen coverage on a different monitor does not hide the selected rail.
- [x] Leaving fullscreen restores only the collapsed Rail Handle when `EdgeDocked` is still requested; it does not restore expanded state.
- [x] Fullscreen hiding closes rail Manage Items and discards unconfirmed Clear Temporary Items prompts without mutating Shelf state.
- [x] Fullscreen presentation changes do not cancel or corrupt an already-running native drag; OLE completion remains authoritative.
- [x] With fullscreen overlay On, the requested rail remains visible over a covering fullscreen foreground window.
- [x] Foreground/fullscreen evaluation, display change and DPI change are event-driven and do not poll.
- [x] Connected configured/remembered monitor precedence and temporary primary fallback remain correct across display events without rewriting settings or remembered placement solely because of fallback.
- [x] DPI/work-area reflow keeps collapsed and expanded rail bounds inside the selected monitor work area and preserves logical dimensions/timers/transient ownership.
- [x] Drop Shelf topmost repair remains independent: it reasserts z-order only while `Visible` and never activates or shows a hidden/EdgeDocked Shelf.
- [x] Missing/malformed/unreadable 0.0.5 settings preserve source bytes and use complete defaults with warning behavior; successful Save serializes exactly the five-field contract.

## Testing seam

Extend the Resident Application Coordinator seam with deterministic foreground geometry, monitor connection/DPI events, rail presentation and settings-integration adapters plus the real temporary Settings store. Inject failures before/after each transaction effect. Native Windows smoke must observe foreground focus, topmost/no-activate behavior, geometric fullscreen coverage on available monitors, display/DPI reflow, active OLE drag survival and actual warning delivery.

## Demo path

Dock/expand the rail, open Manage Items and a Clear prompt, then cover the selected monitor with an owned fullscreen probe. Observe transients close and rail hide; leave fullscreen and observe only the handle. Enable fullscreen overlay and repeat. Change edge/monitor/fullscreen with Cancel and Save, then force occupied hotkey, startup and atomic-file failures and observe exact rollback/retained-state reporting.

## Non-goals

Shake gesture integration, final release/version advancement, automatic update, recovery hardening and final visual-resource work are out of scope.

## Comments

## Answer

Ticket 04 resolved. Geometric fullscreen occlusion, event-driven WinEvent hooks (without `WINEVENT_SKIPOWNPROCESS` on foreground), `ShowRailOverFullscreen` checkbox in Settings with atomic transaction commit/rollback, and active in-flight drag survival are implemented and verified. Version metadata remains 0.0.4; no commit created.

### Exercised evidence

- Fresh full native test suite passed **75/75** tests without filtering. `fullscreen_coverage_evaluation_and_transient_dismissal_without_canceling_drag` verifies `rcMonitor` geometric coverage vs maximized `rcWork`, cross-monitor isolation, transient dismissal, handle restoration, and `show_over_fullscreen` bypass.
- vue-tsc/Vite build and cargo-xwin release builds passed. Release binary: 13,181,952 bytes, SHA-256 `88932f2a2858069bebf645be4da30046f3e8b2365bb24863d4274872c04fddc3`.
- Native smoke matrix verified on dual-monitor topology (`DISPLAY1` primary, `DISPLAY2` secondary):
  - Fullscreen window covering `rcMonitor` on rail monitor hides rail presentation; destroying probe restores collapsed handle only (`16 × 280px`). Fullscreen window on different monitor does not hide rail.
  - `ShowRailOverFullscreen = true` keeps rail handle visible over covering fullscreen window.
  - Settings UI exposes `Show over fullscreen applications` checkbox; Cancel preserves `settings.json`, Save atomically persists through settings transaction.
  - Active modal OLE drag session (`DoDragDrop`) is preserved during fullscreen occlusion transition; `ActiveDrag` hold is retained while UI transients are dismissed.
  - Clean exit via tray menu returned 0; all windows destroyed; ambient registry Run keys and shortcuts verified untouched.

### Limits

- Tested on host's dual 96-DPI physical displays; mixed-DPI scaling remains covered by deterministic unit/geometry tests. Full packaged release verification belongs to Ticket 05.
