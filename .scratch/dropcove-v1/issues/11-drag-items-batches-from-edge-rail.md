# 11: Drag Shelf Items and Shelf Batches from the Edge Rail

**What to build:** Let users perform the established one-item and whole-batch native drag workflows directly from the Edge Rail without opening the full shelf or changing lifecycle semantics.

**Blocked by:** 10: Expand and configure the no-activate Edge Rail.

**Status:** ready-for-agent

- [ ] A single-item Shelf Batch can be dragged directly from the collapsed or interactive rail.
- [ ] A multi-item Shelf Batch can be dragged as one native Copy operation directly from the rail.
- [ ] A batch flyout exposes individual Shelf Items from a multi-item batch for one-item drag-out.
- [ ] Rail drag-out preserves focus until a user explicitly opens the full shelf.
- [ ] Temporary, pinned, mixed-batch, Missing, Unavailable, cancellation, and failure behavior matches the unified Drop Shelf exactly.
- [ ] Flyout interaction prevents premature rail collapse and closes coherently after drag completion or cancellation.
- [ ] Application-seam tests are reused rather than duplicating rail-specific lifecycle logic.
- [ ] Packaged smoke scenarios exercise item, batch, folder, pinned, temporary, canceled, and failed rail drag-out.