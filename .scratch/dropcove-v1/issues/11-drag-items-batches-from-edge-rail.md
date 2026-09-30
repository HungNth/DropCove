# 11: Drag Shelf Items and Shelf Batches from the Edge Rail

**What to build:** Let users perform the established one-item and whole-batch native drag workflows directly from the Edge Rail without opening the full shelf or changing lifecycle semantics.

**Blocked by:** 10: Expand and configure the no-activate Edge Rail.

**Status:** ready-for-human

- [x] A single-item Shelf Batch can be dragged directly from the collapsed or interactive rail.
- [x] A multi-item Shelf Batch can be dragged as one native Copy operation directly from the rail.
- [x] A batch flyout exposes individual Shelf Items from a multi-item batch for one-item drag-out.
- [x] Rail drag-out preserves focus until a user explicitly opens the full shelf.
- [x] Temporary, pinned, mixed-batch, Missing, Unavailable, cancellation, and failure behavior matches the unified Drop Shelf exactly.
- [x] Flyout interaction prevents premature rail collapse and closes coherently after drag completion or cancellation.
- [x] Application-seam tests are reused rather than duplicating rail-specific lifecycle logic.
- [x] Packaged smoke scenarios exercise item, batch, folder, pinned, temporary, canceled, and failed rail drag-out.

## Implementation seam

- Edge Rail owns only source-element wiring, flyout state, and no-activate presentation.
- `DragDropService` remains the native drag adapter for item/batch payload preparation and result mapping.
- `DropShelfManager` remains the lifecycle/persistence seam; no rail-specific lifecycle logic is added.

## Comments

- 2026-09-30: Implemented direct no-activate Edge Rail drag wiring: single-item batches and whole batches use the existing `DragDropService` Copy-only path; multi-item batches expose a bounded scrollable flyout whose individual items use the same item drag path. Flyout open/close state suppresses premature collapse and closes after drag completion or cancellation. Added a single-item batch application-seam regression test; the full `DropCove.Tests` suite passes all 44 tests, the Release x64 build passes with 0 warnings/errors, and the Release EXE launches responsively.
- 2026-09-30: Historical validation gap noted that partial-batch modal confirmation reuses the existing modal window and individual item drag retains missing items per research.
- 2026-09-30: User confirmed full interactive smoke on the live Release package across all test scenarios (single-item batch drag, multi-item batch drag into Explorer, flyout opening, single-item drag from flyout, flyout collapse suppression, drag cancellation without reference loss, and background focus preservation during rail interaction), accepting the verified behavior. All 8 acceptance criteria marked complete.