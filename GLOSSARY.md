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

**Shelf Batch**:
A grouping of filesystem items that were dragged into DropCove together in a single drop operation.
_Avoid_: Drop group, file list, collection

**Shelf Item**:
An individual filesystem path reference stored within a shelf batch.
_Avoid_: File copy, attachment, payload

**Edge Rail**:
A collapsed UI strip docked at the screen edge indicating held items and allowing quick interaction.
_Avoid_: Sidebar, dock, toolbar

**Shake-to-open**:
The gesture of rapidly moving the pointer back and forth while holding the left mouse button to request that the Shelf appear near the pointer.
_Avoid_: Mouse jiggle, cursor flick
