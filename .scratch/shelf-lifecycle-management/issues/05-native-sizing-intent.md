# 05: Native resizing and retained Manual Height Override

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can resize the borderless Shelf naturally and have the core retain exactly the sizing intent they chose. Complete the existing automatic growth/width baseline without treating programmatic growth or monitor clamping as manual height input.

## Acceptance criteria

- [x] All four edges and four corners resize natively with an approximately 8-logical-pixel hit region, appropriate cursor, and accent hover outline.
- [x] Actual vertical/corner resizing that changes height creates Manual Height Override; width-only resizing preserves automatic-height eligibility when no override exists.
- [x] Programmatic growth, restore/reflow, and work-area/DPI clamping do not create or overwrite manual intent.
- [x] Preferred width and nullable manual logical height persist while content exists and restore after process restart.
- [x] Width-only changes do not clear an existing Manual Height Override.
- [x] Empty-shelf intent remains transient and is committed with the first accepted batch; unsupported/duplicate input does not persist or alter it.
- [x] Automatic height follows the existing responsive-row tiers, can grow on accepted content without live shrinking, and recalculates on reopening/restart when no override exists.
- [x] Actual bounds remain inside the selected monitor work area without rewriting preferred intent solely because of temporary clamping.
- [x] Manage Items closes at the start of user resizing, before resize changes detach its origin; programmatic growth is not mistaken for user resize start.
- [x] Removal of the final Shelf Item clears width/manual-height intent and resets the next opening to 180 × 180.
- [x] Failed sizing persistence preserves the previous retained intent and reports the failure.

## Testing seam

Core checks cover width-only versus height-changing intent, pending empty intent, persisted manual override, non-persisted automatic growth, final-item reset, and rollback. Native smoke proves real edges/corners, cursor/outline, DPI conversion/clamping, programmatic-versus-user classification, and management closure at resize start.

## Demo path

Resize width only and add enough batches to change row count; observe automatic eligibility. Resize from bottom and a corner, restart, and observe manual height retention. Reopen with fewer references, remove the final item, then observe default next-opening dimensions. Repeat with Manage Items open and verify immediate resize-start closure.

## Non-goals

No generic sizing framework, new settings UI, polling loop, or fourth display state. Reuse the existing sizing columns and automatic formulas.

## Comments

- Cut over every source/test caller from `save_width` to `ShelfState::save_size(width, manual_height)`. Width-only None preserves the current override. Nonempty settings update atomically; empty width/manual height remain staged until first accepted batch; final-item transitions clear durable and staged sizing. Existing schema and automatic-height formula are reused; no aliases or compatibility path remain.
- Added native Windows resize-session observation and eight 8-logical-pixel client hit regions with appropriate diagonal/horizontal/vertical cursors and accent hover outline. Native WM_SIZING provenance plus actual logical client-height delta decides manual intent; ordinary programmatic size changes do not persist. Native capture release/message posting runs on the UI thread, with packed coordinate values rather than a temporary pointer. SQLite/publishing run on a worker after the native loop, not inside the window procedure.
- Native interactive rectangles and frontend programmatic projection bounds include nonclient extents and stay within current monitor work area. DPI conversion uses the owned HWND. Generic frontend onResized persistence was removed; projection size/position application waits until the interactive session ends. Manage Items closes at resize start without restoring obsolete focus.
- Added six real-SQLite sizing regressions and one native intent boundary check. `pnpm build` passed; cargo-xwin compiled the Windows-targeted suite; direct native test execution reported **34 passed, 0 failed**. Embedded Windows x64 release build passed. External CRT PDB linker warnings remained non-fatal.
- Actual native mouse down/move/up exercised all eight edges/corners: preferred logical width/height changed as expected; horizontal edges preserved manual height; vertical/corner height changes created/replaced the override. Cursor, 8px regions and accent outline were observed on the actual surface. Smoke ran at the available DPI scale 1; live mixed-DPI/multi-monitor transitions were not separately exercised and remain part of integrated release verification.
- Empty-shelf East resize staged width 280, South resize staged height 260 without a settings row. Unsupported input did not persist/change it; first valid batch atomically stored 280/260; duplicate input was a no-op. Further drops respected the manual height.
- Native persistence failure trigger reported the real database error and preserved the exact prior snapshot/settings. A programmatic physical resize outside the native interactive loop changed no retained intent. Preferred 3000 × 1500 persisted while actual complete window bounds clamped to the observed 1920 × 1032 work area; restart restored preferred/manual intent without overwriting it from actual clamping.
- Final-reference Clear reset both manual/preferred state to 180 × 180 and removed sizing settings. Subsequent empty width-only native resize to 400 retained automatic eligibility: six batches produced three rows/292 height with nullable manual height still NULL. Removing two batches did not shrink the live Shelf; restart recalculated four batches/two rows to 236. Interactive horizontal resizing was bounded to the monitor and still did not create manual height. Source bytes remained unchanged.
- Popup start-closure smoke passed on West before any geometry change. The first East probe failed because the pointer hit an overlapping management popup instead of the Shelf: a separate owned repro confirmed the top-level window under that point with WindowFromPoint/GetAncestor. Repositioning so the popup used left placement exposed East; the identical East start invariant then passed (active session, popup released, unchanged size before movement). This was an input-targeting failure, not an unexplained sizing-state assertion.
- Closed all managed CDP tabs and only the known owned app processes. Released SQLite handles and removed both owned sizing/probe profiles, fixtures and WebView data. A short-lived WebView cache lock delayed the probe teardown; subsequent removal succeeded, with final directory existence false. No temporary script, diagnostic tag or obsolete save_width remains in source; ambient Shelf profiles/recovery data were untouched.
- Changes remain uncommitted. Version metadata remains 0.0.2 until release ticket 06; no installer acceptance claimed here.
