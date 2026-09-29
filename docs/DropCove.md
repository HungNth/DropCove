# DropCove

> **A temporary drag-and-drop space for Windows.**

## 1. Overview

**DropCove** is a native Windows productivity application that provides a temporary drag-and-drop workspace for holding files and folders between workflows.

The product is inspired by:

- **Holdem** — temporary file holding and drag-and-drop workflow.
- **Edge-Drop** — edge docking, compact interaction, and polished animations.

DropCove is **not** intended to be a full file manager or clipboard manager.

Its core purpose is:

> Let users temporarily hold files and folders, preserve the context of each drop operation, and quickly drag those items into another application when needed.

---

## 2. Product Identity

### Official naming

| Item | Name |
|---|---|
| Product | `DropCove` |
| Repository | `dropcove` |
| Executable | `DropCove.exe` |
| Solution | `DropCove.sln` |
| Root namespace | `DropCove` |

### Internal terminology

```text
DropCove
└── Product name

Drop Shelf
└── Main floating surface used to hold and manage items

Edge Rail
└── Vertical docked version of the shelf at the screen edge

ShelfBatch
└── A group created from one drop operation

ShelfItem
└── A file or folder inside a batch
```

---

## 3. Core Product Principles

DropCove should be:

- Windows-native.
- Lightweight.
- Event-driven.
- Fast to summon.
- Easy to dismiss.
- Optimized for drag-and-drop.
- Minimal while idle.
- Capable of rich, smooth animation.
- Compatible with native Windows applications.
- Designed around temporary references rather than file duplication.

The application should avoid unnecessary background activity.

No continuous cursor polling should be used when event-driven Windows APIs are available.

---

# 4. Technology Stack

## Language

```text
C#
```

## Runtime

```text
.NET 10
```

## UI

```text
WinUI 3
Windows App SDK
XAML
```

## Animation

```text
Microsoft.UI.Composition
```

Composition should be used for:

- shelf appearance and dismissal;
- slide-in / slide-out;
- edge rail expansion;
- opacity;
- scale;
- translation;
- hover effects;
- compact-to-expanded transitions;
- item insertion/removal animations.

## Application architecture

```text
MVVM
CommunityToolkit.Mvvm
```

## Native Windows integration

```text
CsWin32
P/Invoke
Win32
WinRT
OLE
```

Native integration is expected for:

- HWND handling;
- global hotkeys;
- low-level mouse events;
- shake detection;
- always-on-top behavior;
- no-activate window behavior;
- monitor detection;
- DPI awareness;
- native drag-and-drop;
- native file data objects.

---

# 5. Core Interaction Model

DropCove has three primary ways to access the shelf.

## 5.1 Global hotkey

A global shortcut opens and activates the Drop Shelf immediately.

Default:

```text
Ctrl + Shift + Space
```

The shortcut is configurable. If registration fails because another application owns the combination, DropCove remains running, reports the conflict through the tray, and asks the user to choose another shortcut.

The hotkey is intended for quickly opening the shelf even when no drag operation is currently active.

---

## 5.2 Shake-to-open

When a user is already dragging files or folders, a shake gesture can summon the Drop Shelf close to the current cursor position.

Example:

```text
Explorer
   ↓
drag files
   ↓
shake cursor
   ↓
Drop Shelf appears near cursor
   ↓
drop files
```

Shake-to-open is enabled by default and can be disabled in Settings.

Windows does not provide a documented global event that identifies another application's OLE file drag before a DropCove target receives `DragEnter`. While shake is enabled, DropCove therefore keeps a low-level mouse hook installed:

```text
WH_MOUSE_LL
```

The hook observes pointer input and detects quick direction reversals as a heuristic. It cannot prove that the user is dragging files, so false positives are handled by cooldown and state restoration.

The hook callback must perform minimal work and queue analysis elsewhere. DropCove must not poll cursor coordinates continuously.

When the configured threshold is reached:

```text
ShowShelfNearCursor()
```

---

## 5.3 Edge Rail

If DropCove still contains items when the main shelf is dismissed, the shelf docks to the configured left or right screen edge.

The Edge Rail remains directly interactive.

Users can:

- inspect batches;
- hover to reveal details;
- drag items out;
- drag entire batches out;
- drag new files into DropCove;
- open the full Drop Shelf.

---

# 6. Shelf Data Model

DropCove stores items by **drop operation**, not as one flat list.

Each drop creates a `ShelfBatch`.

Paths are deduplicated within that drop operation. The same path dropped again later creates a new `ShelfItem` in a new batch, preserving the context of each operation.

Example:

```text
Drop #1
├── A.png
├── B.png
└── C.png

Drop #2
└── logo.svg

Drop #3
├── docs.pdf
└── assets/
```

This preserves an important piece of context:

> Which files were added together?

---

# 7. ShelfBatch

A batch represents one successful drop operation.

Suggested model:

```csharp
public sealed class ShelfBatch
{
    public Guid Id { get; init; }

    public DateTime CreatedAt { get; init; }

    public ObservableCollection<ShelfItem> Items { get; init; }
}
```

A new batch is created every time a user drops one or more files/folders into DropCove.

Example:

```text
First drop:
A.png
B.png
C.png
```

creates:

```text
Batch #1
├── A.png
├── B.png
└── C.png
```

Then dropping:

```text
logo.svg
```

creates:

```text
Batch #2
└── logo.svg
```

The files should not be flattened into one undifferentiated list.

---

# 8. ShelfItem

A `ShelfItem` represents one file or folder.

Suggested model:

```csharp
public sealed class ShelfItem
{
    public Guid Id { get; init; }

    public string Path { get; init; }

    public string DisplayName { get; init; }

    public ShelfItemType Type { get; init; }

    public bool IsPinned { get; set; }
}
```

Item type:

```csharp
public enum ShelfItemType
{
    File,
    Folder
}
```

---

# 9. Temporary Items

Every newly added item is temporary by default.

Lifecycle:

```text
drop into DropCove
        ↓
temporary item
        ↓
drag out
        ↓
successful drop
        ↓
remove from DropCove
```

A temporary item is removed only when the destination accepts its drag-out operation with the `Copy` effect. Cancellation, rejection, or failure keeps the reference in DropCove.

DropCove cannot atomically commit its local state together with another application's drop. If the destination accepts a drop and DropCove crashes before persisting the removal, the temporary reference may reappear after restart. V1 accepts this at-least-once behavior because an extra reference is safer than incorrectly losing one.

---

# 10. Pinned Items

Users can pin files or folders for repeated use.

Pinned items should display a clear pin indicator.

Example:

```text
📌 logo.svg
   screenshot.png
📌 brand-guide.pdf
```

Behavior:

```text
Pinned item
    ↓
drag out
    ↓
successful drop
    ↓
keep item in DropCove
```

Clicking the pin action again converts the item back to temporary.

```text
Pinned
  ↓
Unpin
  ↓
Temporary
```

Pinned items are suitable for frequently reused assets such as:

- logos;
- design assets;
- templates;
- reference files;
- project icons;
- frequently reused documents.

---

# 11. Batch Lifecycle

Example:

```text
Batch #1
├── A.png
├── B.png
└── C.png
```

If `A.png` is pinned:

```text
Batch #1
├── 📌 A.png
├── B.png
└── C.png
```

After `B.png` is successfully dragged out:

```text
Batch #1
├── 📌 A.png
└── C.png
```

After `C.png` is successfully dragged out:

```text
Batch #1
└── 📌 A.png
```

The batch remains because it still contains an item.

If a batch contains only temporary items and all are consumed:

```text
Batch.Items.Count == 0
```

then the batch should be removed automatically.

---

# 12. Display State

Final V1 display-state model:

```csharp
public enum ShelfDisplayState
{
    Hidden,
    EdgeDocked,
    Compact,
    Expanded
}
```

The enum above is the final V1 target. Stage 1 does not introduce `Compact` or `Expanded`; while the native workflow is being proven, the visible state uses one bounded unified Drop Shelf.

## Stage 1 bounded unified Drop Shelf

The pre-mode Drop Shelf uses one vertical, newest-first Shelf Batch list. It is not `Compact` mode and has no separate management presentation.

Exact logical-pixel window sizes:

| Shelf Batches | Width | Height |
| ---: | ---: | ---: |
| 0–1 | 180 | 180 |
| 2 | 180 | 236 |
| 3 | 180 | 292 |
| 4 or more | 180 | 348 |

The window uses a 32-pixel integrated drag strip with Settings and Close controls. Its top edge remains anchored while it grows downward; placement shifts upward only when required to remain inside the current monitor work area. Resizing is immediate in Stage 1.

Each Shelf Batch is one vertical summary card. Single-item batches show the native icon and truncated name; multi-item batches show a bounded stacked visual and item count. The whole content surface accepts drops: an empty shelf shows a drop prompt, while a populated shelf shows a drag-over overlay without reserving a permanent drop panel.

Four recent batches are visible. Additional batches remain in the same window through vertical scrolling, with a `+N batches` indicator. A newly accepted batch appears first and returns the list to the top.

## Hidden

Used whenever the floating shelf is not displayed. Before Edge Rail is implemented, `Hidden` may still contain items; the hotkey reopens the shelf without losing them.

## EdgeDocked

Used after Edge Rail is implemented when the shelf is dismissed but still contains items.

## Compact — Ticket 15 and later

The final V1 quick-access presentation introduced by Ticket 15 after the native workflow is proven.

## Expanded — Ticket 15 and later

The final V1 management presentation used to inspect and manage all content.

---

# 13. Placement State

Placement should be modeled separately from display state.

```csharp
public enum ShelfPlacement
{
    Docked,
    Default,
    Cursor
}
```

Before Ticket 15, the bounded unified Drop Shelf combines with placement as follows:

| Invocation | Presentation | Placement |
| --- | --- | --- |
| Shake | Bounded unified Drop Shelf | Cursor |
| Global hotkey | Bounded unified Drop Shelf | Default |
| Edge interaction | Bounded unified Drop Shelf | Docked |

Ticket 15 later replaces that single presentation with `Compact` and `Expanded`. Placement remains independent so display and monitor-position behavior do not become one oversized state model.

---

# 14. Dismissing the Drop Shelf

The hotkey toggles the shelf. `Esc` and the `×` button also dismiss it; losing focus does not.

Before Edge Rail is implemented, dismissing always hides the shelf while preserving its content:

```text
Bounded unified Drop Shelf → Hidden
```

After Edge Rail is implemented:

```text
Items.Count == 0 → Hidden
Items.Count > 0  → EdgeDocked
```

The remaining items may be temporary, pinned, or a mixture of both.

---

# 15. Compact Drop Shelf — Ticket 15 and later

Ticket 15 introduces `Compact` as a distinct quick-access presentation only after the bounded unified Drop Shelf and native workflow are proven. It consumes the same shelf state as Expanded mode and does not change batch, drag, or persistence semantics.

Compact stays small and focuses on recent batches:

```text
╭────────────────────────────────────╮
│ 🗂3     🖼     🗂5     🗂2     ⤢  │
╰────────────────────────────────────╯
```

Each visual entry represents a `ShelfBatch`. The target is 4–6 recent batches. When more batches exist, Compact exposes overflow without growing indefinitely:

```text
🗂3  🖼  🗂5  🗂2  +4  ⤢
```

This post-Ticket-15 horizontal presentation is separate from the Stage 1 vertical, stepped-size bounded unified Drop Shelf.

---

# 16. Batch Visuals

## Single-item batch

Example:

```text
╭──────────╮
│    🖼    │
│ logo.svg │
╰──────────╯
```

## Multi-item batch

Multi-item batches should use a stacked visual.

Example:

```text
╭──────────╮
│ ╭────╮   │
│╭────╮│   │
││ +4 ││   │
│╰────╯╯   │
╰──────────╯
```

The stacked treatment communicates that the visual represents one grouped drop operation rather than one file.

---

# 17. Expanded Drop Shelf

Expanded mode is the primary management view.

Example:

```text
╭────────────────────────────────────────╮
│ DropCove                          ─  × │
├────────────────────────────────────────┤
│                                        │
│ Drop #1                                │
│ [ A.png ] [ B.png ] [ C.png 📌 ]      │
│                                        │
│ Drop #2                                │
│ [ logo.svg 📌 ]                        │
│                                        │
│ Drop #3                                │
│ [ X ] [ Y ] [ Z ] [ assets/ ]         │
│                                        │
╰────────────────────────────────────────╯
```

Expanded mode should support:

- viewing all batches;
- viewing all items;
- pinning/unpinning;
- removing items;
- clearing temporary items;
- multi-selection;
- dragging individual items;
- displaying paths;
- displaying file metadata;
- dealing with larger numbers of files.

---

# 18. Edge Rail

The Edge Rail is not just a launcher.

It is a vertically oriented compact shelf that remains directly usable.

Example:

```text
                            screen edge
                                 │

                             ╭────────╮
                             │ 🖼 📌  │
                             ├────────┤
                             │ 🗂 3   │
                             ├────────┤
                             │ 📄     │
                             ├────────┤
                             │ 🗂 5   │
                             ╰────────╯
```

Each cell represents one batch.

The rail is an overlay, not a Windows AppBar, and does not reserve desktop work area. It is hidden over fullscreen applications by default; Settings can allow it to remain visible. Fullscreen detection must be event-driven.

---

# 19. Collapsed Edge Rail

When idle, the rail should remain narrow.

Suggested width:

```text
48–64 px
```

Example:

```text
╭──────╮
│ 🖼📌 │
├──────┤
│ 🗂3  │
├──────┤
│ 📄   │
├──────┤
│ 🗂5  │
╰──────╯
```

The collapsed rail should:

- consume little screen space;
- remain visually identifiable;
- show file/batch type;
- show pinned status where relevant;
- allow direct drag initiation.

---

# 20. Interactive Edge Rail

When the pointer remains inside the Edge Rail for `200 ms`, it expands horizontally without activating DropCove. Leaving collapses it after `300 ms`; opening the full shelf is the action that activates DropCove.

Example transition:

```text
48–64 px
    ↓ hover for 200 ms
160–220 px
```

Example:

```text
╭──────────────────╮
│ 🖼 logo.svg 📌   │
├──────────────────┤
│ 🗂 3 files       │
├──────────────────┤
│ 📄 report.pdf    │
├──────────────────┤
│ 🗂 5 files       │
╰──────────────────╯
```

This remains a compact interaction mode.

It is not yet the full Expanded Drop Shelf.

---

# 21. Dragging From the Edge Rail

Users should be able to drag content directly from the rail.

Example:

```text
Edge Rail
    │
    │ drag logo.svg
    ▼
Photoshop
```

### Pinned item

```text
drag
 ↓
successful drop
 ↓
keep item
```

### Temporary item

```text
drag
 ↓
successful drop
 ↓
remove item
```

This allows experienced users to perform most common actions without opening the full shelf.

---

# 22. Dragging an Entire Batch

Dragging a multi-item batch should initiate one native drag operation containing all items in that batch.

Example:

```text
🗂 4 files
      ↓
drag
      ↓
Explorer / another compatible application
```

Windows OLE should provide the underlying multi-file drag behavior.

---

# 23. Inspecting a Batch From the Edge Rail

If the user wants one specific file from a batch, hovering or clicking the batch should open a small flyout.

Example:

```text
╭───────────────────╮
│ 🗂 4 files        │
├───────────────────┤
│ image-1.png       │
│ image-2.png 📌    │
│ notes.pdf         │
│ assets/           │
╰───────────────────╯
```

The user can then drag an individual item from the flyout.

---

# 24. Vertical Scrolling

The Edge Rail must not grow indefinitely.

Recommended maximum rail height:

```text
~50–70% of monitor height
```

If additional batches exist:

```text
vertical scroll
```

Mouse wheel interaction should work naturally while the pointer is over the rail.

Any scrollbar should be visually subtle and preferably only visible while interacting.

---

# 25. Dragging New Files Into the Edge Rail

The Edge Rail should also work as a drop target.

Example:

```text
file ─────────► Edge Rail
```

When a drag enters the rail and remains there for `200 ms`, the rail expands without activating DropCove:

```text
DragEnter
   ↓ 200 ms
still dragging?
   ↓
expand
```

If the pointer leaves before the delay expires, expansion is canceled. After `DragLeave`, the rail collapses after `300 ms` unless a batch flyout remains open.

---

# 26. Three UI Levels

DropCove has three main UI levels.

## Level 1 — Edge Rail

```text
48–64 px
```

Purpose:

```text
quick access
```

## Level 2 — Interactive Edge Rail

```text
160–220 px
```

Purpose:

```text
quick drag-and-drop
```

## Level 3 — Expanded Drop Shelf

```text
~500–700 px
```

Purpose:

```text
management
```

Typical progression:

```text
Edge Rail
    ↓ hover

Interactive Edge Rail
    ↓ expand

Expanded Drop Shelf
```

---

# 27. Shake State Restoration

Shake invocation should temporarily override the current placement.

Example:

```text
EdgeDocked
   ↓
drag + shake
   ↓
Compact near cursor
```

If the user cancels the drag without dropping anything:

```text
Compact near cursor
   ↓
restore previous state
   ↓
EdgeDocked
```

Likewise:

```text
Hidden
 ↓
drag + shake
 ↓
Compact near cursor
 ↓
cancel
 ↓
Hidden
```

The shake workflow does not permanently alter the shelf state unless a supported drop is accepted. Cancellation, an unsupported payload, or a false-positive shake restores the previous state. After an accepted drop, the shelf remains near the cursor so the user can inspect the new batch; dismissing it follows the normal Hidden/EdgeDocked rules.

---

# 28. Storage Behavior

DropCove stores path references and never copies source files into application storage. It must not move, rename, overwrite, or delete the referenced filesystem objects.

Supported references include local drives, removable drives, UNC/network locations, and cloud-backed files when the source supplies a filesystem path.

```text
D:\Videos\large-video.mov
```

A reference identifies whatever filesystem entry currently exists at that path. If the original file is deleted and another file later appears at the same path, the reference resolves to the replacement.

Benefits:

- instant insertion;
- minimal disk usage;
- no duplicated large files;
- behavior matches the temporary-workspace concept.

---

# 29. Missing and Unavailable Files

External deletion, movement, disconnected volumes, unavailable networks, cloud state, or permission changes must not crash the application.

- **Missing**: the path is confirmed not to exist.
- **Unavailable**: the path may still exist but cannot currently be accessed.

Validation occurs when references are restored, when the shelf is shown, and immediately before drag-out. V1 does not require a realtime filesystem watcher.

Starting a drag removes Missing references from DropCove before the native drag begins; this never deletes a filesystem object. If the batch becomes empty, the empty batch is removed. Cancellation does not restore those invalid references.

Unavailable references are retained. When a batch also contains available items, DropCove warns the user and requires confirmation before dragging only the available items.

---

# 30. Persistence

DropCove restores both temporary and pinned shelf references after application or Windows restart. Persistence is part of the first usable vertical slice.

Shelf metadata is stored in a per-user SQLite database. Each completed state change is committed transactionally, including drop-in, pin/unpin, manual removal, Missing cleanup, and successful drag-out.

Persisted data includes batch order and creation time plus item identity, path, type, and pinned state. Source file contents and thumbnail bytes are not stored by DropCove.

If the database cannot be opened or is corrupt, DropCove preserves it as a timestamped backup, informs the user, and creates a new database rather than overwriting the failed file.

---

# 31. Drag-In Architecture

V1 accepts files and folders that provide filesystem paths. Virtual items that exist only as streams are not materialized into DropCove storage.

One accepted drop creates one `ShelfBatch`. Paths are deduplicated within that operation while retaining source order. If a payload mixes supported paths and unsupported items, DropCove accepts the supported paths and reports how many items were skipped; a payload with no supported paths is rejected.

Incoming data should be converted immediately into the application domain model.

---

# 32. Drag-Out Architecture

Drag-out uses native OLE file transfer so DropCove can interoperate with third-party Windows applications:

```text
UI
 ↓
DragDropService
 ↓
IDataObject / DoDragDrop
 ↓
Windows
```

V1 offers the `Copy` effect only. DropCove never moves or deletes the source filesystem object.

A successful drag-out is the destination accepting the operation with a non-empty `Copy` result. It removes participating temporary references and retains participating pinned references. Cancellation, rejection, or failure leaves references unchanged.

V1 supports dragging one item or an entire batch. Arbitrary multi-selection across batches is deferred. A mixed pinned/temporary batch remains as a smaller batch containing its pinned items after a successful drop.

DropCove runs unelevated and guarantees drag/drop only between applications at the same integrity level. Dragging to or from an elevated application may fail because of Windows security boundaries; failed operations retain temporary references.

---

# 33. DragDropService

OLE-specific behavior belongs in `DragDropService`, not in views or view models. The initial service is concrete; add an interface only when a second implementation or a real replaceable boundary exists.

---

# 34. Window Behavior

DropCove is a single-instance, always-available utility.

- It starts with Windows by default but can be disabled in Settings.
- Auto-start restores state while keeping the UI hidden and unfocused.
- A second launch redirects activation to the existing instance.
- The global hotkey activates the floating shelf for keyboard access.
- Edge Rail hover and direct drag do not activate DropCove; opening the full shelf does.
- The shelf remains topmost while visible.
- The `×` button dismisses the shelf; **Exit DropCove** in the tray terminates the process.
- The shelf does not automatically hide merely because another application receives focus.

---

# 35. Multi-Monitor Support

DropCove must be DPI-aware and work correctly across multiple monitors.

- **Hotkey**: show at the default work-area position on the monitor containing the cursor.
- **Shake**: show near the cursor on its current monitor.
- **Edge Rail**: dock to the left or right edge of the monitor where the shelf was dismissed, then remember that monitor and side.

The default edge is Right. Settings allow Left/Right selection and target-monitor selection. The rail remains on its chosen monitor rather than following the cursor.

---

# 36. Edge Selection

V1 should support:

```text
Left
Right
```

Top and bottom docking are intentionally out of scope for V1.

Settings:

```text
Dock position

○ Left
◉ Right
```

---

# 37. Animation Guidelines

## Shelf appearing

```text
opacity: 0 → 1
scale:   0.9 → 1
```

## Edge Rail expansion

Use translation and size animation.

## Compact → Expanded

Use:

- translation;
- scale;
- opacity.

## Item removal

Use:

```text
opacity ↓
scale ↓
layout collapse
```

## New batch insertion

Use:

```text
slide + fade
```

Animations should feel responsive rather than decorative or slow.

Animations are added only after the associated behavior is stable and must respect Windows reduced-motion and animation settings.

---

# 38. Event-Driven Design

Avoid polling loops such as:

```csharp
while (true)
{
    GetCursorPos(...);
    Thread.Sleep(...);
}
```

Prefer Windows events:

```text
Global hotkey event
Mouse hook event
Drag event
Pointer event
Window event
```

Release-build acceptance targets:

```text
Idle CPU average                    ≤ 0.1%
Periodic cursor polling             0
Show shelf from resident process    p95 ≤ 150 ms
Idle working set target             < 150 MB
```

Thumbnail loading and SQLite I/O must not block the UI thread. V1 should remain responsive with at least 100 batches and 1,000 items by using lazy thumbnail loading and UI virtualization. The shelf realizes only visible item elements; native icons are requested independently, and image thumbnails are discarded when their visible realization ends.

---

# 39. Core Architecture

```text
                ┌─────────────────────────┐
                │        WinUI 3          │
                │                         │
                │ Drop Shelf              │
                │ Edge Rail               │
                │ Batch Flyout            │
                │ Settings                │
                └────────────┬────────────┘
                             │
                       ViewModels
                             │
                ┌────────────▼────────────┐
                │    DropShelfManager     │
                │                         │
                │ ShelfBatch[]            │
                │ DisplayState            │
                │ Placement               │
                └────────────┬────────────┘
                             │
          ┌──────────────────┼───────────────────┐
          │                  │                   │
          ▼                  ▼                   ▼
 DragDropService       HotkeyService       WindowService
          │                  │                   │
          ▼                  ▼                   ▼
       OLE/Win32         RegisterHotKey      HWND / Win32
          │
          ▼
       Windows
```

---

# 40. Solution Structure

```text
DropCove.sln

src/
│
├── DropCove.App/
│   ├── App.xaml
│   └── App.xaml.cs
│
├── DropCove.Core/
│   ├── Shelf/
│   │   ├── ShelfItem.cs
│   │   ├── ShelfBatch.cs
│   │   ├── ShelfDisplayState.cs
│   │   ├── ShelfPlacement.cs
│   │   └── DropShelfManager.cs
│   │
│   └── Settings/
│
├── DropCove.Services/
│   ├── DragDropService.cs
│   ├── HotkeyService.cs
│   ├── ShakeDetector.cs
│   ├── MouseHookService.cs
│   ├── WindowService.cs
│   ├── MonitorService.cs
│   ├── ThumbnailService.cs
│   ├── PersistenceService.cs
│   └── SettingsService.cs
│
└── DropCove.Native/
    ├── Ole/
    ├── Win32/
    └── WindowInterop/
```

UI-specific components inside `DropCove.App` may be organized as:

```text
UI/
├── Shelf/
│   ├── ShelfWindow.xaml
│   ├── CompactShelf.xaml
│   └── ExpandedShelf.xaml
│
├── EdgeRail/
│   ├── EdgeRail.xaml
│   └── BatchFlyout.xaml
│
├── Items/
│   ├── ShelfItemView.xaml
│   └── ShelfBatchView.xaml
│
└── Settings/
```

This four-project split is the selected starting structure. `CommunityToolkit.Mvvm` is used for observable UI state and commands; domain types and services remain concrete unless a real replaceable boundary appears.

---

# 41. Core Domain Structure

```text
DropShelfManager
│
├── ObservableCollection<ShelfBatch>
│
├── DisplayState
│    ├── Hidden
│    ├── EdgeDocked
│    ├── Compact
│    └── Expanded
│
└── Placement
     ├── Docked
     ├── Default
     └── Cursor
```

Batch:

```text
ShelfBatch
│
├── Id
├── CreatedAt
└── Items[]
```

Item:

```text
ShelfItem
│
├── Id
├── Path
├── DisplayName
├── Type
└── IsPinned
```

---

# 42. Delivery Sequence — V1.0

## Stage 1 — Native vertical slice

- Unpackaged self-contained floating application installed by a CLI-built EXE, with one unified shelf.
- Single-instance activation, tray lifecycle, configurable global hotkey, and Start with Windows.
- Hotkey placement uses the default work-area position on the monitor containing the cursor.
- Floating shelf placement and sizing must be correct for the current monitor's DPI.
- Drag files/folders in from Explorer and create ordered batches.
- Drag one item or one whole batch out through native OLE.
- Temporary and pinned item lifecycle.
- Remove Item, Remove Batch, and Clear Temporary Items; bulk removal requires confirmation.
- SQLite persistence and restore.
- Missing/Unavailable handling.
- Windows file/folder icons through the Windows Shell and Windows Shell thumbnails for images, loaded lazily from visible visual elements with icon fallback.
- Minimal Settings for hotkey and startup.

## Stage 2 — Edge Rail

- Left/right overlay docking on the remembered monitor.
- Collapsed and interactive rail, direct drag-in/out, batch flyout, and vertical scrolling.
- No-activate hover/direct drag behavior and fullscreen auto-hide.
- The rail opens the existing unified shelf.

## Stage 3 — Shake-to-open

- Default-enabled low-level mouse hook with configurable sensitivity.
- Cursor placement, cooldown, false-positive recovery, and previous-state restoration.

## Stage 4 — Final V1 UI and polish

- Distinct Compact and Expanded shelf modes.
- Multi-file stacked visuals and management view.
- Composition animations added after behavior is stable and respecting reduced motion.
- Full Edge Rail multi-monitor polish, accessibility, and installed-EXE release verification.

## Compatibility gate

Before V1.0, verify drag-in from Explorer/Desktop and drag-out to Explorer, Edge/Chrome file inputs, VS Code, and at least one common chat application. Folder drag-out must work with Explorer. Elevated cross-integrity drag/drop is not supported.

## Verification strategy

Automated tests cover shelf state transitions and SQLite persistence. WinUI, OLE interoperability, global hotkeys, window activation, DPI, hooks, and cross-application behavior are verified with actual smoke scenarios against the installed EXE release build rather than mocks alone.

---

# 43. Explicitly Out of Scope for V1

Do not add these features until the core native drag-and-drop workflow is stable:

```text
Clipboard history
Text snippets
URL manager
Cloud sync
Accounts
Plugin system
Multiple named shelves
File synchronization
File-copy storage
AI features
Network sharing
```

---

# 44. Future Roadmap

## V1.1

Potential additions:

- URL items;
- text items;
- clipboard images;
- clipboard quick-drop.

## V1.2

Potential additions:

- multiple shelves;
- named shelves;
- pinned workspaces;
- search;
- filtering;
- optional pinned-item prioritization.

## V2+

Possible future directions:

- project-specific shelves;
- custom shelf profiles;
- plugins;
- cross-device workflows;
- optional cloud synchronization.

---

# 45. Example User Workflow

The user drags three files from Explorer:

```text
Explorer
   │
   │ drag A B C
   │
   ├── shake
   │
   ▼

        DropCove
     ╭───────────────╮
     │   DROP HERE   │
     ╰───────────────╯
            │
           drop
            │
            ▼

     ╭───────────────╮
     │     🗂 3      │
     ╰───────────────╯
```

The user later adds `logo.svg`:

```text
╭─────────────────────╮
│ 🗂 3    🖼 logo.svg │
╰─────────────────────╯
```

The user pins it:

```text
🖼 logo.svg 📌
```

Temporary files are dragged out:

```text
A.png
  ↓
Photoshop
  ↓
successful drop
  ↓
remove A.png from DropCove
```

The user dismisses the shelf while `logo.svg` still exists:

```text
remaining item
      ↓
EdgeDocked
```

The Edge Rail remains visible:

```text
                         ╭──────╮
                         │ 🖼📌 │
                         ╰──────╯
```

Hover expands it:

```text
                    ╭─────────────────╮
                    │ 🖼 logo.svg 📌  │
                    ╰─────────────────╯
```

The user can drag `logo.svg` directly into another application.

Because it is pinned:

```text
logo.svg remains in DropCove
```

---

# 46. Product Definition

DropCove should be positioned as:

> **A native Windows drag-and-drop workspace for temporarily holding and reusing files across workflows.**

Short version:

> **A temporary drag-and-drop space for Windows.**

The core interaction remains:

```text
Drag
 ↓
Hold
 ↓
Continue working
 ↓
Drag out
```

Pinned items extend that model:

```text
Keep frequently reused files close at hand.
```

---

# 47. Final Technical Direction

```text
C#
.NET 10
WinUI 3
Windows App SDK

CommunityToolkit.Mvvm
CsWin32

XAML
Microsoft.UI.Composition

Win32
WinRT
OLE
IDataObject
DoDragDrop

WH_MOUSE_LL
RegisterHotKey

SQLite persistence
Self-contained x64 EXE installer (initial)
MSIX/Microsoft Store submission (later)
```

Initial release baseline:

```text
Supported Windows 11 releases
x64 first
Unpackaged self-contained WinUI application
CLI-built EXE installer
No Visual Studio, MSVC Build Tools, standalone MSBuild, or Visual Studio XAML tooling
No telemetry, background update checker, or application-owned network traffic
```

The initial installer is for a working application before signing and Microsoft Store submission. It must include the complete self-contained publish output and account for the x64 Visual C++ Redistributable prerequisite. Microsoft Store/MSIX submission is deferred until Partner Center supplies the package name and publisher.

Responsibility split:

```text
WinUI 3
→ UI

Microsoft.UI.Composition
→ animation

C#
→ application and domain logic

WinRT
→ standard Windows APIs

Win32 / OLE
→ low-level native integration

MVVM
→ separation of UI and application state
```

This architecture gives DropCove a modern Windows-native UI while retaining direct control over the native drag-and-drop, windowing, hotkey, edge-docking, and mouse interactions that define the product.
