# DropCove

DropCove is a temporary drag-and-drop workspace for Windows. It holds references to files and folders between workflows without becoming a file manager, clipboard manager, or file store.

## Shelf

**Drop Shelf**:
The floating surface where users hold and manage referenced files and folders.
_Avoid_: Main window, drop zone

**Edge Rail**:
The narrow screen-edge form of the shelf that remains directly usable while content is held.
_Avoid_: Sidebar, launcher

**Rail Handle**:
The narrow visible resting form of the Edge Rail that expands on pointer hover or drag entry while preserving direct access from the screen edge.
_Avoid_: Hidden hot zone, collapsed shelf

**Shelf Batch**:
The ordered group of unique paths accepted from one drop operation. The same path may appear independently in batches created by different drop operations.
_Avoid_: Folder, collection

**Shelf Item**:
One occurrence of a referenced file or folder inside a Shelf Batch.
_Avoid_: File copy, attachment

## Shelf sizing

**Automatic Shelf Growth**:
The Drop Shelf increases only its height to expose additional responsive-grid rows while no Manual Height Override applies. It does not change shelf width or introduce a separate presentation mode.
_Avoid_: Size-to-content, automatic expansion mode

**Manual Height Override**:
A shelf height explicitly chosen through vertical resizing that suspends Automatic Shelf Growth until the shelf becomes empty. Width-only resizing does not create this override.
_Avoid_: Manual mode, locked size

**Adaptive Rail Sizing**:
The expanded Edge Rail derives its height from Shelf Batch count up to a bounded maximum. It may grow while open but defers shrinking until a later expansion.
_Avoid_: Fixed rail size, Automatic Shelf Growth

## References and lifecycle

**Path Reference**:
A reference to the filesystem entry currently found at a path. It does not preserve the identity or contents of a file that is later moved, deleted, or replaced.
_Avoid_: Stored file, managed file

**Temporary Item**:
A Shelf Item retained until the user removes it, a drag attempt cleans up its confirmed-Missing reference, or a destination accepts its drag-out operation. It survives application and Windows restarts.
_Avoid_: Session item, unpinned item

**Pinned Item**:
A Shelf Item retained after accepted drag-out operations for repeated reuse.
_Avoid_: Permanent file, favorite

**Bulk Pinning**:
A lifecycle action that sets every Shelf Item in one Shelf Batch to the Pinned or Temporary lifecycle state together. Any all-pinned, all-temporary, or mixed status is derived from those items; the Shelf Batch never owns a pinned state.
_Avoid_: Pinned Batch, Pin Batch, Batch-level pin

**Clear Temporary Items**:
A confirmed shelf-wide lifecycle action that removes every Temporary Item reference while preserving every Pinned Item. It never deletes filesystem objects.
_Avoid_: Delete all, Clear batches

**Successful Drag-Out**:
A drag-out operation for which the destination confirms acceptance before the operation ends. If the destination rejects the operation, the user cancels it, or the destination stops before it confirms acceptance, the drag-out is not successful. Acceptance does not guarantee that the destination completes later internal processing.
_Avoid_: File transfer completion

## Availability

**Missing Item**:
A Shelf Item whose path is confirmed not to exist. A drag attempt may remove its reference from DropCove, but never deletes a filesystem object.
_Avoid_: Unavailable item

**Unavailable Item**:
A Shelf Item whose path cannot currently be accessed because its volume, network location, cloud provider, or permissions are unavailable. Its reference is retained.
_Avoid_: Missing item
