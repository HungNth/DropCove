# 04: Drag one Shelf Item out safely

**What to build:** Let users pin, remove, and natively drag one Shelf Item to a compatible destination while preserving source-file safety and the confirmed temporary/pinned lifecycle.

**Blocked by:** 03: Drop filesystem paths into Shelf Batches.

**Status:** ready-for-agent

- [x] Users can pin and unpin a Shelf Item, and the visible pin state updates immediately.
- [x] Users can remove one Shelf Item reference without moving, deleting, renaming, or overwriting the source filesystem object.
- [x] Dragging one Shelf Item uses native OLE with Copy-only semantics and interoperates with Explorer.
- [x] A destination-accepted drag removes a participating Temporary Item and retains a participating Pinned Item.
- [x] Cancellation, rejection, and failure retain valid participating references unchanged.
- [ ] If a destination accepts a drag and DropCove terminates before committing removal, restart may retain the reference; no recovery state incorrectly consumes it.
- [x] Empty Shelf Batches disappear after their last item is removed or consumed.
- [ ] Application-seam tests cover pin transitions, manual removal, accepted/canceled/failed outcomes, empty-batch cleanup, and at-least-once crash semantics.
- [x] Installed-EXE smoke scenarios exercise accepted and canceled one-item drag-out to Explorer.

## Comments

- 2026-09-29: The native adapter uses standard XAML `CanDrag="True"`, `DragStarting`, `DropCompleted`, `DataPackage.SetStorageItems`, and Copy-only `AllowedOperations`/`RequestedOperation`. `DropShelfManager` commits lifecycle changes only after the native result reports Copy.
- 2026-09-29: Nine Release application-seam tests pass, including pin/unpin, manual removal, accepted temporary consumption, pinned retention, canceled/rejected/failed retention, and empty-batch cleanup.
- 2026-09-29: Installed-EXE UI smoke verified immediate visible pin state and manual removal returning the shelf to empty while the source file remained unchanged.
- 2026-09-29: User verified live one-item drag-out to File Explorer using standard `CanDrag` on the installed application: accepted drag copied to Explorer and removed temporary item; pinned item was retained; Esc canceled drag and kept references unchanged.
- 2026-09-29: At-least-once persistence crash window semantics will be fully exercised in Ticket 06 (Restore shelf through SQLite).
- 2026-09-29: User confirmed the current interactive Ticket 04 behavior. The crash-window and corresponding persistence test boxes remain open because Ticket 06 has not implemented or tested restart persistence yet.