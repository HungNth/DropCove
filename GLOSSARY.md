# DropCove Domain Glossary

DropCove is a resident Windows utility providing a temporary drag-and-drop workspace for file and folder path references.

## Core Language
**Resident Instance**:
The single DropCove process that owns one DropCove profile, its global integrations, and its presentation surfaces.
_Avoid_: Background copy, tray process

**Activation Request**:
A request that the Resident Instance show and activate the Shelf without creating another state owner.
_Avoid_: Relaunch, second instance

**Hidden Start**:
A sign-in launch that restores durable state and global integrations while leaving all Shelf surfaces hidden.
_Avoid_: Background mode, minimized launch


**Shelf**:
The persistent workspace holding referenced files and folders organized into batches.
_Avoid_: Clipboard, storage bin, drop bucket

**Drop Shelf**:
The floating window surface that presents the Shelf for direct interaction and management.
_Avoid_: Shelf, main window

**Dismiss Drop Shelf**:
The user-facing dismissal of the floating Drop Shelf without ending the Resident Instance or discarding Shelf references. An empty Shelf leaves every Shelf surface hidden; a non-empty Shelf remains available through the Edge Rail.
_Avoid_: Exit DropCove, close the application, hide all Shelf surfaces

**Shelf Batch**:
A grouping of filesystem items that were dragged into DropCove together in a single drop operation.
_Avoid_: Drop group, file list, collection

**Shelf Item**:
An individual filesystem path reference stored within a shelf batch.
_Avoid_: File copy, attachment, payload

**Shell Visual**:
A transient file/folder icon or image thumbnail that represents a Shelf Item without becoming part of Shelf state.
_Avoid_: Stored preview, attachment preview, cached Shelf Item

**Edge Rail**:
The screen-edge surface that presents the Shelf in collapsed and expanded states.
_Avoid_: Sidebar, dock, toolbar

**Rail Handle**:
The collapsed state of the Edge Rail.
_Avoid_: Edge Rail, sidebar tab

**Open Shelf**:
The action that shows and activates the Drop Shelf on the display where the Edge Rail is currently docked.
_Avoid_: Expand Edge Rail, open Edge Rail

**Shake-to-open**:
The gesture of rapidly moving the pointer back and forth while holding the left mouse button to request that the Shelf appear near the pointer.
_Avoid_: Mouse jiggle, cursor flick

**Recovery Backup**:
The failed Shelf persistence artifacts retained together after a recoverable database failure for manual recovery.
_Avoid_: Recovered Shelf, salvaged database, automatic repair

**Cooperative Shutdown**:
An update or uninstall request that closes the Resident Instance through normal teardown and proceeds only after confirmed exit.
_Avoid_: Force kill, terminate process

**Installer-Owned Shortcut**:
A Start Menu or Desktop shortcut created and still controlled by the DropCove installer.
_Avoid_: Any DropCove-named shortcut, user shortcut

**Profile Data**:
The durable and recovery information owned by one DropCove profile and retained across update and uninstall.
_Avoid_: Program files, installation directory
