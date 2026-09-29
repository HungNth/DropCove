# 04: Drag one Shelf Item out safely

**What to build:** Let users pin, remove, and natively drag one Shelf Item to a compatible destination while preserving source-file safety and the confirmed temporary/pinned lifecycle.

**Blocked by:** 03: Drop filesystem paths into Shelf Batches.

**Status:** ready-for-agent

- [ ] Users can pin and unpin a Shelf Item, and the visible pin state updates immediately.
- [ ] Users can remove one Shelf Item reference without moving, deleting, renaming, or overwriting the source filesystem object.
- [ ] Dragging one Shelf Item uses native OLE with Copy-only semantics and interoperates with Explorer.
- [ ] A destination-accepted drag removes a participating Temporary Item and retains a participating Pinned Item.
- [ ] Cancellation, rejection, and failure retain valid participating references unchanged.
- [ ] If a destination accepts a drag and DropCove terminates before committing removal, restart may retain the reference; no recovery state incorrectly consumes it.
- [ ] Empty Shelf Batches disappear after their last item is removed or consumed.
- [ ] Application-seam tests cover pin transitions, manual removal, accepted/canceled/failed outcomes, empty-batch cleanup, and at-least-once crash semantics.
- [ ] Packaged smoke scenarios exercise accepted and canceled one-item drag-out to Explorer.