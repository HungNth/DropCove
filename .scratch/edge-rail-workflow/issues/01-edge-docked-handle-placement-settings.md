# 01: EdgeDocked Rail Handle and Placement Settings

Status: resolved
Type: task
Version: 0.0.5
Parent: [Specification](../spec.md)
Blocked by: None
Implementation approval: approved by user on 2026-10-13

## What to build

Replace the temporary 0.0.4 non-empty hide behavior with a working `EdgeDocked` state and collapsed no-activate Rail Handle. Deliver the complete edge/monitor placement preference path, remembered dismissal monitor ownership and Open Shelf return path so this first slice works end to end before expansion or rail actions are added.

## Acceptance criteria

- [x] Single resident coordinator owns Hidden/Visible/EdgeDocked and commits placement with visibility.
- [x] Empty dismissal/final-item removal hide all surfaces; nonempty dismissal docks.
- [x] Hidden Start retains items without showing either surface.
- [x] Native no-activate Rail Handle is 16 × 280 logical pixels, centered and edge-aligned with correct corners.
- [x] Right/selected-monitor defaults and overlay placement implemented.
- [x] Explicit/remembered/Shelf-monitor precedence and primary fallback implemented and tested.
- [x] Actual dismissal monitor persists; display-only fallback preserves remembered/configured preferences.
- [x] Open Shelf uses current docked monitor, activates Shelf and hides rail.
- [x] Five required settings fields with Rail defaults and strict clean cutover implemented.
- [x] Missing/unknown/invalid schema and old 0.0.4 file preservation covered by real-store checks.
- [x] Functional edge/monitor drafts, Save/Cancel and unavailable saved choice implemented.
- [x] No actionable fullscreen control exposed before ticket 04.
- [x] Placement/JSON rollback and incomplete restoration reporting tested.
- [x] Existing cursor-monitor activation entry points retained.

## Testing seam

Coordinator tests use real temporary Shelf and Settings stores plus deterministic Shelf-window/Rail-window and monitor adapters. Cover every display transition, empty/non-empty/final-item boundaries, placement precedence, remembered-monitor writes, fallback non-persistence, Open Shelf placement and settings rollback. Actual no-activate style, native monitor identifiers, work-area/DPI conversion and window geometry require Windows smoke.

## Demo path

Create a non-empty Shelf, dismiss it and observe the Right Rail Handle without focus theft. Change edge and target monitor through Settings, exercise Cancel then Save, dismiss again, disconnect or emulate an unavailable saved monitor and observe temporary primary fallback without preference loss. Choose Open Shelf and observe the Drop Shelf on the rail's current monitor; remove the final item and observe all surfaces hidden.

## Non-goals

Rail expansion, timers, drop-in, item actions, drag-out, fullscreen behavior and release/version advancement are out of scope.

## Comments

## Answer

Ticket 01 resolved. Resident presentation atomically stores visibility and rail placement; native rail adapter uses persistent Windows display interface identifiers, not HMONITOR or volatile display numbers. Settings UI shows distinct display labels and preserves unavailable preferences.

### Exercised evidence

- `pnpm build` passed vue-tsc/Vite. Final cargo-xwin release build with embedded frontend succeeded; freshly compiled Windows suite passed **71/71**. Vendor CRT PDB warnings remain nonfatal; version metadata remains 0.0.4.
- Actual owned-profile HWND/visual smoke proved Right handle `[1904,376,1920,656]`, Left secondary handle `[1920,376,1936,656]`, and primary fallback `[0,376,16,656]` on two 96-DPI displays. Screenshots observed the real handle and 420×520 Settings with functional edge/monitor controls and no fullscreen control.
- Native hover preserved an owned foreground probe. Physical handle click activated Shelf and hid rail. Open Shelf used secondary monitor bounds `[2782,422,2978,611]` despite previously remembered primary placement. Later dismissal persisted the actual secondary identifier.
- Actual controls proved draft/Cancel preserve JSON and placement; Save changes docked edge/monitor and serializes exactly five fields. FILE_SHARE_READ lock induced post-effect ReplaceFileW error32; previous JSON and Left secondary placement were restored. Unavailable saved preference remained selectable/persisted; WM_DISPLAYCHANGE fallback did not overwrite remembered monitor.
- Hidden restart restored retained content while both surfaces stayed hidden; hotkey showed/docked it. Final-item removal reset sizing and hid all surfaces; normal secondary showed empty Shelf and empty dismissal remained Hidden.
- Smoke discovered and fixed a real show/hide mismatch: native show bypassed Tao's cached visibility flag, making its hide a no-op. Native show/hide now use symmetric Win32 positioning, with an explicit hide postcondition. Direct command and physical-click failure repro both passed afterward. Native adapter behavior is smoke evidence, not claimed mock-test coverage.
- Final native Exit returned0. Attached tabs and owned foreground probe closed; original absent Run value and five unrelated entries were unchanged. WebView cleanup initially raced teardown (WinError145); retry succeeded and the owned fixture root was verified absent. User installation/profile data was not modified; no installer or commit created.

### Limits

- Live mixed-DPI and physical monitor disconnect/reconnect were unavailable; deterministic monitor/geometry tests plus saved-unavailable preference and display-event smoke cover the exercised paths. Schema cutover and incomplete rollback use permanent real-store/deterministic-adapter checks, not unobserved live system failure.
- Expansion, timers, rail intake/management/drag-out and fullscreen behavior remain tickets02–04. Ticket02 is unblocked, not started.
