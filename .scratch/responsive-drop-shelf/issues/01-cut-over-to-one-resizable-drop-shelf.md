# 01: Cut over to one resizable Drop Shelf

**Parent specification:** [Responsive Resizable Drop Shelf Specification](../spec.md)

**What to build:** Replace the fixed Compact/Expanded presentations with one native, borderless Drop Shelf that users can resize from any edge or corner. Preserve the user's preferred logical size while content exists, restore it safely across monitors and restarts, and reset the workflow completely when the final Shelf Item disappears.

**Blocked by:** None (can start immediately).

**Status:** complete

**Testing seam:** Use the existing Drop Shelf manager as the automated state seam and real temporary SQLite databases as the persistence seam. Use an installed self-contained Release build for native resize hit testing, work-area clamping, and actual window restoration.

**Demo path:** Add one item, resize the Drop Shelf, hide/show it, restart DropCove, move or summon it on another work area, then remove the final item and summon it again.

- [x] Compact and Expanded display states, APIs, fixed-size selection, surfaces, footer controls, tests, and obsolete comments are removed in a clean cutover; no aliases, compatibility paths, or hidden presentation modes remain.
- [x] The remaining display model supports `Hidden`, one visible Drop Shelf state, and `EdgeDocked` while preserving current hotkey, shake, close, tray, and Edge Rail transitions.
- [x] The borderless Drop Shelf resizes naturally from every edge and corner without aspect-ratio locking, using an approximately `8` logical-pixel hit target and a subtle hover affordance.
- [x] Actual bounds respect a `180 × 180` logical minimum whenever the monitor work area permits and never extend outside the current work area; a smaller work area takes precedence over the minimum.
- [x] Preferred logical size is distinct from actual DPI-scaled and work-area-clamped bounds, is shared across monitors, and is not overwritten by positioning, restoration, DPI conversion, clamping, content refresh, or other programmatic resize operations.
- [x] A completed user resize updates the preferred size. It is persisted while content exists; resizing an empty visible shelf becomes durable when the first item is accepted.
- [x] Preferred size uses a dedicated additive singleton settings record with valid logical width and height of at least `180`; it does not repurpose or rewrite Shelf Batch, Shelf Item, or Edge Rail placement data.
- [x] Restart restores preferred size only when persisted Shelf Items remain. Absence of the record or an empty restored shelf yields the default `180 × 180` opening.
- [x] Every path that removes the final Shelf Item transitions either the visible Drop Shelf or `EdgeDocked` state to `Hidden` and deletes preferred size as part of the persisted state change; remembered Edge Rail placement remains intact.
- [x] Adding or removing content while at least one item remains does not resize the Drop Shelf automatically.
- [x] Automated tests replace obsolete Compact/Expanded assertions and cover visible/Hidden/EdgeDocked transitions, preferred-size save/restore, empty-to-first-item persistence, stale empty-state cleanup, final-item removal from visible and Edge Rail states, and rail-placement independence through real SQLite restarts.
- [x] Installed Release smoke proves resize from all edges/corners, minimum and work-area behavior, hide/show and restart restoration, temporary small-work-area clamping without preferred-size loss, and `180 × 180` reset after final-item removal.

## Comments

- Implemented using Windows App SDK `InputNonClientPointerSource` border and caption regions; no synthetic move/resize initiation remains. Native callback dependencies initialize before the frame-change hook. Preferred-size insertion/deletion shares the content mutation transaction.
- Focused manager/persistence/native run: **60 passed, 0 failed**. SQLite trigger regressions prove rollback when first preference insertion or final preference deletion fails.
- Installed self-contained Release pointer smoke: all eight edges/corners increased the expected dimensions by 25 px; opposite edges remained fixed. Minimum width and height each stopped at 180. Large resize stopped at the 1920 × 1032 work-area limits.
- Real work area temporarily changed to `[600,300,760,460]`: actual window became 160 × 160; SQLite preference remained 425 × 425. Restoring the work area restored 425 × 425. Original system work area was restored in `finally`.
- Hide via UI Automation, global-hotkey message dispatch, restart, and summon on DISPLAY2 each restored 425 × 425. Native caption dragging moved the window without changing its preference. Removing the final batch via the actual UI hid the window, deleted the settings row, and the next summon opened at 180 × 180.
- Runtime monitors were both 96 DPI; mixed-DPI runtime remains part of release qualification. Keyboard hotkey injection did not produce an observed transition; the registered message dispatch was exercised instead. Final installer repackaging and full regression qualification belong to ticket 04.
