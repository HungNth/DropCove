# 11: Drag Shelf Items and Shelf Batches from the Edge Rail

**What to build:** Let users perform the established one-item and whole-batch native drag workflows directly from the Edge Rail without opening the full shelf or changing lifecycle semantics.

**Blocked by:** 10: Expand and configure the no-activate Edge Rail.

**Status:** in-progress

- [ ] A single-item Shelf Batch can be dragged directly from the collapsed or interactive rail.
- [ ] A multi-item Shelf Batch can be dragged as one native Copy operation directly from the rail.
- [ ] A batch flyout exposes individual Shelf Items from a multi-item batch for one-item drag-out.
- [ ] Rail drag-out preserves focus until a user explicitly opens the full shelf.
- [ ] Temporary, pinned, mixed-batch, Missing, Unavailable, cancellation, and failure behavior matches the unified Drop Shelf exactly.
- [ ] Flyout interaction prevents premature rail collapse and closes coherently after drag completion or cancellation.
- [ ] Application-seam tests are reused rather than duplicating rail-specific lifecycle logic.
- [ ] Packaged smoke scenarios exercise item, batch, folder, pinned, temporary, canceled, and failed rail drag-out.

## Implementation seam

- Edge Rail owns only source-element wiring, flyout state, and no-activate presentation.
- `DragDropService` remains the native drag adapter for item/batch payload preparation and result mapping.
- `DropShelfManager` remains the lifecycle/persistence seam; no rail-specific lifecycle logic is added.

## Comments

- 2026-09-30: Implemented direct no-activate Edge Rail drag wiring: single-item batches and whole batches use the existing `DragDropService` Copy-only path; multi-item batches expose a bounded scrollable flyout whose individual items use the same item drag path. Flyout open/close state suppresses premature collapse and closes after drag completion or cancellation. Added a single-item batch application-seam regression test; the full `DropCove.Tests` suite passes all 44 tests, the Release x64 build passes with 0 warnings/errors, and the Release EXE launches responsively. Packaged native drag-out smoke evidence remains open, so no acceptance checkbox is closed yet.
- 2026-09-30: Validation gap retained: partial-batch confirmation currently reuses the existing modal `ConfirmWindow`, which activates its owner flow; this is not claimed as satisfying the rail focus-preservation criterion. Individual item drag continues to retain references when path resolution fails, matching `drag-out-research.md` rather than performing batch-only Missing cleanup. Packaged item/batch/folder/pinned/temporary/canceled/failed drag smoke remains required before acceptance.