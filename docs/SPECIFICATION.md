# DropCove Product Specification

> Canonical specification of target DropCove behavior.

DropCove is a resident Windows utility that provides a temporary drag-and-drop workspace for files and folders. It holds path references between workflows, preserves which items arrived together, and lets users drag them into another application later.

This document defines product behavior, features, settings, installation, update, and removal. Architecture and internal state flows are defined in [ARCHITECTURE.md](ARCHITECTURE.md).

## 1. Product boundaries

DropCove is:

- a temporary workspace for filesystem references;
- optimized for drag-in, temporary holding, and drag-out;
- resident in the Windows notification area;
- available through a hotkey, Shake-to-open, the Edge Rail, or the tray icon;
- persistent across application and Windows restarts.

DropCove is not:

- a file manager;
- a clipboard-history manager;
- a file synchronization or backup tool;
- a content store;
- a cloud service;
- a source-file deletion or move tool.

DropCove never intentionally moves, renames, overwrites, or deletes a referenced filesystem object.

## 2. System requirements

| Requirement            | Current support                                                           |
| ---------------------- | ------------------------------------------------------------------------- |
| Operating system       | Windows 11, build 22000 or later                                          |
| Processor architecture | x64                                                                       |
| Installation scope     | Current user; administrator elevation is not required                     |
| Required prerequisite  | x64 Microsoft Visual C++ Redistributable                                  |
| Drag-and-drop security | Reliable between applications running at the same Windows integrity level |
| Display model          | Per-Monitor V2 DPI-aware; multiple monitors supported                     |

Dragging to or from an elevated application may fail because Windows blocks cross-integrity drag-and-drop. A failed operation retains Temporary Items.

## 3. Installation, update, and removal

### 3.1 Installer

The normal distribution artifact is:

```text
DropCove-Setup-<version>.exe
```

The setup program:

1. checks for the x64 Microsoft Visual C++ Redistributable;
2. installs for the current user without elevation;
3. defaults to `%LOCALAPPDATA%\Programs\DropCove`;
4. remembers a previously selected install directory;
5. always creates a Start Menu shortcut;
6. offers an optional Desktop shortcut, disabled by default;
7. offers **Launch DropCove** on the Finish page, enabled by default.

Wizard sequence:

```text
Welcome → Directory → Shortcut options → Installation progress → Finish
```

The installer does not configure **Start with Windows**. DropCove owns that setting after launch.

### 3.2 First launch

A normal first launch:

- creates the per-user profile directory when needed;
- initializes `shelf.db` and `settings.json` as they become necessary;
- creates the notification-area icon;
- applies default settings;
- attempts to register the default global hotkey;
- attempts to enable Shake-to-open;
- shows and activates the Drop Shelf.

Default profile directory:

```text
%LOCALAPPDATA%\DropCove
```

### 3.3 Updating

DropCove has no automatic updater or background release checker.

To update:

1. run a newer versioned setup executable;
2. setup requests the running resident process to exit;
3. setup waits approximately five seconds;
4. if DropCove does not exit, setup aborts and asks the user to close it;
5. otherwise, files are replaced in the remembered install directory.

Shelf and settings data remain in the per-user profile directory during an update.

### 3.4 Uninstalling

The uninstaller requests the resident process to exit and aborts if it cannot close.

Uninstall removes:

- installed program files;
- Start Menu and Desktop shortcuts created for DropCove;
- Installed Apps registration;
- installer location registration;
- the **Start with Windows** registration.

Uninstall preserves:

```text
%LOCALAPPDATA%\DropCove
```

Therefore `shelf.db`, `settings.json`, and recovery data remain available to a later reinstall. Users who want a complete data removal must delete that profile directory separately after uninstalling.

## 4. Canonical terminology

| Term                       | Meaning                                                                                                                           |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Drop Shelf**             | Floating surface used to hold and manage references.                                                                              |
| **Edge Rail**              | Screen-edge form of the shelf, directly usable while content remains.                                                             |
| **Rail Handle**            | Narrow resting form of the Edge Rail.                                                                                             |
| **Shelf Batch**            | Ordered group of unique paths accepted from one drop operation.                                                                   |
| **Shelf Item**             | One file or folder path occurrence inside a Shelf Batch.                                                                          |
| **Temporary Item**         | Shelf Item removed after Successful Drag-Out unless removed earlier by the user or Missing cleanup.                               |
| **Pinned Item**            | Shelf Item retained after Successful Drag-Out.                                                                                    |
| **Manage Items**           | Item-level inspection and management for a multi-item Shelf Batch.                                                                |
| **Successful Drag-Out**    | Destination reports acceptance with the Windows `Copy` result.                                                                    |
| **Missing Item**           | Path is confirmed not to exist.                                                                                                   |
| **Unavailable Item**       | Existence cannot currently be decided because access to the root, volume, network, cloud provider, or permissions is unavailable. |
| **Automatic Shelf Growth** | Height growth based on responsive rows when no Manual Height Override exists.                                                     |
| **Manual Height Override** | Height explicitly chosen by vertical or corner resizing.                                                                          |

A path reference identifies whatever filesystem entry exists at that path when it is used. If the original object is replaced by another object at the same path, DropCove resolves the replacement.

## 5. Resident process behavior

### 5.1 Single instance

DropCove runs one normal resident instance per Windows user profile.

A second normal launch:

1. signals the existing instance;
2. exits immediately;
3. causes the existing instance to show and activate the Drop Shelf.

### 5.2 Notification area

The notification-area icon remains available while DropCove is resident.

- Left click or double-click: show the Drop Shelf.
- Right click: open a menu containing **Settings** and **Exit DropCove**.
- **Exit DropCove**: terminate the resident process.
- Recoverable startup or integration failures may appear as warning notifications.

Closing the Drop Shelf does not exit DropCove.

### 5.3 Sign-in startup

**Start with Windows** is enabled by default.

At Windows sign-in, DropCove starts with a hidden-start option:

- persisted shelf state is restored;
- global integrations are initialized;
- the notification-area icon is created;
- neither the Drop Shelf nor Edge Rail is shown automatically.

The user can later show the shelf through the hotkey, tray icon, or another launch.

## 6. Display states

| State        | Visible surface | Meaning                                                   |
| ------------ | --------------- | --------------------------------------------------------- |
| `Hidden`     | None            | No DropCove shelf surface is visible.                     |
| `Visible`    | Drop Shelf      | Floating, active management surface.                      |
| `EdgeDocked` | Edge Rail       | Non-empty shelf is available at a configured screen edge. |

Transitions:

- Showing from `Hidden` or `EdgeDocked` produces `Visible`.
- Dismissing an empty `Visible` shelf produces `Hidden`.
- Dismissing a non-empty `Visible` shelf produces `EdgeDocked`.
- Removing the final Shelf Item from either visible surface produces `Hidden`.
- Losing focus does not dismiss the Drop Shelf.

Version staging: until the Edge Rail ships in `0.0.5`, `0.0.4` temporarily maps dismissal of a non-empty `Visible` Shelf to `Hidden`. All references remain durable and resident access remains available. `0.0.5` replaces that staging transition with the canonical `Visible` → `EdgeDocked` behavior above.

## 7. Ways to open DropCove

| Method                    | Result                                          | Placement                 |
| ------------------------- | ----------------------------------------------- | ------------------------- |
| Normal application launch | Shows and activates Drop Shelf                  | Monitor containing cursor |
| Second application launch | Existing process shows and activates Drop Shelf | Monitor containing cursor |
| Global hotkey             | Toggles or shows Drop Shelf                     | Monitor containing cursor |
| Tray icon                 | Shows and activates Drop Shelf                  | Monitor containing cursor |
| Open Shelf from Edge Rail | Shows and activates Drop Shelf                  | Monitor where the rail is currently docked |
| Shake-to-open             | Shows Drop Shelf near cursor                    | Monitor containing cursor |

### 7.1 Global hotkey

Default:

```text
Ctrl + Shift + Space
```

If registration fails at startup, DropCove remains resident and shows a warning. The user can open Settings from the tray and choose another shortcut.

The shortcut recorder:

1. starts when the field is clicked or activated with `Enter` or `Space`;
2. requires at least one of `Ctrl`, `Alt`, `Shift`, or `Win` plus a supported primary key;
3. displays modifiers in `Ctrl + Alt + Shift + Win + key` order;
4. completes on the first valid non-modifier key;
5. disables **Save** while actively listening;
6. cancels on `Escape`, focus loss, or Settings deactivation;
7. lets bare `Tab` leave the field and cancel recording;
8. treats modified `Tab`, `Enter`, and `Space` as shortcut input;
9. retains the previously active hotkey until **Save** succeeds.

If Windows rejects the new combination, Settings stays open, reports the conflict, and preserves the previous registered and persisted shortcut.

### 7.2 Shake-to-open

Shake-to-open is enabled by default. It observes event-driven low-level mouse movement while the left button is held and looks for rapid axis reversals.

| Sensitivity | Minimum movement per accepted direction sample | Required reversals |
| ----------- | ---------------------------------------------: | -----------------: |
| Low         |                                          24 px |                  4 |
| Normal      |                                          10 px |                  2 |
| High        |                                           8 px |                  2 |

Additional rules:

- detection window: `600 ms`;
- trigger cooldown: `500 ms`;
- the hook does not prove that another application is performing a file drag;
- the current `Hidden` or `EdgeDocked` state and placement are saved before showing near the cursor;
- an accepted new Shelf Batch keeps the shelf visible near the cursor;
- button release, rejection, unsupported input, duplicate-only input, cancellation, or a false positive restores the previous state;
- the pointer hook can be disabled in Settings.

## 8. Drop Shelf

The Drop Shelf is a borderless, topmost, resizable floating window.

### 8.1 Window behavior

- Minimum preferred size: `180 × 180` logical pixels.
- Maximum actual bounds: current monitor work area.
- Resizable from all four edges and all four corners.
- Resize hit region: approximately `8` logical pixels around the perimeter.
- Resize hover displays the appropriate pointer and an accent outline.
- The rounded surface background and border align flush with the transparent window frame at all four corners.
- No persistent title/subtitle header is shown.
- The empty Drop Shelf shows only a downward icon and `Drop here`.
- A bottom-left trash icon exposes **Clear Temporary Items** only while temporary content exists.
- Status messages can be dismissed independently.
- Foreground application changes reassert topmost z-order without activating the shelf.
- `Esc` closes Manage Items first; a later `Esc` dismisses the shelf.

### 8.2 Responsive batch grid

| Property                     |                                                 Value |
| ---------------------------- | ----------------------------------------------------: |
| Horizontal outer padding     |                                         8 px per side |
| Gap between rows and columns |                                                  4 px |
| Minimum card width           |                                                164 px |
| Ordering                     | Newest batch first; left-to-right, then top-to-bottom |
| Horizontal scrolling         |                                              Disabled |
| Vertical scrolling           |                               Automatic when required |

Column count is the largest positive integer `n` satisfying:

```text
n × 164 + (n - 1) × 4 ≤ window width - 16
```

Examples:

| Window width | Columns |
| -----------: | ------: |
|   180–347 px |       1 |
|   348–515 px |       2 |
|   516–683 px |       3 |

### 8.3 Automatic Shelf Growth

When no Manual Height Override exists, opening or accepting a new batch derives height from responsive row count:

| Responsive rows | Automatic height |
| --------------: | ---------------: |
|             0–1 |           180 px |
|               2 |           236 px |
|               3 |           292 px |
|       4 or more |           348 px |

Rules:

- width determines column capacity; batch count determines row count;
- a visible accepted batch can grow height but never shrinks it live;
- removing content does not shrink the currently visible shelf while other items remain;
- reopening or restarting recalculates automatic height and may reclaim unused space;
- duplicate-only and unsupported drops do not grow the shelf;
- automatic height is not persisted as manual user intent;
- a width-only resize preserves automatic-height eligibility;
- a resize that changes height through a top, bottom, or corner edge creates a Manual Height Override;
- the Manual Height Override remains until the final Shelf Item is removed;
- removing the final item resets the next opening to `180 × 180`.

## 9. Shelf Batch presentation

### 9.1 Single-item batch

A single-item card displays:

- file/folder icon or image thumbnail;
- truncated display name;
- direct card drag-out;
- compact Pin/Unpin icon action;
- compact Remove Batch icon action.

It does not show Manage Items because its only Shelf Item is already directly accessible. Pin and Remove occupy one trailing icon-control row.

### 9.2 Multi-item batch

A multi-item card displays:

- stacked preview using at most the first three Shelf Items;
- item count/title summary;
- whole-batch drag-out;
- compact tri-state Bulk Pinning icon action;
- compact Manage Items icon action;
- compact Remove Batch icon action.

Visible drag-hint subtitles are not shown on either card type. Full paths and drag instructions remain available through tooltips and accessibility text. Multi-item actions occupy one trailing icon-control row.

## 10. Manage Items

Manage Items provides item-level control for a multi-item Shelf Batch.

### 10.1 Placement and size

- Width: `280` logical pixels.
- Maximum height: `480` logical pixels on the Drop Shelf.
- Placement preference: right, left, below, then above the originating card.
- Fallback: clamp inside the monitor work area with a preferred `16`-pixel margin.
- Only one Drop Shelf Manage Items popup is open at a time.

The Edge Rail uses a matching management flyout constrained to the rail width and a bounded vertical scroller.

### 10.2 Content and actions

Header:

- title and item count;
- `X/N pinned` summary;
- compact tri-state Bulk Pinning icon control;
- compact Close icon control.

Each row exposes:

- icon or thumbnail;
- item name;
- shortened path with full path available through tooltip and accessibility text;
- availability label;
- compact Pin/Unpin icon action;
- compact Remove Item icon action;
- direct item drag-out.

Each header or row action group uses a single trailing icon-control row.

### 10.3 Lifecycle and keyboard behavior

- Pointer opening does not force keyboard focus into the popup.
- Keyboard opening with `Enter` or `Space` focuses the Bulk Pinning control.
- `Tab` and `Shift+Tab` cycle through Bulk Pinning, Close, then each item's Pin/Unpin and Remove controls.
- Closing explicitly restores focus to the originating Manage Items control when it still exists.
- `Esc` closes Manage Items before it can dismiss the Drop Shelf.
- Opening another batch replaces the existing popup.
- The popup closes when its batch is removed, the shelf hides, shelf resizing begins, or whole-batch drag starts.
- Individual item drag keeps Manage Items open while the batch remains.
- During an active item drag, dismissal is guarded; `Esc` cancels the drag before a later `Esc` can close the popup.

## 11. Edge Rail

The Edge Rail appears when a non-empty Drop Shelf is dismissed.

### 11.1 Placement

- Dock edge comes from `RailEdge` in settings; the default is `Right`.
- A non-empty `RailMonitorId` is the explicit target monitor. When connected, it takes precedence over remembered placement. When unavailable, DropCove temporarily uses the Windows primary display without changing the setting.
- An empty `RailMonitorId` requests the remembered dismissal monitor from shelf state. If that monitor is unavailable, DropCove temporarily uses the Windows primary display. If no remembered monitor exists, dismissal uses the monitor containing the Drop Shelf.
- Every non-empty Drop Shelf dismissal stores the actual selected monitor. A display-change fallback while the rail is already docked does not by itself replace the remembered monitor.
- The rail remains centered vertically in the selected monitor work area.
- The rail is an overlay and does not reserve desktop work area.
- Screen-edge corners remain square and both desktop-facing corners are symmetrically rounded with a continuous border. Left docking rounds the right corners; Right docking rounds the left corners.
- Rail interaction does not activate DropCove or steal focus from the current application.
- Keyboard-only users open the Drop Shelf through the global hotkey and use its existing keyboard workflow; the no-activate Edge Rail does not install a second global keyboard hook for rail traversal.

### 11.2 Rail Handle and expansion

| Property                            |               Value |
| ----------------------------------- | ------------------: |
| Rail Handle size                    | 16 × 280 logical px |
| Expanded width                      |      280 logical px |
| Batch row height                    |       48 logical px |
| Row gap                             |        4 logical px |
| Maximum expanded height             |      640 logical px |
| Hover or drag-entry expansion delay |              200 ms |
| Pointer/drag leave collapse delay   |              300 ms |

For a positive batch count, target expanded height is:

```text
max(280, min(640, 52 × batch count + 62))
```

Behavior:

- one through four batches remain at 280 px height;
- five through eleven batches grow through bounded tiers;
- twelve or more batches use 640 px and vertical scrolling;
- new batches can grow an already expanded rail;
- removal does not live-shrink the expanded rail; the next expansion recalculates it;
- removing the final Shelf Item hides the rail immediately;
- an open management flyout or Clear confirmation keeps the rail expanded.

### 11.3 Rail actions

The expanded Edge Rail:

- omits a retained-batch header title;
- supports inspection and drag-out of compact Shelf Batch rows;
- supports drag of one item from Manage Items;
- accepts new files/folders through drop-in;
- supports Pin/Unpin, Bulk Pinning, Remove Item, and Remove Batch;
- shows a bottom-left Clear Temporary Items trash icon only while Temporary Items exist;
- shows an inward Open Shelf arrow at top-right when docked Left and top-left when docked Right.

The collapsed Rail Handle remains the full Open Shelf activation target and retains its vertical label.

### 11.4 Fullscreen policy

**Show over fullscreen applications** is disabled by default.

When disabled, the requested rail hides while the selected monitor is fully covered by the foreground window and returns when that condition ends. When enabled, the rail remains visible over fullscreen windows.

When fullscreen hiding begins, DropCove closes Edge Rail Manage Items and discards any unconfirmed Clear Temporary Items prompt. When fullscreen coverage ends, only the collapsed Rail Handle returns. An already-running native drag is not canceled by this presentation change.

## 12. Drop-in behavior

DropCove accepts Windows storage items that provide non-empty filesystem paths.

### 12.1 Batch creation

One drag operation creates at most one Shelf Batch.

Processing order:

1. discard unsupported entries without paths;
2. remove repeated path strings within the payload using ordinal case-insensitive comparison;
3. preserve the first occurrence and source order;
4. compare the resulting path set with every retained Shelf Batch;
5. add one new Shelf Batch only when no exact retained match exists.

### 12.2 Duplicate Shelf Batch

An incoming batch is duplicate when:

- item count matches a retained batch; and
- every incoming path exists in that retained batch under ordinal case-insensitive string comparison;
- item order is ignored.

DropCove does not canonicalize path strings or rewrite separators for this comparison.

Consequences:

- exact duplicate batch: silent no-op;
- partial path overlap: accepted as a new batch;
- the same path may appear in different Shelf Batches;
- duplicate-only input does not persist, resize, reorder, scroll, or alter shake restoration.

### 12.3 Mixed supported and unsupported input

- If at least one supported path remains, DropCove accepts those paths and reports how many entries were skipped.
- If no supported path remains, no batch is created.

## 13. Temporary and Pinned Item behavior

### 13.1 Temporary Item

Every newly accepted Shelf Item is Temporary by default.

A Temporary Item remains until one of these events:

- Successful Drag-Out;
- Remove Item;
- Remove Batch;
- Clear Temporary Items;
- confirmed-Missing cleanup before batch drag.

Temporary Items survive application and Windows restarts.

### 13.2 Pinned Item

A Pinned Item:

- displays a pin indicator;
- remains after Successful Drag-Out;
- can be returned to Temporary through Unpin or Bulk Unpin;
- can still be removed explicitly through Remove Item or Remove Batch.

### 13.3 Bulk Pinning

For multi-item batches:

| Item states   | Control state | Activation result         |
| ------------- | ------------- | ------------------------- |
| All Temporary | Off           | Pin every item            |
| All Pinned    | On            | Make every item Temporary |
| Mixed         | Indeterminate | Pin every item            |

Bulk Pinning includes Available, Missing, and Unavailable references still present in the batch. It changes lifecycle state only; it does not change path availability, ordering, source files, or shelf sizing.

### 13.4 Removal actions

- **Remove Item** removes one reference and prunes its batch when empty.
- **Remove Batch** removes every reference in that Shelf Batch.
- **Clear Temporary Items** requires confirmation, removes Temporary Items across all batches, and preserves Pinned Items.
- No removal action deletes a source filesystem object.

If persistence fails, the visible and durable state remain unchanged and DropCove reports the failure.

## 14. Drag-out behavior

DropCove supports:

- dragging one Shelf Item;
- dragging every eligible item in one Shelf Batch.

Arbitrary selection across multiple Shelf Batches is not supported.

### 14.1 Copy-only contract

Drag-out offers Windows `Copy` only. It never offers `Move`.

Completion rules:

| Destination result        | Temporary Item | Pinned Item |
| ------------------------- | -------------- | ----------- |
| Accepted `Copy`           | Removed        | Retained    |
| Rejected                  | Retained       | Retained    |
| User canceled             | Retained       | Retained    |
| Failure before acceptance | Retained       | Retained    |

If a mixed batch succeeds, participating Temporary Items are removed and participating Pinned Items remain in the smaller batch.

### 14.2 Availability preparation

Before whole-batch drag:

1. reclassify every path;
2. durably remove confirmed Missing Items;
3. retain Unavailable Items;
4. collect Available Items as drag participants;
5. if both Available and Unavailable Items remain, ask whether to drag only the Available Items;
6. if no Available Items remain, cancel drag and show a status message.

Missing cleanup is not rolled back if the later drag is canceled or rejected.

Before one-item drag, DropCove resolves that item's storage object. Failure cancels the drag and keeps its reference unless it has already been removed through a separate lifecycle action.

### 14.3 Acceptance limitation

Destination acceptance is the only completion signal available to DropCove. It cannot atomically coordinate its local database transaction with another application's later processing.

If a destination accepts `Copy` and DropCove terminates before its removal is persisted, a Temporary Item may reappear after restart. Retaining an extra reference is preferred over incorrectly losing one.

## 15. Missing and Unavailable Items

| Classification | Detection                                                                               | Behavior                               |
| -------------- | --------------------------------------------------------------------------------------- | -------------------------------------- |
| Available      | Filesystem attributes can be read                                                       | Eligible for drag-out.                 |
| Missing        | File/directory is absent while its root is reachable                                    | Removed during batch drag preparation. |
| Unavailable    | Root, volume, network, cloud, permission, or another error prevents a reliable decision | Retained and excluded from that drag.  |

Availability is refreshed at restore and drag boundaries. DropCove does not continuously watch every path for changes.

## 16. Settings

### 16.1 Available settings and defaults

| Setting                           | Default                | Values / behavior                                                   | Introduced |
| --------------------------------- | ---------------------- | ------------------------------------------------------------------- | ---------- |
| Global hotkey                     | `Ctrl + Shift + Space` | At least one modifier plus a supported primary key                  | `0.0.4`    |
| Start with Windows                | On                     | Adds/removes current-user sign-in registration                      | `0.0.4`    |
| Docking edge                      | Right                  | Left or Right                                                       | `0.0.5`    |
| Target monitor                    | Use remembered monitor | A display identifier or remembered placement; unavailable saved displays remain selectable | `0.0.5`    |
| Show over fullscreen applications | Off                    | Controls Edge Rail fullscreen hiding                                | `0.0.5`    |
| Enable shake gesture              | On                     | Installs/removes global pointer hook                                | `0.0.6`    |
| Shake sensitivity                 | Normal                 | Low, Normal, High                                                   | `0.0.6`    |

There is no setting to disable the Edge Rail while retaining its standard non-empty dismissal behavior.

Settings only expose controls whose owning runtime capability has shipped. `0.0.4` exposes Global hotkey and Start with Windows; later versions extend the persisted contract through a clean cutover when Edge Rail and Shake-to-open become active.

### 16.2 Save and cancel behavior

- Settings changes remain drafts until **Save** succeeds.
- **Cancel** or closing Settings discards unsaved changes.
- The current registered hotkey is retained while a new hotkey is tested; startup registration and atomic JSON persistence are then applied.
- Only integrations owned by the active version participate. Edge Rail and Shake integration are added with their owning versions.
- A failure attempts to restore the previous configuration.
- If complete rollback is impossible, DropCove reports the retained safe state through a warning.

### 16.3 Persisted settings contract

Default path:

```text
%LOCALAPPDATA%\DropCove\settings.json
```

Serialized content in `0.0.4`:

```json
{
    "HotKey": {
        "Control": true,
        "Alt": false,
        "Shift": true,
        "Windows": false,
        "VirtualKey": 32
    },
    "StartWithWindows": true
}
```

Later owning versions extend this object with their settings; `0.0.4` does not persist inactive Rail or Shake fields.

Serialized content in `0.0.5`:

```json
{
    "HotKey": {
        "Control": true,
        "Alt": false,
        "Shift": true,
        "Windows": false,
        "VirtualKey": 32
    },
    "StartWithWindows": true,
    "RailEdge": "Right",
    "RailMonitorId": "",
    "ShowRailOverFullscreen": false
}
```

Serialized content in `0.0.6`:

```json
{
    "HotKey": {
        "Control": true,
        "Alt": false,
        "Shift": true,
        "Windows": false,
        "VirtualKey": 32
    },
    "StartWithWindows": true,
    "RailEdge": "Right",
    "RailMonitorId": "",
    "ShowRailOverFullscreen": false,
    "ShakeEnabled": true,
    "ShakeSensitivity": "Normal"
}
```

Each schema extension (`0.0.5` and `0.0.6`) is a clean cutover: all top-level fields for the active version are required and unknown fields remain rejected. Earlier-version settings files without the newer fields follow the normal malformed-settings path: preserve the original bytes, load active defaults in memory, warn, and replace the file only after a successful explicit Save.

`RailEdge` and `RailMonitorId` are user preferences. An empty `RailMonitorId` selects the remembered dismissal monitor stored in shelf state; a non-empty value takes precedence whenever that display is connected.

The numeric virtual-key value is stable across keyboard layouts. Its displayed punctuation label follows the current Windows keyboard layout.

Missing settings load defaults without warning. Malformed or unreadable settings remain untouched, load defaults in memory, and produce a warning notification. A successful explicit Save replaces the file atomically through a temporary file.

## 17. Persistence and recovery

DropCove restores Temporary and Pinned Items after application or Windows restart.

Persisted shelf data includes:

- Shelf Batch identity, creation time, and order;
- Shelf Item identity, order, path, name, file/folder type, and pinned state;
- remembered Edge Rail dismissal monitor;
- preferred shelf width and optional Manual Height Override while content exists.

Not persisted:

- source file contents;
- icon or thumbnail bytes;
- current availability classification;
- transient popup/flyout state;
- automatic height as manual intent.

If the shelf database is corrupt or cannot be opened through a recoverable storage/format failure, DropCove:

1. preserves the failed database and sidecar files under a timestamped `corrupt` name;
2. creates a clean database;
3. starts with an empty shelf;
4. warns the user and provides the backup path.

## 18. Multi-monitor and DPI behavior

- All documented dimensions are logical pixels.
- Physical window bounds scale to each target monitor's DPI.
- Drop Shelf and Edge Rail remain inside the monitor work area.
- Hotkey, tray, and ordinary launch use the cursor monitor.
- Shake-to-open uses the current cursor monitor and nearby placement.
- Edge Rail follows the configured-monitor and remembered-monitor precedence defined in section 11.1 instead of following the cursor.
- An unavailable configured or remembered monitor temporarily resolves to the Windows primary display. This fallback does not rewrite settings, and a display-change fallback alone does not replace remembered placement.
- Display, device, and DPI changes trigger event-driven repositioning or reflow.

Top and bottom Edge Rail docking are not supported.

## 19. Accessibility, theme, and motion

- Action controls expose accessible names and use inline SVG icons that inherit the current text/system color.
- Icon actions expose native tooltips, hover treatment, and visible `focus-visible` indicators.
- Binary Pin controls expose `aria-pressed`; tri-state Bulk Pinning exposes `role="checkbox"` with `aria-checked="false"`, `"true"`, or `"mixed"`.
- Keyboard users can open and close Manage Items, traverse actions, activate buttons with standard Enter/Space behavior, and restore focus to the originating control.
- Full paths and drag instructions remain available through tooltips and accessibility text when visual text is shortened or omitted.
- Theme-aware resources support Light, Dark, and High Contrast presentation.
- Entrance, insertion, removal, and rail transitions are short opacity/scale effects.
- Non-essential motion is skipped when Windows disables client-area animations.
- Drag-and-drop and state changes remain functional when animations are disabled.

## 20. Current limitations and deliberate exclusions

- Files and folders must expose filesystem paths.
- Stream-only or virtual shell content is not materialized into DropCove storage.
- Drag-out is Copy-only.
- Cross-integrity drag-and-drop is not guaranteed.
- Arbitrary multi-selection across Shelf Batches is not supported.
- Edge Rail supports only Left and Right docking.
- No automatic updater.
- No clipboard history, text snippets, URL items, accounts, cloud sync, plugins, named shelves, file synchronization, or application-owned file-copy storage.

## 21. End-to-end example

1. The user drags `A.png`, `B.png`, and `C.png` into the Drop Shelf.
2. DropCove creates one three-item Shelf Batch.
3. The user separately drops `logo.svg`; DropCove creates a newer one-item Shelf Batch.
4. The user pins `logo.svg`.
5. The user successfully drags the first batch into another application with `Copy`; its Temporary Items are removed.
6. The user successfully drags `logo.svg`; it remains because it is pinned.
7. The user dismisses the non-empty Drop Shelf; it becomes the Edge Rail.
8. The user can drag `logo.svg` directly from the rail or open the full shelf again.
