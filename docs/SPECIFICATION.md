# DropCove Product Specification

> Canonical specification of target DropCove behavior.

DropCove is a resident Windows utility that provides a temporary drag-and-drop workspace for external filesystem references and DropCove-owned Managed Files. It preserves which items arrived together and lets users drag them into another application later.

This document defines product behavior, features, settings, installation, update, and removal. Architecture and internal state flows are defined in [ARCHITECTURE.md](ARCHITECTURE.md).

## 1. Product boundaries

DropCove is:

- a temporary workspace for external filesystem references and Managed Files created from supported non-filesystem drops;
- optimized for drag-in, temporary holding, and drag-out;
- resident in the Windows notification area;
- available through a hotkey, Shake-to-open, the Edge Rail, or the tray icon;
- persistent across application and Windows restarts.

DropCove is not:

- a file manager;
- a clipboard-history manager;
- a file synchronization or backup tool;
- a general-purpose content store;
- a cloud service;
- a move or deletion tool for external source objects.

DropCove never intentionally moves, renames, overwrites, or deletes an external referenced filesystem object. It permanently deletes only Managed Files that it owns after their Shelf Item lifecycle ends.

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
5. offers an optional Start Menu shortcut, enabled by default;
6. offers an optional Desktop shortcut, disabled by default;
7. offers **Launch DropCove** on the Finish page, enabled by default.

Wizard sequence:

```text
Welcome → Directory → Shortcut options → Installation progress → Finish
```

Shortcut options contract:

- **Create Start Menu shortcut** is interactive, enabled by default, and receives initial focus on the Shortcut options page.
- Unchecking the Start Menu shortcut skips shortcut creation and bypasses the Start Menu conflict check, allowing setup to complete even if an unowned file or shortcut exists at the target path.
- When Start Menu shortcut creation is enabled (default), setup verifies that any existing shortcut at the target path is an Installer-Owned Shortcut, aborting safely (`exit 2`) before writing program files if an unrelated shortcut collision occurs.
- Silent (`/S`) and passive (`/P`) unattended installations create the Start Menu shortcut by default unless the `/NS` switch is passed.
- Passing the `/NS` command-line switch disables both Start Menu and Desktop shortcut creation and bypasses shortcut conflict verification.
- Updating or reinstalling with Start Menu shortcut creation unchecked preserves any pre-existing Start Menu shortcut without modifying or deleting it.
- The Desktop shortcut remains optional and unchecked by default across interactive, silent, and passive runs.

The installer does not configure **Start with Windows**. DropCove owns that setting after launch.
The Brand Visual is shared by the application executable, setup and uninstaller executables and their window icons, Windows application-management entry, Installer-Owned Shortcuts, About Surface, native application windows, and notification-area icon. Preserve the existing mark and colors; generate Windows bundle icon assets from the canonical vector source. Installer Branding uses that same multi-resolution icon without adding custom sidebar or header artwork. Appearance Mode does not replace or recolor the Brand Visual.

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

DropCove has no automatic updater or background release checker. The About Surface offers a user-requested Update Check that reports the Latest Release and opens its release page; it never downloads or installs anything.

To update:

1. run a newer versioned setup executable;
2. setup requests the running resident process to exit;
3. setup waits approximately five seconds;
4. if DropCove does not exit, setup aborts and asks the user to close it;
5. otherwise, files are replaced in the remembered install directory.

Shelf state, settings, retained Managed Files, and pending Managed File deletion state remain in the per-user profile directory during an update.

### 3.4 Uninstalling

The uninstaller requests the resident process to exit and aborts if it cannot close.

Uninstall removes:

- installed program files;
- Installer-Owned Shortcuts in the Start Menu and Desktop (the uninstaller verifies installer ownership before removal and completes cleanly when shortcuts are absent);
- Installed Apps registration;
- installer location registration;
- the **Start with Windows** registration.
Uninstall preserves:

```text
%LOCALAPPDATA%\DropCove
```

Therefore `shelf.db`, `settings.json`, retained Managed Files, pending deletion state, and recovery data remain available to a later reinstall. Users who want complete data removal must delete that profile directory separately after uninstalling.

## 4. Canonical terminology

| Term                              | Meaning                                                                                                                           |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Drop Shelf**                    | Floating surface used to hold and manage Shelf Items.                                                                             |
| **Edge Rail**                     | Screen-edge form of the shelf, directly usable while content remains.                                                             |
| **Rail Handle**                   | Narrow resting form of the Edge Rail.                                                                                             |
| **Shelf Batch**                   | Ordered group of Shelf Items accepted from one drop operation.                                                                    |
| **Batch Selection**               | Transient, surface-local set of Shelf Batches chosen for one multi-batch drag-out.                                                 |
| **Item Selection**                | Transient set of Shelf Items chosen within one Manage Items popup for one multi-item drag-out.                                     |
| **Drop Placeholder**              | Transient card-shaped marker for the newest position of one incoming Shelf Batch.                                                 |
| **Shelf Item**                    | One filesystem path occurrence inside a Shelf Batch, backed by an external object or a Managed File.                              |
| **Managed File**                  | Filesystem file created and owned by DropCove for accepted content that did not arrive as a filesystem object.                    |
| **Website Image Virtual File**    | Image file represented by supplied bytes rather than a durable source filesystem object.                                         |
| **Pending Managed File Deletion** | Managed File whose Shelf Item lifecycle ended but whose permanent deletion still awaits retry.                                   |
| **Temporary Item**                | Shelf Item removed after Successful Drag-Out unless removed earlier by the user or Missing cleanup.                               |
| **Pinned Item**                   | Shelf Item retained after Successful Drag-Out.                                                                                    |
| **Manage Items**                  | Item-level inspection and management for a multi-item Shelf Batch.                                                                |
| **Successful Drag-Out**           | Destination reports acceptance with the Windows `Copy` result.                                                                    |
| **Missing Item**                  | Path is confirmed not to exist.                                                                                                   |
| **Unavailable Item**              | Existence cannot currently be decided because access to the root, volume, network, cloud provider, or permissions is unavailable. |
| **Automatic Shelf Sizing**        | Height derived from responsive rows and updated in both directions when no Manual Height Override exists.                         |
| **Manual Height Override**        | Height explicitly chosen by vertical or corner resizing.                                                                          |

An external path reference identifies whatever filesystem entry exists at that path when it is used. If the original object is replaced by another object at the same path, DropCove resolves the replacement. A Managed File instead has DropCove-owned storage and deletion semantics.

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
- Right click: open a menu containing **Settings**, **About**, and **Exit DropCove**, in that order.
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

### 5.4 About Surface

The notification-area **About** action and the Drop Shelf Utility Menu open the same non-modal, single-instance About Surface. Repeated requests show and focus the existing window without moving it; `Escape` or the window close action destroys it so a later request can recreate it.

The decorated, fixed-size `400 × 460`-logical-pixel surface appears on the Drop Shelf monitor when opened there, or the pointer monitor when opened from the notification area. Opening About from the notification area does not show or change any Shelf surface.

About shows the canonical logo, product name, synchronized running version, the product description directly below that version, Author, and a **Source** link that opens the repository in the default browser. Author is `HungNth`; the description is **A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references.** Author, description, and repository come from Cargo package metadata rather than frontend copies. The repository is not called a Website; homepage remains unset until a public product site exists.

**Check for updates** performs an Update Check only when activated. DropCove requests the repository's GitHub releases, ignores drafts, pre-releases, and tags not matching `v?MAJOR.MINOR.PATCH`, and compares the running version with the highest remaining version. The request aborts after `10 s` and its result is not retained after About closes.

One status line below the action announces each result politely to assistive technology:

| State                  | Action              | Status                                         |
| ---------------------- | ------------------- | ---------------------------------------------- |
| Idle                   | `Check for updates` | empty                                          |
| Checking               | `Checking…`, `aria-disabled`, focus kept | empty                     |
| Up to date             | `Check for updates` | `You're up to date.`                           |
| Newer release          | `Download X.Y.Z`    | `Version X.Y.Z is available.`                  |
| No release             | `Check for updates` | `No releases published yet.`                   |
| Failure or invalid data | `Check for updates` | `Couldn't check for updates. Try again later.` |
| HTTP 403 or 429        | `Check for updates` | `Update check limit reached. Try again later.` |

When a newer release is available, the status is styled as an accent callout with subtle background, accent border, semibold text, and an inline download icon; the action is rendered as a primary accent button (`Download X.Y.Z`) to distinguish it from secondary checks.

`Download X.Y.Z` opens that release's GitHub page in the default browser; release pages outside the DropCove repository's `releases/` path are invalid data. Installer-based updates remain the supported workflow.


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

- Minimum preferred size: `191 × 180` logical pixels; restored preferred width is bounded by the current 191-pixel minimum.
- Maximum actual bounds: current monitor work area.
- Resizable from all four edges and all four corners.
- Resize hit region: approximately `8` logical pixels around the perimeter.
- Resize hover displays the appropriate pointer without changing the Shelf border or adding an accent outline.
- The rounded surface background and border align flush with the transparent window frame at all four corners.
- No persistent title/subtitle header is shown.
- A titleless `28`-logical-pixel control strip contains a top-right `24 × 24` close action named **Hide Drop Shelf**. The action uses Dismiss Drop Shelf: empty content becomes `Hidden`, retained content becomes `EdgeDocked`, Manage Items closes, and the Resident Instance remains running without discarding references.
- The remainder of the control strip serves as the exclusive window-drag region. Movement uses platform-native dragging and is session-only; it closes Manage Items before movement begins, is blocked during native resize, modal confirmations, or transactional Shelf state, and preserves existing card Copy drag-out, scrolling, controls, and eight-direction resize behavior.
- Reopening the Drop Shelf uses canonical cursor-monitor or Edge Rail placement; dragged coordinates are not persisted.
- When no supported incoming drag is over the surface, an empty Drop Shelf shows only a downward icon and `Drop here`.
- During a supported incoming drag, the Drop Shelf shows one `48`-logical-pixel Drop Placeholder at the prospective newest Shelf Batch position. It replaces the empty prompt or precedes retained batches, uses a dashed drag-hover border and a background distinct from the Shelf surface, and does not show a filename, thumbnail, or item count before acceptance.
- Drag entry does not apply a surface-wide background, border, outline, or glow and does not temporarily resize the Drop Shelf. Leaving or completing the drag removes the placeholder; an accepted new Shelf Batch then applies Automatic Shelf Sizing.
- A bottom-left trash icon exposes **Clear Temporary Items** only while temporary content exists.
- An always-visible bottom-right `24 × 24` **More** action opens the Drop Shelf Utility Menu above it. The menu contains **Settings** then **About**, supports standard keyboard navigation, and closes on outside click or `Escape`. Dismissal restores focus unless an opened utility surface receives it. Settings reuses its existing singleton window and Save/Cancel lifecycle.
- The Utility Menu uses a stronger border token than Shelf batch cards and an elevated shadow to remain distinct from the Drop Shelf in Dark and Light themes. Windows Contrast retains a system-color border even when shadows are suppressed.
- Status messages can be dismissed independently.
- Foreground application changes reassert topmost z-order without activating the shelf.
- `Esc` closes an open Selection Actions Menu first, then cancels an open selection-removal confirmation; both retain selection. With no Selection Actions transient, it cancels an active batch availability confirmation and its prepared token. While Manage Items is open, that popup consumes `Esc` according to section 10.4 before it closes. Once no management popup remains, a later `Esc` clears non-empty Batch Selection, then a later press dismisses the shelf. Existing Utility Menu, Item Actions Menu, and Clear confirmation cancellation remain authoritative while those surfaces are open.

### 8.2 Responsive batch grid

| Property                     |                                                 Value |
| ---------------------------- | ----------------------------------------------------: |
| Collection horizontal insets |                                          7 px per side |
| Gap between rows and columns |                                                  4 px |
| Minimum card width           |                                                164 px |
| Ordering                     | Newest batch first; left-to-right, then top-to-bottom |
| Horizontal scrolling         |                                              Disabled |
| Vertical scrolling           |                               Automatic when required |

Column count is the largest positive integer `n` satisfying:

```text
n × 164 + (n - 1) × 4 ≤ window width - 16 - scrollbar channel width
```

Examples without vertical overflow (subtract another 8 px of available width when the native scrollbar is present):

| Window width | Columns |
| -----------: | ------: |
|   191–347 px |       1 |
|   348–515 px |       2 |
|   516–683 px |       3 |

Both horizontal insets stay at 7 px with and without overflow. On the Drop Shelf, the 4-pixel painted scrollbar thumb starts at the content-side edge of its 8-pixel hit channel, so left-border-to-card and rightmost-card-to-visible-thumb clearances are both 7 px. The thumb stays rounded and keeps existing idle, hover, active, and Windows Contrast colors. A fixed 8-pixel collection boundary sits outside the top of the real scrollport; 8 pixels of bottom padding remain inside the scroll range. Short collections remain top-aligned. Ordinary and virtualized collections share these boundaries, and the final Shelf Batch remains fully reachable at maximum scroll. The approved 191-pixel minimum window width remains a conservative floor.

### 8.3 Automatic Shelf Sizing

When no Manual Height Override exists, opening, width-only resize, accepted content, or content removal derives height from responsive row count:

| Responsive rows | Automatic height |
| --------------: | ---------------: |
|             0–2 |           180 px |
|               3 |           230 px |
|               4 |           282 px |
|       5 or more |           334 px |

Equivalent formula:

```text
max(180, min(334, 52 × responsive row count + 74))
```

Six or more responsive rows use `334` px and vertical scrolling.

The compact fit budget retains the 28-pixel control strip, 8-pixel fixed collection top boundary, 8-pixel scroll-range bottom clearance, 24-pixel footer controls with 8 pixels below them, and the surface borders. The footer adds no separate top padding.

Rules:

- width determines column capacity; batch count determines row count;
- an accepted Shelf Batch can grow the visible Drop Shelf to the applicable tier;
- when no Manual Height Override exists, every mutation that reduces responsive rows immediately shrinks the visible Drop Shelf to the applicable tier, including Remove Item, Remove Batch, Clear Temporary Items, Successful Drag-Out consumption, and confirmed-Missing cleanup;
- a width-only resize preserves automatic sizing and recalculates in both directions, including shrinking when the wider grid uses fewer responsive rows;
- automatic resizing keeps the top-left window origin fixed and retains existing work-area clamping;
- reopening or restarting recalculates the applicable automatic height;
- duplicate-only and unsupported drops do not resize the shelf;
- automatic height is not persisted as manual user intent;
- a resize that changes height through a top, bottom, or corner edge creates a Manual Height Override;
- while content remains, content mutations do not change a Manual Height Override;
- removing the final item clears the Manual Height Override and resets the next opening to `191 × 180`.

## 9. Shelf Batch presentation

### 9.1 Single-item batch

A single-item card displays:

- file/folder icon or image thumbnail;
- truncated display name;
- direct card drag-out;
- compact Pin/Unpin icon action;
- compact More icon action when its Shelf Item has Item Actions;
- compact Remove Batch icon action.

It does not show Manage Items because its only Shelf Item is already directly accessible. The trailing controls are Pin/Unpin, More when eligible, and Remove. A Managed Image retains only Pin/Unpin and Remove.

### 9.2 Multi-item batch

A multi-item card displays:

- stacked preview using at most the first three Shelf Items;
- item count/title summary;
- whole-batch drag-out;
- compact tri-state Bulk Pinning icon action;
- compact Manage Items icon action;
- compact Remove Batch icon action.

On both the Drop Shelf and expanded Edge Rail, each multi-item preview occupies a `24 × 24` logical-pixel box. The first Shelf Item is the front layer at the lower-left; each following layer sits behind it, offset `3` logical pixels up and right. The preview region is `30 × 30` logical pixels, bounded to three layers. Each visual retains its natural transparency with no opaque per-layer backing; later layers or the card surface may show through transparent artwork and aspect-fit margins. The stack remains decorative and non-interactive; the existing item-count title, batch drag region, trailing controls, and `48`-pixel card height are unchanged.

Visible drag-hint subtitles are not shown on either card type. External Reference full paths and drag instructions remain available through tooltips and accessibility text. Managed Files use their user-visible name/content rather than a Profile Data path. Multi-item actions occupy one trailing icon-control row.

### 9.3 Batch Selection

Batch Selection is transient presentation state owned by the current surface. It stores stable Shelf Batch identities plus one optional selection anchor; neither is persisted or shared when switching between the Drop Shelf and Edge Rail.

- Plain click selects one Shelf Batch and sets the anchor. `Ctrl+click` toggles one batch without clearing the others and sets the anchor even when the batch is toggled off.
- `Shift+click` replaces the current selection with the inclusive range from the anchor to the target in complete displayed newest-first row-major order, including virtualized batches outside the viewport. Shift does not move a valid anchor. With no valid anchor it selects only the target and establishes it as anchor. `Ctrl+Shift` has the same replacement-range behavior in `0.0.25`.
- Plain pointer press on an already selected card delays collapse until release below the established drag threshold. Crossing the threshold uses the pre-press selection: dragging a selected card drags the complete Batch Selection, while dragging an unselected card drags only that batch.
- The card rule applies to one-item and multi-item Shelf Batches. Direct one-item drag remains available only from Manage Items.
- Independent Pin/Unpin, Bulk Pinning, Manage Items, and accepted drop-in do not change selection. Selection Actions Pin/Unpin also retain selection. Remove, Remove Selected, and confirmed-Missing cleanup prune only identities that disappear; surviving selected batches remain selected, item counts update, and a removed anchor becomes absent without selecting a replacement.
- Rejected or canceled drag-out retains selection when no separate clear trigger occurs. Successful Drag-Out, empty collection-space click, `Escape`, surface hide, surface switch, or restart clears it. Hide or surface switch remains authoritative if a native drag is already active; its prepared participants complete independently and rejection does not restore hidden selection.
- Selected cards use the accent border and accent-subtle surface, with Windows Highlight resources in Contrast Themes. The footer on each surface reports `N batches · M items`, where `M` counts current Shelf Item occurrences in the selected batches before availability preparation. No selection toolbar, mode, checkbox, or per-card count badge is added.

On the Drop Shelf, the batch collection is a multiselectable ARIA grid because cards contain independently operable action controls. Each selectable batch row exposes `aria-selected`; its drag region participates in roving focus. Left/Right and Up/Down move through the responsive row-major layout, `Home` and `End` move to collection boundaries, `Space` selects only the focused batch, `Ctrl+Space` toggles it, `Shift+Space` selects the anchor range, and `Ctrl+A` selects the complete collection without changing a valid anchor. If no anchor exists, `Ctrl+A` leaves it absent, so the next Shift gesture selects only its target and establishes a new anchor. `Escape` follows the ordering in section 8.1.

Pointer selection on the Drop Shelf focuses the card drag region; action-control activation retains control focus. If the focused batch disappears, focus moves to the batch now at the same displayed index, otherwise the previous batch, otherwise the More action when the Shelf becomes empty. Selection changes use the existing polite announcement channel with coalesced batch/item counts or `Selection cleared`.

The no-activate Edge Rail exposes the same pointer gestures, selected visual, and footer count but remains pointer-only. It does not add focusable selection regions, keyboard selection, or a live announcement channel; keyboard users open the Drop Shelf through the global hotkey.

### 9.4 Item Actions Menu

The per-item **More** button opens one transient **Item Actions Menu**. It appears on eligible single-item cards in the Drop Shelf and expanded Edge Rail and eligible rows in Manage Items. A multi-item card has no item menu; use Manage Items to target a specific Shelf Item.

| Authoritative classification | Menu actions, in order |
|---|---|
| External Reference, file or folder | Show in File Explorer; Copy Path |
| Plain-text Managed File | Copy Text |
| Image Managed File | No More button or Item Actions Menu |

Core Shelf State supplies ownership and Managed content kind; presentation never infers them from path substrings or extensions. Requests contain only the Shelf Item identity and requested action, not a caller-provided path or text payload.

- **Copy Path** replaces the Windows clipboard with the exact retained absolute External Reference path as Unicode text, without quotes, newline, normalization, or existence requirement.
- **Copy Text** replaces the clipboard with the full original Managed Text, preserving Unicode and line endings rather than copying a truncated preview. The menu remains available for cached Available, Missing, and Unavailable items. Invocation requires a committed `ready` Managed File record and attempts a live read; `ready` is not an availability classification. Read failure retains the item without cleanup or availability mutation.
- **Show in File Explorer** checks the current External Reference path, selects the file or non-root folder in its parent, and opens a filesystem root directly. Missing/inaccessible paths report an error without mutation. Explicit reveal may activate Explorer; Copy actions never activate the Edge Rail.
- Every action targets only the item whose More button opened it. Batch Selection, Item Selection, anchors, Pin state, Temporary/Pinned lifecycle, ordering, sizing, and display state remain unchanged. Clipboard Copy is not Successful Drag-Out and never consumes a Temporary Item.
- Copy success reports `Path copied.` or `Text copied.` locally; failures identify the operation locally. No Windows toast is emitted, and successful Explorer reveal needs no duplicate banner.

The menu follows the Utility Menu keyboard contract: menu/menuitem semantics, Arrow Up/Down, Home/End, Enter/Space, Escape, Tab exit, outside-click dismissal, and focus restoration to the originating More button. Explorer activation is not followed by forced DropCove focus. The Edge Rail and its pointer-opened management popup remain no-activate and do not claim keyboard/live-announcement ownership.

Only one item menu is open per surface. It closes on action, dismissal, another menu, projection replacement, target disappearance/unmount, scroll, drag/resize/movement, modal or busy ownership, surface hide/close, and Rail collapse/fullscreen/display transitions. The popover stays within its surface without resizing it. Rail menu lifetime uses a dedicated transient hold released on dismissal; leaving the Rail closes the menu and permits normal delayed collapse.

Managed File internal paths are not shown or copied by item actions, tooltips, accessibility labels, status, or announcements. Managed Text uses its user-visible name and content preview; Managed Image uses its name and `DropCove-created image`. External References retain their full user-facing path.

In Manage Items, secondary-button input never opens the Item Actions Menu; More remains its sole entry point. A secondary-click on the row selection region follows the Selection Actions Menu contract in section 9.5. Secondary-clicks on independent row controls, inputs, and textareas do not retarget Item Selection or open Selection Actions, and the WebView default browser menu remains suppressed on Shelf Item row boundaries.

### 9.5 Selection Actions Menu

The **Selection Actions Menu** applies lifecycle actions to the current Batch Selection on the Drop Shelf or expanded Edge Rail and to the current Item Selection in Manage Items. It remains distinct from the ownership-aware Item Actions Menu and never contains Show in File Explorer, Copy Path, or Copy Text.

- Secondary-clicking a selected card or row preserves the current selection and targets the complete selection. Secondary-clicking an unselected card or row replaces selection with that member before opening the menu. Secondary-button `Ctrl`, `Shift`, and `Ctrl+Shift` modifiers do not extend, toggle, or range-select.
- Only the card or row selection region opens the menu. Pin, More, Manage, Remove, confirmation controls, inputs, and textareas remain isolated and do not retarget selection.
- Pointer invocation anchors the menu at the invocation point; keyboard invocation anchors it to the focused card or row. The popover remains within its owning surface without resizing it.
- At most one Selection Actions Menu is open per surface. Opening it closes an Item Actions Menu and vice versa. Projection replacement, target disappearance or unmount, scroll, drag, resize, movement, confirmation, busy mutation, surface hide/close, Rail collapse, fullscreen transition, or display transition closes it.

| Selection domain and cardinality | Lifecycle action | Removal action |
|---|---|---|
| One selected single-item Shelf Batch | Pin Item or Unpin Item | Remove Batch |
| One selected multi-item Shelf Batch | Pin All Items or Unpin All Items | Remove Batch |
| Two or more selected Shelf Batches | Pin Selected or Unpin Selected | Remove Selected |
| One selected Manage Items row | Pin Item or Unpin Item | Remove Item |
| Two or more selected Manage Items rows | Pin Selected or Unpin Selected | Remove Selected |

Cardinality follows the selection domain, not affected Shelf Item count. One selected multi-item Shelf Batch is one Batch Selection member. All-Pinned targets offer Unpin; None or Mixed targets offer Pin. Batch Selection Pin/Unpin includes every Shelf Item in every selected batch, while Item Selection Pin/Unpin includes exactly the selected Shelf Item identities in the active batch.

Pin and Unpin validate and mutate the complete target atomically, retain selection on success, and preserve selection plus prior Pin states on validation or persistence failure. Singleton Remove remains immediate. Remove Selected follows the confirmation and ownership-safety contract in section 13.4.

On the Drop Shelf and Drop-Shelf-origin Manage Items, the Context Menu key and `Shift+F10` open the menu for the focused card or row. Opening focuses the first item; Arrow Up/Down, Home/End, Enter/Space, `Escape`, and Tab follow the established menu contract. Keyboard invocation on an unselected focused member selects only that member first. Dismissal restores focus to the invoking member when it still exists.

The Edge Rail and Rail-origin Manage Items remain pointer-only/no-activate. Their menu uses transient ownership to prevent collapse without activating DropCove, claiming keyboard focus, or adding a live-announcement owner.


## 10. Manage Items

Manage Items provides item-level control for a multi-item Shelf Batch.

### 10.1 Placement and size

- Width: `240` logical pixels.
- Maximum height: `480` logical pixels.
- Placement preference: right, left, below, then above the originating card, with a preferred `8`-logical-pixel gap along the placement axis.
- A preferred candidate is eligible only when the complete popup and gap fit inside the monitor work area with the preferred `16`-pixel margin.
- Fallback: when no preferred candidate fits, clamp inside the permitted work area; containment takes precedence and the gap may shrink, disappear, or become unavoidable overlap.
- The same separation and fallback contract applies when Manage Items is opened from the Drop Shelf or expanded Edge Rail.
- Only one Manage Items popup is open at a time.

The same `240`-logical-pixel management popup opens from the Edge Rail without inheriting the Rail's `280`-pixel width. Its vertical scroller remains bounded.

### 10.2 Content and actions

Header:

- title and total item count;
- `X/N pinned` summary, prefixed by `S selected ·` while Item Selection is non-empty;
- compact tri-state Bulk Pinning icon control;
- compact Close icon control.

Each `54`-logical-pixel row, separated by a `5`-pixel gap, exposes:

- icon or thumbnail;
- item name on the first text line;
- availability on the second text line as `Available`, `Unavailable`, or `Missing`, without a duplicate `Pinned ·` prefix;
- External Reference full path through the drag-region tooltip and accessibility text, but not as visible row content; Managed Files use ownership-appropriate name/content descriptions without Profile Data paths;
- compact Pin/Unpin icon action;
- compact More icon action when eligible under section 9.4;
- compact Remove Item icon action;
- Item Selection-aware direct drag-out from the row drag region.

Each header or row action group uses a single trailing icon-control row.

### 10.3 Item Selection

Item Selection is transient presentation state owned by the current Manage Items popup. It stores stable Shelf Item identities plus an optional selection anchor and focused identity for the popup's one active Shelf Batch. It survives virtual-row mount churn, is never persisted, and is discarded rather than transferred when the popup closes or another batch replaces it.

- Plain click selects one Shelf Item and sets the anchor. `Ctrl+click` toggles one item and sets the anchor even when toggling it off.
- `Shift+click` replaces selection with the inclusive source-order range from the anchor to the target, including virtualized items outside the viewport. Shift preserves a valid anchor; without one it selects only the target and establishes that item as anchor. `Ctrl+Shift` uses the same replacement-range behavior.
- Plain pointer press on an already selected row delays collapse until release below the established drag threshold. Crossing the threshold on a selected row drags the complete Item Selection; crossing it on an unselected row drags only that Shelf Item without first replacing the visible selection.
- Selected rows use the existing accent border and accent-subtle surface, with Windows Highlight resources in Contrast Themes. The header summary reports `S selected · X/N pinned`. No toolbar, checkbox, selection mode, or per-row count badge is added.
- Independent per-item Pin/Unpin and Remove controls continue to affect only their own Shelf Item. Bulk Pinning continues to affect the complete Shelf Batch. The Selection Actions Menu defined in section 9.5 explicitly targets Item Selection and does not change the scope of those existing controls.
- Independent and selection-scoped Pinning retain selection. Remove, Remove Selected, and confirmed-Missing cleanup prune only identities that disappear; surviving selected items remain selected, and a removed anchor becomes absent without selecting a replacement. Newly added items are not selected.
- Rejected or canceled drag-out retains Item Selection unless a separate clear trigger occurs. Successful Drag-Out, empty collection-space click, `Escape`, explicit popup close, opening another batch, popup closure caused by its owner, or restart clears it.
- The popup remains open while its Shelf Batch exists, including when only one Shelf Item remains. It closes when the batch disappears.

### 10.4 Lifecycle and keyboard behavior

- Pointer opening does not activate the popup or force keyboard focus into it, including when opened from the no-activate Edge Rail.
- Keyboard opening from the Drop Shelf with `Enter` or `Space` focuses the current item drag region in the collection grid.
- The item collection exposes `role="grid"` with `aria-multiselectable="true"`; rows expose `aria-selected`, one item drag region is the roving focus target, and Pin/Unpin and Remove remain independent buttons.
- Up/Down moves item focus, Home/End moves to collection boundaries, `Space` selects one, `Ctrl+Space` toggles, `Shift+Space` replaces with the anchor range, and `Ctrl+A` selects every item without changing a valid anchor or creating an absent one.
- `Tab` and `Shift+Tab` cycle through Bulk Pinning, Close, the grid's primary drag-region stop, then item Pin/Unpin, eligible More, and Remove controls in source order. Virtualized targets are mounted and scrolled into view before focus moves.
- If the focused item disappears while the batch survives, focus moves to the item now at the same source index, otherwise the previous item, otherwise Bulk Pinning. Selection changes use the existing polite announcement channel with coalesced selected-item counts or `Selection cleared`.
- Explicit close restores focus to the originating Manage Items control when it still exists. If the batch has become a one-item card, focus returns to that card's drag region. If the batch disappeared, the Drop Shelf recovers to the batch now at the same displayed index, otherwise the previous batch, otherwise More. A pointer-opened Edge Rail origin does not receive focus.
- `Esc` closes an open Selection Actions Menu, then cancels an open selection-removal confirmation, then cancels an active item availability confirmation or native item drag. Later presses clear non-empty Item Selection and close Manage Items.
- On a focus-capable Drop-Shelf origin, the Context Menu key and `Shift+F10` open Selection Actions for the focused row and return focus there after dismissal when it survives. A pointer-opened Edge Rail origin remains pointer-only/no-activate and adds no keyboard menu path.
- Opening another batch replaces the existing popup. The popup closes when its batch is removed, the shelf hides, shelf resizing begins, or whole-batch or Batch Selection drag starts.
- Item Selection drag keeps Manage Items open while its batch remains. A hide or close request during an active native drag becomes pending; the recorded transfer completes independently, then the popup closes without restoring discarded selection.

## 11. Edge Rail

The Edge Rail appears when a non-empty Drop Shelf is dismissed.

### 11.1 Placement

- Dock edge comes from `RailEdge` in settings; the default is `Right`.
- Target monitor `Follow pointer` is represented by an empty `RailMonitorId`. On every fresh non-empty Drop Shelf dismissal, DropCove selects the monitor containing the pointer. If pointer-monitor resolution fails, it falls back to the Drop Shelf monitor, the latest persisted dismissal monitor, then the Windows primary display.
- A non-empty `RailMonitorId` is an explicit target monitor. When connected, it takes precedence over the pointer. When unavailable, DropCove preserves the setting and temporarily applies the same pointer-first fallback; a docked Edge Rail returns to the explicit monitor when it reconnects.
- Every successful non-empty Drop Shelf dismissal stores the actual selected monitor. Fullscreen restore, Shake cancellation, expansion/collapse, content updates, and switching Settings to `Follow pointer` reuse the current docked placement rather than resampling the pointer. Selecting a connected fixed monitor moves a docked Edge Rail immediately; if a Follow pointer monitor disconnects, the Rail moves to the available pointer monitor and retains that actual fallback placement.
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
| Maximum expanded height             |      642 logical px |
| Hover or drag-entry expansion delay |              200 ms |
| Pointer/drag leave collapse delay   |              300 ms |

For a positive batch count, target expanded height is:

```text
max(280, min(642, 52 × batch count + 70))
```

The expanded collection retains 48-pixel rows, 4-pixel gaps, symmetric 6-pixel horizontal insets, an 8-pixel fixed top boundary outside the actual scrollport, and 8 pixels of bottom clearance inside the scroll range. Its rounded 4-pixel painted scrollbar thumb starts at the content-side edge of the 8-pixel hit channel, making the rightmost-card-to-visible-thumb clearance equal to the 6-pixel left clearance. Existing scrollbar colors/states are retained; Manage Items and Settings thumb alignment is unchanged. Short collections remain top-aligned; the scrollbar appears only on overflow. Ordinary and virtualized collections share this geometry, and the final row remains reachable at maximum scroll.

Behavior:

- one through four batches remain at 280 px height;
- five through eleven batches grow through bounded tiers;
- twelve or more batches use 642 px and vertical scrolling;
- a supported incoming drag keeps the existing 200 ms drag-entry expansion delay; after expansion, one 48-pixel Drop Placeholder appears before the newest retained Shelf Batch;
- the placeholder uses a dashed drag-hover border and distinct background without applying a surface-wide Rail highlight;
- drag hover does not temporarily change Rail height; an accepted new Shelf Batch updates height from the retained batch count through the existing Rail formula;
- new batches can grow an already expanded rail;
- removal does not live-shrink the expanded rail; the next expansion recalculates it;
- removing the final Shelf Item hides the rail immediately;
- an open management flyout or Clear confirmation keeps the rail expanded.

### 11.3 Rail actions

The expanded Edge Rail:

- omits a retained-batch header title;
- supports inspection, single-batch drag-out, and pointer Batch Selection drag-out of compact Shelf Batch rows;
- supports drag of one item from Manage Items;
- accepts supported filesystem, website-image, and plain-text input through drop-in;
- supports Pin/Unpin, Bulk Pinning, Remove Item, and Remove Batch;
- shows a bottom-left Clear Temporary Items trash icon only while Temporary Items exist;
- shows an inward Open Shelf arrow at top-right when docked Left and top-left when docked Right.

Both expanded-Rail corner actions retain `24 × 24`-logical-pixel targets and `6`-pixel horizontal insets. Open Shelf is offset `4` pixels from the top edge within the existing `28`-pixel header budget; Clear Temporary Items is offset `4` pixels from the bottom edge while retaining the existing footer budget. These visual offsets mirror across Left and Right docking and do not change the height formula or expansion tiers.

The expanded Rail footer also shows the compact Batch Selection count when selection is non-empty. This label shares the existing footer budget without changing the height formula, corner-action targets, no-activate behavior, or expansion tiers.

The collapsed Rail Handle remains the full Open Shelf activation target and retains its vertical label.

### 11.4 Fullscreen policy

**Show over fullscreen applications** is disabled by default.

When disabled, the requested rail hides while the selected monitor is fully covered by the foreground window and returns when that condition ends. When enabled, the rail remains visible over fullscreen windows.

When fullscreen hiding begins, DropCove closes Edge Rail Manage Items, discards any unconfirmed Clear Temporary Items prompt, and clears Edge Rail Batch Selection. When fullscreen coverage ends, only the collapsed Rail Handle returns. An already-running native drag is not canceled by this presentation change; its prepared participants complete independently and do not restore selection after rejection or cancellation.

## 12. Drop-in behavior

DropCove accepts one representation family from each Windows OLE data object, in this priority order:

1. existing filesystem storage items with non-empty paths;
2. Website Image Virtual Files;
3. non-empty Unicode plain text.

The selected family is the richest supported representation for that drop. Representations are never combined: an image drag that also exposes URL or text data creates only image Shelf Items. If a higher-priority offered family is invalid, DropCove rejects it rather than silently falling back to a lower-priority representation. URL-only, HTML-only, rich-text, bitmap-only, and generic virtual-file payloads are unsupported. One drag operation creates at most one Shelf Batch.

### 12.1 Existing filesystem storage items

Processing order:

1. discard entries without usable paths;
2. remove repeated path strings within the payload using ordinal case-insensitive comparison;
3. preserve the first occurrence and source order;
4. compare the resulting path set with every retained Shelf Batch;
5. add one new Shelf Batch only when no exact retained match exists.

An incoming filesystem batch is duplicate when its item count matches a retained batch and every incoming path exists in that batch under ordinal case-insensitive comparison; item order is ignored. DropCove does not canonicalize path strings or rewrite separators. Exact duplicate batches are silent no-ops, partial overlap is accepted, and the same path may occur in different Shelf Batches. External filesystem drops remain format-agnostic; the website-image format rules do not restrict ordinary files or folders.

### 12.2 Website image virtual files

DropCove accepts `CFSTR_FILEDESCRIPTORW` plus indexed `CFSTR_FILECONTENTS` image data without issuing its own HTTP request. Source application identity is not required. Current stable Edge and Chrome on Windows are the release acceptance matrix; compatible payloads from other applications are best-effort only.

Supported formats are PNG, JPEG, GIF, WebP, AVIF, SVG, BMP, and ICO. DropCove validates content instead of trusting the extension alone. It preserves a sanitized basename, replaces the extension with one matching the validated content, and generates `Dropped image YYYY-MM-DD HH-mm-ss[-N].ext` when no safe basename remains.

Raster validation uses bounded parsers and must not require more than `512 MiB` of decoded allocation for one entry. A malformed entry, an unsupported parser path, or a decoder that cannot stay within that bound is skipped. Raw SVG bytes are preserved with an `.svg` name after secure well-formed XML validation with DTD and entity expansion disabled. DropCove does not remove scripts or external references from SVG, but it never renders SVG in a WebView; Shell Visual may use a generic icon fallback.

DropCove examines only the first `20` descriptors in source order. Each image has a `100 MiB` encoded-byte limit and the drop has a `500 MiB` accepted encoded-byte limit. An entry that exceeds its individual limit or would make the accepted total exceed the drop limit is skipped; a later descriptor among the first `20` may still be accepted when it fits. Content for descriptors after the first `20` is not opened.

Each accepted image becomes a Managed File. Unsupported, invalid, or over-limit image entries are skipped and counted; if at least one valid image remains, all accepted images form one Shelf Batch in descriptor order. Multiple entries may retain the same sanitized source filename because Managed File identity isolates their backing paths; only generated fallback names receive a numeric suffix. A partial success reports one non-modal **Added N images; skipped M.** status. A fully rejected image family reports one non-modal categorized warning for invalid or over-limit input, Attachment Services, I/O, or persistence failure. An I/O, Windows Attachment Services, or persistence failure rejects the complete drop and removes staged files rather than committing a partial batch.

Every staged website image passes through mandatory Windows Attachment Services using non-UI `IAttachmentExecute::Save`; `0.0.24` provides no bypass, allowlist, or retry-without-scan path. Only `http` or `https` URLs obtained from `CFSTR_INETURLW` or `CFSTR_INETURLA` supply the security zone; DropCove never derives provenance from companion text, HTML, filenames, or image content. When no accepted source URL exists, the Restricted Zone default applies. A trust-service failure or requirement for user interaction rejects the complete drop without opening a security prompt. Successful `Save` is authoritative even when Windows policy or the profile filesystem does not create `Zone.Identifier`; DropCove preserves zone metadata whenever the filesystem provides it.

After Attachment Services returns, DropCove revalidates the final file. Final supported bytes that remain within all limits are authoritative even when a trust service changed the content or image type: DropCove preserves those exact bytes, animation, and metadata, and repairs the extension to match the final type. A missing, invalid, unsupported, or over-limit final file rejects the complete drop; pre-scan bytes are never retained.

`0.0.24` adds no intake progress or cancellation surface. Native extraction is bounded by the descriptor and byte limits, and each surface reports one aggregate result when processing finishes.

### 12.3 Plain text

DropCove accepts non-empty `CF_UNICODETEXT`, encodes it as UTF-8, and creates one `.txt` Managed File in a one-item Shelf Batch. The maximum encoded size is `10 MiB`. The filename uses local time in the form `Dropped text YYYY-MM-DD HH-mm-ss.txt`, adding a numeric suffix when necessary.

HTML and RTF formatting are discarded rather than preserved. A URL received as plain text remains literal text; DropCove does not open or download it. Plain-text intake does not apply website attachment-zone evidence.

### 12.4 Duplicate and mixed-input results

Managed Files are not content-hashed, shared, or deduplicated by bytes. Each accepted text or image drop represents a new user action and creates a new Shelf Batch even when its content matches a retained Managed File.

Within the selected representation family, unsupported entries are skipped and reported. If none remain, no batch is created. Duplicate-only filesystem input, rejected higher-priority input, and unsupported input do not persist, resize, reorder, scroll, or alter Shake restoration.

## 13. Temporary and Pinned Item behavior

### 13.1 Temporary Item

Every newly accepted Shelf Item is Temporary by default.

A Temporary Item remains until one of these events:

- Successful Drag-Out;
- Remove Item;
- Remove Batch;
- Clear Temporary Items;
- confirmed-Missing cleanup before batch or Batch Selection drag.

Temporary Items survive application and Windows restarts. A retained Temporary Item backed by a Managed File keeps that file in Profile Data until the lifecycle ends.

### 13.2 Pinned Item

A Pinned Item:

- displays a pin indicator;
- remains after Successful Drag-Out;
- can be returned to Temporary through Unpin or Bulk Unpin;
- can still be removed explicitly through Remove Item or Remove Batch;
- retains its Managed File across restart, update, and uninstall while the Shelf Item remains.

### 13.3 Bulk Pinning

For multi-item batches:

| Item states   | Control state | Activation result         |
| ------------- | ------------- | ------------------------- |
| All Temporary | Off           | Pin every item            |
| All Pinned    | On            | Make every item Temporary |
| Mixed         | Indeterminate | Pin every item            |

Bulk Pinning includes Available, Missing, and Unavailable references still present in the batch. It changes lifecycle state only; it does not change path availability, ordering, source files, or shelf sizing.

Selection Actions reuse this tri-state rule. An all-Pinned target becomes Temporary; a None or Mixed target becomes Pinned. Multi-target Pin/Unpin validates the complete Batch Selection or Item Selection identity set and commits one transaction; partial lifecycle changes are not permitted.

### 13.4 Removal actions

- **Remove Item** removes one Shelf Item and prunes its batch when empty.
- **Remove Batch** removes every Shelf Item in that Shelf Batch.
- **Remove Selected** on Batch Selection removes every selected Shelf Batch and every contained Shelf Item.
- **Remove Selected** on Item Selection removes exactly the selected Shelf Items from the active Manage Items batch and prunes the batch when empty.
- **Clear Temporary Items** requires confirmation, removes Temporary Items across all batches, and preserves Pinned Items.
- Removing an external reference never deletes, moves, renames, or overwrites its source filesystem object.
- Removing a Managed File-backed item permanently deletes the app-owned file rather than sending it to the Recycle Bin.

Reference removal and a required Pending Managed File Deletion record commit atomically. DropCove attempts deletion immediately; a sharing, antivirus, or other filesystem failure leaves the item removed, reports how many app-owned files remain pending, and retries through event-driven cleanup and later startup. Best-effort deletion without durable retry is not permitted.

If persistence fails, the visible and durable Shelf state remain unchanged and no Managed File becomes unowned.

Singleton Remove Item and Remove Batch remain immediate. Removing two or more Batch Selection or Item Selection members always requires confirmation, including all-External selections. Batch confirmation reports `N batches · M items`; Manage Items confirmation reports `N items`.

Confirmation copy follows authoritative ownership counts:

- all External References: source files and folders will not be deleted;
- mixed External References and Managed Files: External sources stay and the exact number of DropCove-created files to be permanently deleted is reported;
- all Managed Files: the exact number of DropCove-created files to be permanently deleted is reported.

The confirmation target is an immutable authoritative identity snapshot with selection-unit, item, and ownership counts. The caller does not supply trusted counts or ownership. Confirm revalidates the complete target and rejects stale, missing, duplicate, wrong-domain, or cross-batch identities without partial mutation. Reference removal and required Pending Managed File Deletion records commit in one transaction; filesystem deletion failure retains durable retry responsibility without restoring removed Shelf Items.

While confirmation is open, the originating surface blocks selection changes, drag start, Pin/Unpin, Remove, Clear Temporary Items, accepted drop-in mutation, movement, resize, and competing Shelf mutations. Success prunes removed identities from selection; Cancel, `Escape`, setup failure, stale-target failure, or persistence failure retains selection.

Drop Shelf and Drop-Shelf-origin Manage Items use focus-capable modal confirmation on their owning surface with **Cancel** focused initially and focus restoration to the invoking member when it survives. Edge Rail and Rail-origin Manage Items use inline pointer confirmation plus transient hold without foreground activation or keyboard focus. Fullscreen, owner destruction, Cooperative Shutdown, and Exit retain higher teardown authority and cancel an unconfirmed transient without mutation.

The Drop Shelf Clear confirmation opens in its own decorated, fixed-size `360 × 130`-logical-pixel, taskbar-hidden, always-on-top window centered within the Drop Shelf monitor work area. Its heading is **Clear temporary items?**, its explanation is **Pinned items stay. External sources stay. DropCove-created temporary files are permanently deleted.**, and its actions are **Cancel** and **Clear**. Cancel is focused initially; `Escape` and the window close action cancel without mutation.

While this confirmation is open, the Drop Shelf cannot be dismissed, moved, resized, dragged from, or used for competing mutations or utility actions. Repeated requests focus the existing confirmation. Confirmation rechecks authoritative Temporary Item availability and invokes the existing atomic Clear operation; stale empty requests make no removal. Exit DropCove and Cooperative Shutdown retain normal teardown authority.

The Edge Rail keeps its local confirmation, transient hold, and fullscreen-cancellation lifecycle, uses the same explanation, and does not open the dedicated Drop Shelf confirmation.

## 14. Drag-out behavior

DropCove supports:

- dragging one Shelf Item;
- dragging every eligible item in one Shelf Batch;
- dragging eligible items across one Batch Selection.

### 14.1 Copy-only contract

Drag-out offers Windows `Copy` only. It never offers `Move`.

Completion rules:

| Destination result        | Temporary Item | Pinned Item |
| ------------------------- | -------------- | ----------- |
| Accepted `Copy`           | Removed        | Retained    |
| Rejected                  | Retained       | Retained    |
| User canceled             | Retained       | Retained    |
| Failure before acceptance | Retained       | Retained    |

For a Batch Selection, DropCove resolves selected Shelf Batch identities in displayed newest-first order and preserves source order inside each batch; click order never controls payload order. Exact paths repeated across selected batches are emitted once using ordinal case-insensitive comparison, while same filenames at different paths remain distinct. The preparation token records every selected Available occurrence represented by an emitted path. Accepted `Copy` consumes every represented Temporary occurrence and retains every represented Pinned occurrence. No content hashing or shared Managed File backing participates in this deduplication.

For Item Selection, DropCove resolves the selected Shelf Item identities in source order inside the active Shelf Batch; click order never controls payload order. Dragging an unselected row supplies a singleton item set without replacing the visible selection. Exact paths are deduplicated with the same ordinal case-insensitive rule while every represented Available occurrence remains recorded for exact Temporary/Pinned completion. When the prepared payload represents exactly one Available plain-text Managed File, it retains the existing Unicode text-only OLE representation and does not advertise file-drop storage formats. Multi-item payloads expose only file/folder storage paths and never concatenate text content.

### 14.2 Availability preparation

Every card drag uses serialized batch-set preparation; an unselected card supplies a singleton batch set. Every Manage Items row drag uses serialized item-occurrence preparation; a selected row supplies the complete Item Selection and an unselected row supplies a singleton item set. Batch and item identity sets remain distinct contracts, then share the same occurrence classification, deduplicated transfer, completion, and cleanup rules.

Before drag-out, preparation:

1. binds the token to the exact ordered selected batch or item identities and authoritative Shelf snapshot;
2. validates that item identities are unique and belong to the active Manage Items batch;
3. reclassifies every represented Shelf Item occurrence;
4. durably removes confirmed Missing Items, records required Managed File cleanup, and prunes empty batches;
5. retains Unavailable Items;
6. records every Available occurrence while producing ordinal case-insensitively deduplicated outbound paths;
7. reports Available and Unavailable counts as Shelf Item occurrences;
8. shows one aggregate confirmation when both Available and Unavailable occurrences remain;
9. cancels with status feedback when no Available occurrence remains.

Dragging the confirmation primary action starts that exact prepared operation immediately. Clicking it may arm a later drag from the same selected card or row only while the selected identifiers and Shelf snapshot remain unchanged; any selection or Shelf mutation cancels the old token and requires preparation again. `Cancel` or `Escape` cancels the token but retains selection.

Missing cleanup is not rolled back if the later confirmation or drag is canceled or rejected. Rejected, canceled, or pre-acceptance-failed drag-out retains all recorded Available participants and the current Batch Selection or Item Selection only when no hide, close, surface-switch, restart, or other selection-clear trigger occurred.

### 14.3 Acceptance limitation

Destination acceptance is the only completion signal available to DropCove. It cannot atomically coordinate its local database transaction with another application's later processing.

If a destination accepts `Copy` and DropCove terminates before its removal is persisted, a Temporary Item may reappear after restart. Retaining an extra reference is preferred over incorrectly losing one. After persisted consumption, a Temporary Managed File enters durable permanent cleanup even though DropCove cannot prove the destination's later internal processing.

## 15. Missing and Unavailable Items

| Classification | Detection                                                                               | Behavior                                                |
| -------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Available      | Filesystem attributes can be read                                                       | Eligible for item, batch, or Batch Selection drag-out. |
| Missing        | File/directory is absent while its root remains reachable                               | Removed during batch or Batch Selection preparation.   |
| Unavailable    | Root, volume, network, cloud, permission, or another error prevents a reliable decision | Retained and excluded from that drag.                   |

Availability is refreshed at restore and drag boundaries. DropCove does not continuously watch every path for changes.

## 16. Settings

Settings is one decorated, fixed-size `420 × 520`-logical-pixel, non-modal, non-topmost top-level window with a Windows taskbar button. It is not owned by the hidden Drop Shelf. The Resident Instance owns its draft lifecycle and closes it during Exit DropCove or Cooperative Shutdown.

A new Settings window is centered within the invoking monitor's work area: the Drop Shelf monitor for a Drop Shelf Utility Menu request, or the pointer monitor for a notification-area request, with Windows primary-monitor fallback. Placement measures the complete decorated outer frame in physical pixels before showing the window. A repeated request restores, shows, and focuses the singleton without moving it or replacing its draft. Closing Settings permits a later request to recreate and recenter it; user-moved positions are not persisted.

Opening Settings from the notification area does not show, move, or change the Drop Shelf or Edge Rail. Hiding the Drop Shelf does not hide Settings.

Settings presents two horizontal tabs below its title: **General** and **Appearance**. General contains every setting introduced before `0.0.19`; Appearance contains the application-wide appearance selector. A newly created or reopened Settings window starts on General, tab selection is not persisted, switching tabs retains the shared draft, and one Save/Cancel footer governs both tabs.

### 16.1 Available settings and defaults

| Setting                           | Default                | Values / behavior                                                   | Introduced |
| --------------------------------- | ---------------------- | ------------------------------------------------------------------- | ---------- |
| Global hotkey                     | `Ctrl + Shift + Space` | At least one modifier plus a supported primary key                  | `0.0.4`    |
| Start with Windows                | On                     | Adds/removes current-user sign-in registration                      | `0.0.4`    |
| Docking edge                      | Right                  | Left or Right                                                       | `0.0.5`    |
| Target monitor                    | Follow pointer         | Follow the pointer on fresh dismissal, or use a connected explicit display identifier | `0.0.5`    |
| Show over fullscreen applications | Off                    | Controls Edge Rail fullscreen hiding                                | `0.0.5`    |
| Enable shake gesture              | On                     | Installs/removes global pointer hook                                | `0.0.6`    |
| Shake sensitivity                 | Normal                 | Low, Normal, High                                                   | `0.0.6`    |
| Appearance mode                   | System                 | System, Dark, or Light; Windows Contrast Theme overrides rendering  | `0.0.19`   |

There is no setting to disable the Edge Rail while retaining its standard non-empty dismissal behavior.

Settings only exposes controls whose owning runtime capability has shipped. General contains the existing resident, Rail, and Shake controls; Appearance contains only the application-wide Appearance Mode selector. Persisted contract extensions use a clean cutover when their owning capability becomes active.

### 16.2 Appearance mode

Appearance presents `System`, `Dark`, and `Light` as one three-option radio group. The options are compact horizontal preview tiles with Monitor, Moon, and Sun icons, a visible selected state, standard Tab entry/exit, and arrow-key movement within the group.

- `System` follows the live Windows application Light or Dark theme and is the default.
- `Dark` retains Dark presentation regardless of ordinary Windows application-theme changes.
- `Light` retains Light presentation regardless of ordinary Windows application-theme changes.
- Windows Contrast Theme always overrides the rendered WebView palette and native-frame appearance without changing the selected or persisted Appearance Mode.
- While Contrast Theme is active, the selector remains operable and reports that Windows Contrast Theme currently overrides the chosen appearance.
- Appearance Mode applies to Drop Shelf, Edge Rail, Manage Items, Settings, About, confirmation surfaces, menus, and native frames, including surfaces that become visible after the selection changes.

DropCove does not expose custom accents, user-authored palettes, per-surface modes, or scheduled theme switching.

### 16.3 Save and cancel behavior

- Settings changes remain drafts until **Save** succeeds.
- Selecting an Appearance Mode previews it immediately across every current DropCove WebView surface and native frame. A surface that becomes visible while Settings remains open uses the same preview.
- **Save** atomically commits the complete Settings candidate, retains the preview as the active appearance, and persists `AppearanceMode` with the other settings.
- **Cancel**, native Settings close, or a failed Save discards the Appearance draft and restores every surface and native frame to the previously persisted mode. Switching tabs does not discard the shared draft.
- The current registered hotkey is retained while a new hotkey is tested; startup registration and atomic JSON persistence are then applied.
- Only integrations owned by the active version participate. Edge Rail, Shake, and Appearance integration are included with their owning versions.
- A failure attempts to restore the previous complete configuration. If complete rollback is impossible, DropCove reports the retained safe state through a warning.

### 16.4 Persisted settings contract

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

Serialized content in `0.0.19`:

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
    "ShakeSensitivity": "Normal",
    "AppearanceMode": "System"
}
```

Each schema extension (`0.0.5`, `0.0.6`, and `0.0.19`) is a clean cutover: all top-level fields for the active version are required and unknown fields remain rejected. Earlier-version settings files without the newer fields follow the normal malformed-settings path: preserve the original bytes, load active defaults in memory, warn, and replace the file only after a successful explicit Save.

For the `0.0.19` cutover, a pre-feature file without `AppearanceMode` therefore resets every in-memory preference to the documented defaults, including the hotkey, startup, Rail, fullscreen, Shake, and `AppearanceMode: System` values. DropCove does not migrate that file, add compatibility fields, or create a second appearance store.

`RailEdge` and `RailMonitorId` are user preferences. An empty `RailMonitorId` selects Follow pointer for each fresh non-empty dismissal; a non-empty value takes precedence whenever that display is connected. The actual monitor used by the docked Edge Rail remains Shelf state for restoration and does not rewrite the preference.

The numeric virtual-key value is stable across keyboard layouts. Its displayed punctuation label follows the current Windows keyboard layout.

Missing settings load defaults without warning. Malformed or unreadable settings remain untouched, load defaults in memory, and produce a warning notification. A successful explicit Save replaces the file atomically through a temporary file.

## 17. Persistence and recovery

DropCove restores Temporary and Pinned Items after application or Windows restart.

Persisted shelf data includes:

- Shelf Batch identity, creation time, and order;
- Shelf Item identity, order, path, name, file/folder type, backing ownership, and pinned state;
- Pending Managed File Deletion paths;
- remembered Edge Rail dismissal monitor;
- preferred shelf width and optional Manual Height Override while content exists.

Managed File bytes live under `%LOCALAPPDATA%\DropCove\managed\`, isolated by Shelf Item identity while preserving the user-visible sanitized filename. They are Profile Data and survive restart, update, and uninstall while retained or pending deletion.

Not persisted in `shelf.db`:

- external source file contents;
- Managed File bytes themselves;
- icon or thumbnail bytes;
- current availability classification;
- Batch Selection or other transient popup/flyout state;
- automatic height as manual intent.

Pending Managed File Deletions are retried at startup and after cleanup-producing mutations without a continuous polling loop. A successful deletion removes its pending record; a missing target is treated as already deleted.

If the shelf database is corrupt or cannot be opened through a recoverable storage/format failure, DropCove:

1. preserves the failed database, matching WAL/SHM sidecars, and the complete managed storage together in one timestamped Recovery Backup;
2. creates a clean database and empty active managed-storage root;
3. starts with an empty shelf;
4. warns the user and provides the backup path.

DropCove does not automatically salvage the preserved state. If complete preservation or clean recreation fails, startup remains fatal rather than deleting or partially reusing uncertain Managed Files.

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

## 19. Accessibility, appearance, and motion

- Action controls expose accessible names and use inline SVG icons that inherit the current semantic text or system color.
- Icon actions expose native tooltips, hover treatment, and visible `focus-visible` indicators.
- Binary Pin controls expose `aria-pressed`; tri-state Bulk Pinning exposes `role="checkbox"` with `aria-checked="false"`, `"true"`, or `"mixed"`.
- Keyboard users can open and close Manage Items, operate its multiselectable item grid, traverse independent actions, activate buttons with standard Enter/Space behavior, and operate the Appearance radio group.
- The Manage Items collection uses `role="grid"` with `aria-multiselectable="true"`; selected item rows expose `aria-selected`, item drag regions use roving focus with Up/Down/Home/End navigation, and `Space`, `Ctrl+Space`, `Shift+Space`, `Ctrl+A`, and `Escape` operate the Item Selection contract in section 10.3. Pin/Unpin and Remove remain independently named buttons.
- Item Selection count changes are announced politely and coalesced. Pointer opening, including from the Edge Rail, does not activate or focus the popup; keyboard opening from the Drop Shelf focuses the grid.
- The Drop Shelf batch collection uses `role="grid"` with `aria-multiselectable="true"`; selected batch rows expose `aria-selected`, batch drag regions use roving focus with arrow/Home/End navigation, and `Space`, `Ctrl+Space`, `Shift+Space`, `Ctrl+A`, and `Escape` operate the Batch Selection contract in section 9.3.
- Batch Selection count changes are announced politely and coalesced on the Drop Shelf. The Edge Rail remains pointer-only/no-activate and exposes only the visible selected treatment and footer count.
- Selection Actions Menu uses menu/menuitem semantics and ownership-safe accessible names. Focus-capable Drop Shelf and Manage Items origins support the Context Menu key and `Shift+F10`, standard menu navigation, `Escape` dismissal, Tab exit, and focus restoration. Edge Rail and Rail-origin Manage Items remain pointer-only/no-activate.
- Remove Selected confirmation exposes exact selection-unit/item scope and ownership consequences. Focus-capable confirmation is modal with Cancel initially focused; pointer-only Rail confirmation does not claim focus or a live region. Selection Actions use existing semantic menu, danger, focus, and Windows system-color resources.
- External Reference full paths and drag instructions remain available through tooltips and accessibility text when visual text is shortened or omitted. Managed Files never expose Profile Data paths there; use user-visible names, text previews, or `DropCove-created image` instead.
- The resolved Appearance Mode drives Light or Dark semantic resources and matching native frames. Windows Contrast Theme bypasses the application palette and uses Windows system colors, explicit borders, and visible focus indicators.
- Drop Placeholders use the drag-hover semantic colors in Dark and Light appearances and Windows system colors in Contrast Themes.
- Entrance, insertion, removal, and rail transitions are short opacity/scale effects.
- Non-essential motion is skipped when Windows disables client-area animations.
- Drag-and-drop and state changes remain functional when animations are disabled.

### 19.1 Semantic appearance palette

| Semantic role | Dark | Light |
|---|---:|---:|
| Application background | `#000000` | `#F3F3F3` |
| Shelf Batch card | `#101010` | `#F8F8F8` |
| Raised control or menu | `#181818` | `#FFFFFF` |
| Hover surface | `#232323` | `#EAEAEA` |
| Primary text | `#F5F5F5` | `#111111` |
| Secondary text | `#C7C7C7` | `#3F3F3F` |
| Muted text | `#929292` | `#686868` |
| Default border | `#383838` | `#D0D0D0` |
| Interactive border | `#5A5A5A` | `#858585` |
| Primary action | `#2B63EB` | `#2B63EB` |
| Focus and accent text | `#7AA2FF` | `#2B63EB` |

Primary actions use `#2B63EB` with white text. Dark accent text and icons use `#7AA2FF`; Light accent text and icons use `#2B63EB`. Default and interactive borders are resting and hover resources, not substitutes for the dedicated focus indicator. Shelf Batch cards have no shadow in Dark appearance and only a slight one-pixel separation shadow in Light appearance; larger shadows are reserved for menus, dialogs, and other floating surfaces. Ordinary icons use the secondary text color, selected and action icons use the applicable cobalt, and destructive icons use the danger semantic color. Warning and danger retain distinct contrast-checked amber and red roles rather than inheriting cobalt. Contrast Themes ignore this table and use Windows system colors.

## 20. Current limitations and deliberate exclusions

- Ordinary external files and folders must expose filesystem paths; they remain references rather than imported copies.
- Non-path intake is limited to validated website-image virtual files and Unicode plain text under the documented formats and size limits.
- No HTTP URL fallback, URL items, HTML/RTF items, generic virtual-file intake, or content-addressed Managed File deduplication.
- Drag-out is Copy-only.
- Cross-integrity drag-and-drop is not guaranteed.
- Batch Selection is surface-local and session-only; it is not persisted or shared between the Drop Shelf and Edge Rail.
- Additive `Ctrl+Shift` ranges, named selections, selection mode UI, persistent or cross-batch Item Selection, and arbitrary item selection outside the current Manage Items batch are not supported.
- Edge Rail supports only Left and Right docking and remains pointer-only/no-activate.
- No automatic updater.
- No clipboard history, accounts, cloud sync, plugins, named shelves, file synchronization, or arbitrary application-owned copies of ordinary filesystem drops.
- No custom accent color, user-authored palette, independent per-surface appearance, or scheduled theme switching.

## 21. End-to-end example

1. The user drags `A.png` and `B.png` from one folder into the Drop Shelf; DropCove creates one external-reference Shelf Batch.
2. The user separately drags `C.png` from another folder; DropCove creates a second batch without copying any source file.
3. The user drags `logo.svg` from Edge; DropCove validates the virtual file, applies Windows attachment security, and creates a Managed File-backed batch.
4. The user drags plain text; DropCove creates a timestamped UTF-8 `.txt` Managed File in another batch.
5. The user pins `logo.svg`, selects the two external batches and the text batch, and performs one Successful Drag-Out.
6. The selected external Temporary references are consumed without deleting their source files. The text reference is consumed and its Managed File is permanently deleted or recorded for retry.
7. The pinned `logo.svg` and its Managed File remain after drag-out and restart.
8. The user dismisses the non-empty Drop Shelf; the Edge Rail appears and supports the same pointer Batch Selection workflow without taking foreground focus.
