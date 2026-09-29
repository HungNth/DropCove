# DropCove

DropCove is a temporary drag-and-drop workspace for Windows. It holds references to files and folders between workflows without becoming a file manager, clipboard manager, or file store.

## Shelf

**Drop Shelf**:
The floating surface where users hold and manage referenced files and folders.
_Avoid_: Main window, drop zone

**Edge Rail**:
The narrow screen-edge form of the shelf that remains directly usable while content is held.
_Avoid_: Sidebar, launcher

**Shelf Batch**:
The ordered group of unique paths accepted from one drop operation. The same path may appear independently in batches created by different drop operations.
_Avoid_: Folder, collection

**Shelf Item**:
One occurrence of a referenced file or folder inside a Shelf Batch.
_Avoid_: File copy, attachment

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

**Successful Drag-Out**:
A drag-out operation accepted by the destination. It does not guarantee that the destination completes later internal processing.
_Avoid_: File transfer completion

## Availability

**Missing Item**:
A Shelf Item whose path is confirmed not to exist. A drag attempt may remove its reference from DropCove, but never deletes a filesystem object.
_Avoid_: Unavailable item

**Unavailable Item**:
A Shelf Item whose path cannot currently be accessed because its volume, network location, cloud provider, or permissions are unavailable. Its reference is retained.
_Avoid_: Missing item
