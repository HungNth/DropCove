# DropCove Domain Glossary

DropCove is a resident Windows utility providing a temporary drag-and-drop workspace for filesystem references and Managed Files.

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

**Drop Shelf Utility Menu**:
A transient menu on the Drop Shelf that exposes resident-level surfaces without changing Shelf state.
_Avoid_: Settings menu, notification-area menu

**Settings**:
The DropCove surface for reviewing and changing durable preferences through its Save and Cancel lifecycle.
_Avoid_: Preferences page, configuration window

**Appearance Mode**:
The durable Settings preference selecting System, Dark, or Light presentation for DropCove. Windows Contrast Theme overrides the rendered result without changing the preference.
_Avoid_: Theme preference, color scheme, skin

**About Surface**:
The single informational surface that presents DropCove's product identity and release metadata.
_Avoid_: About page, About popup, About modal

**Update Check**:
A user-requested comparison of the running DropCove version against the Latest Release. It never downloads or installs anything; the user updates by running the newer setup.
_Avoid_: Auto-update, updater, background check

**Latest Release**:
The highest-versioned published DropCove release, excluding drafts and pre-releases.
_Avoid_: Newest tag, latest build

**Dismiss Drop Shelf**:
The user-facing dismissal of the floating Drop Shelf without ending the Resident Instance or discarding Shelf references. An empty Shelf leaves every Shelf surface hidden; a non-empty Shelf remains available through the Edge Rail.
_Avoid_: Exit DropCove, close the application, hide all Shelf surfaces

**Shelf Batch**:
A grouping of Shelf Items accepted from one drop operation.
_Avoid_: Drop group, file list, collection

**Batch Selection**:
The transient, surface-local set of Shelf Batches chosen for one multi-batch drag-out.
_Avoid_: Selected Shelf, persistent selection, batch group

**Drop Placeholder**:
A transient card-shaped marker showing where one incoming Shelf Batch would be inserted while a supported incoming drag is over the Drop Shelf or expanded Edge Rail.
_Avoid_: File placeholder, hover card, ghost batch

**Automatic Shelf Sizing**:
The Drop Shelf height policy that derives height from responsive rows while no Manual Height Override exists.
_Avoid_: Automatic Shelf Growth, auto-grow

**Manual Height Override**:
A user-chosen Drop Shelf height that remains fixed while Shelf content exists.
_Avoid_: Fixed Shelf size, manual Shelf sizing

**Shelf Item**:
An individual filesystem path reference stored within a Shelf Batch. Its path can reference either an external filesystem object or a Managed File.
_Avoid_: File copy, attachment, payload

**Managed File**:
A filesystem file created and owned by DropCove to represent accepted content that did not arrive as a filesystem object.
_Avoid_: Managed Item, temporary file, cached copy

**Pending Managed File Deletion**:
A Managed File whose Shelf Item lifecycle has ended but whose permanent deletion remains pending.
_Avoid_: Orphan file, leaked temporary file, Recycle Bin item

**Temporary Item**:
A Shelf Item with a consumable lifecycle. Its removal can delete a Managed File but never an external source filesystem object.
_Avoid_: Temporary file, cached copy

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

**Brand Visual**:
The canonical visual mark representing DropCove, from which all application icons, tray representations, and installer visual identities derive.
_Avoid_: Logo file, app picture, banner graphic

**Installer Branding**:
The visual presentation of DropCove within the setup wizard, uninstaller, and operating system application management derived from the Brand Visual.
_Avoid_: Installer skin, setup theme, installer decoration

**Profile Data**:
The durable and recovery information owned by one DropCove profile and retained across update and uninstall.
_Avoid_: Program files, installation directory
