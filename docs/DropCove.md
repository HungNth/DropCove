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
- popup appearance and dismissal;
- item insertion/removal animations.

DropCove reads the Windows client-area animation preference before each transition. When Windows disables animation effects, non-essential Composition transitions are skipped while state changes, focus, and drag behavior remain available.

Monitor placement uses PerMonitorV2-aware physical bounds. A remembered monitor is preferred when connected; if it is removed, the shelf and Edge Rail resolve to the current connected monitor and retry the remembered display when it returns. Display, device, and DPI changes trigger an event-driven reflow rather than background polling.


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

Configure it in **Settings → Global hotkey**:

1. Click the shortcut field, or focus it with Tab and press Enter or Space. Merely focusing the field does not start recording; the Enter/Space activation press is not recorded.
2. Hold at least one of Ctrl, Alt, Shift, or Win and press a supported primary key. The first valid primary-key press completes recording and displays the draft in `Ctrl + Alt + Shift + Win + key` order.
3. Select **Save** to register and persist the draft. Until Save succeeds, the previously saved shortcut remains active. DropCove suppresses its own shelf toggle only while recording, including the press that completes recording.

Supported primary keys are A–Z, 0–9, Space, Tab, Enter, Backspace, Delete, Insert, arrow keys, Home/End/Page Up/Page Down, F1–F24, numpad digits/arithmetic, and standard punctuation/OEM keys. While recording, modifier-plus-Tab/Enter/Space is captured before normal focus or button behavior. Bare Tab leaves the field and cancels recording. Escape cancels only the recording attempt; Backspace and Delete with a modifier are normal shortcut keys, not clear commands. Lock keys, Print Screen, Pause, media/volume, browser/launch, IME, mouse buttons, and unknown keys are not supported.

During recording, Save is disabled. Incomplete or unsupported input leaves the field listening and shows inline guidance. Escape, moving focus to another control, or switching to another window restores the prior draft. There is no recording timeout. Cancel or closing Settings discards all unsaved settings.

Left/right modifiers are combined; AltGr uses Windows' Ctrl+Alt representation. Punctuation labels follow the current Windows keyboard layout for display only. Settings continue storing the same numeric virtual-key value, so a layout change can change the displayed symbol without rewriting the shortcut.

Win is supported when Windows delivers the combination to Settings, but Windows-reserved combinations may run an operating-system action or take focus first. DropCove installs no keyboard hook to override them. If Windows takes focus, recording is canceled and the prior draft restored.

If Windows rejects registration, Settings retains the draft and reports that the hotkey may be unavailable or reserved by Windows; the previous shortcut and saved settings remain in place. A later save failure uses the existing rollback to restore the previous registration and settings. On startup, an unavailable shortcut produces a tray warning without stopping the resident process; use tray Settings to choose another shortcut. This recorder adds no clear, disable, or reset action.

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

### Compact Adaptive Edge Rail

The Edge Rail uses a plain visible `16 × 280` logical-pixel Rail Handle. Hover or drag entry expands it after `200 ms`; leaving collapses it after `300 ms` unless a flyout or Clear confirmation remains open. Direct rail interaction does not activate the Drop Shelf.

Expanded width is `280` logical pixels. Each Shelf Batch occupies one horizontal `64`-pixel row with `4`-pixel inter-row spacing. Target height is `max(280, min(640, 68 × batch count + 68))`: one through three batches use `280`, four through eight use `340/408/476/544/612`, and nine or more use `640` with vertical scrolling. Expansion never vertically shrinks the window below the 280-pixel handle height. New batches grow the expanded rail; removal holds its current height until the next expansion. Removing the final item hides the rail immediately.

Both states remain centered on the selected monitor work area and clamp actual bounds after DPI scaling. Open Shelf sits at top-left for a right rail and top-right for a left rail. Clear Temporary Items sits at bottom-left, asks for confirmation, removes temporary references across the shelf, and preserves pinned references and source files.

Only the two desktop-facing corners round (8-pixel radius aligned with Drop Shelf); the screen-edge corners remain square. A subtle 1-pixel border using `CardStrokeColorDefaultBrush` outlines the three desktop-facing edges (0-pixel on the docked screen edge). The Edge Rail window surface is transparent (DWM blur-behind with an empty region, a transparent `SystemBackdrop`, and a black `WM_ERASEBKGND` clear), so the XAML `CornerRadius` renders antialiased against any desktop color, including white. An opaque theme base (`RailBaseBrush`: black for Dark, white for Light, `SystemColorWindowColor` for High Contrast) sits under the translucent `LayerFillColorDefaultBrush`, preserving the previous on-screen color. A DPI-scaled native window region (`SetWindowRgn`, 5-pixel radius) still shapes the input silhouette: the empty corner gaps do not hit-test as rail, but a band about 1 pixel wide just outside the visual 8-pixel arc does. That band lets the region sit inside the antialiased fringe so its 1-bit edge never clips it. This does not change Drop Shelf DWM rounding.

**Qualification status:** Ticket 01 and 02 behaviors are verified. Release smoke at 96 DPI observes both edges, Rail Handle (16×280), expanded tiers (280-wide, 280–640), 8 complete 64px rows fit at 612px via virtualized ListView, selective GDI corner clipping (8px inward, square at screen edge), Open Shelf activation, Clear Temporary Items (cancel, accept, pinned retention, all-temporary hide, persistence-failure abort with warning capture), multi-item flyout lifecycle, and native OLE drag-out/cancel. Automated regression suite passes 226 tests. Same-payload performance evidence (100 batches / 1,000 items) records 0% CPU, show p95 7.20 ms; working set visible passes at 148.66 MB (<150 MB), while post-dismissal (150.50 MB) and post-30-cycles (157.99 MB) retain the independent V1 residency blocker, which is not waived. High Contrast verified in Windows Sandbox. Accepted-drop live growth, non-100% DPI, and short-work-area configurations remain unqualified.

**Transparent-surface corner change (unqualified for expansion):** the evidence above predates the transparent rail surface and 5-pixel input region. For the change itself, 96-DPI captures on white and red desktops show antialiased corners on both edges with the rail fill unchanged; the corner gap does not hit-test as rail while the body does; and 226 tests pass. Hover expansion, the flyout lifecycle, and drag-out have not been re-verified on this build: the automated hover drivers and `scripts/test-edge-rail-flyout.ps1` fail the same way on the unmodified baseline in the current environment.

### Shared shelf presentation

The Drop Shelf and Edge Rail reuse the same compiled 42-pixel bounded batch-preview template, Bulk Pinning visual template, item/batch projections, thumbnail eligibility, and realization lifetime. The existing visual coordinator/provider remains shared; each surface owns cancellation and ImageSource release independently. A batch requests at most its first three item visuals. Collapsed or hidden Edge Rail rows release their visual requests and detach the list content; expansion recreates visible rows through the existing virtualized list.

Manage Items on the Edge Rail now follows the Drop Shelf popup's visual language: a solid theme-aware surface, subtle one-pixel boundary, eight-pixel corners, six-pixel padding, flat shadow-free presentation, and matching header and item-row spacing, typography, and actions. The Drop Shelf remains unchanged. The Edge Rail retains its native Flyout, desktop-facing placement, 320–480 logical-pixel width, 480-pixel height limit, fixed header, vertical-only scrolling, no-activate interaction, keyboard focus rules, motion, and item lifecycle.

**Thumbnail/management parity progress:** shared Drop Shelf extraction and Edge Rail Manage Items presentation alignment are implemented and build in Release. Visual parity with Drop Shelf popup styling (solid background, 1px card stroke boundary, 8px corner radius, 6px padding, shadow suppression, and aligned item typography/actions) has been verified. Formal qualification for the independent total Working Set `<150 MB` release gate remains separate.


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

**Disappearing-destination investigation:** Historical installed attempts reported Copy after the instrumented primary target exited, but did not instrument or exclude the destination exposed underneath it. Controlled native and WinUI probes, followed by a clean DropCove Release run with an isolated test profile, retain temporary references when both destinations reject (5/5) and remove them when a backing destination actually accepts Drop (5/5). No native advantage is established by these tests, and the historical destination identity cannot be reconstructed from them. The responsive shelf remains unqualified pending its separate acceptance/performance gates and a planning decision on native cutover. `RequestedOperation=None` broke legitimate Copy and remains reverted; file existence and later processing are not acceptance signals.

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

## Bulk Pinning

Multi-item Shelf Batch cards expose a `24 × 24` bulk pin toggle before Manage and Remove. Its state is derived from the items: Off for all Temporary, On for all Pinned, and Indeterminate for Mixed. Activating Off or Mixed pins every item; activating On makes every item Temporary. Single-item cards keep their existing binary action, and the Edge Rail adds no pin controls.

Bulk Pinning includes Available, Unavailable, and confirmed-Missing references still present in the batch. It changes only item lifecycle, never availability, batch order, source files, or shelf sizing. One atomic SQLite statement commits the change before the manager replaces its in-memory item collection; no batch pin field or schema migration exists. Persistence failure leaves memory and storage unchanged and reports “Couldn’t update pinning. Nothing changed.” without automatic retry. Successful pinning is shown through the controls and counts, not a success message.


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
    Visible,
    EdgeDocked
}
```

DropCove uses a single continuous Drop Shelf presentation rather than separate modes. `Visible` represents the floating resizable shelf, `EdgeDocked` represents the screen-edge rail, and `Hidden` represents the dismissed state when no items remain or when the shelf is closed.

## Visible

`Visible` is the floating, responsive, resizable Drop Shelf. The user can resize the borderless window from every edge and corner down to a minimum of `180 × 180` logical pixels and up to the current monitor work area. Shelf Batches reflow within a responsive wrapping grid. Multi-item batches expose a non-modal anchored popup for detailed item inspection and actions.

## Hidden

Used whenever no floating shelf or Edge Rail is displayed. Dismissing an empty shelf or removing the final Shelf Item transitions to Hidden. Hidden preserves held batches if dismissed with content when Edge Rail is disabled, but under standard operation with Edge Rail enabled, non-empty dismissal transitions to `EdgeDocked`. Reopening the shelf restores the `Visible` state and reuses the same manager state.

## EdgeDocked

Used when the shelf is dismissed while it still contains items. Opening the shelf from the rail returns to `Visible` on the remembered monitor; emptying the last batch from either the Drop Shelf or Edge Rail transitions back to `Hidden`.

---

# 13. Placement State

Placement is modeled separately from display state.

```csharp
public enum ShelfPlacement
{
    Docked,
    Default,
    Cursor
}
```

The shelf presentation is independent from monitor placement:

| Invocation | Presentation | Placement |
| --- | --- | --- |
| Shake | Drop Shelf (`Visible`) | Cursor monitor / near cursor |
| Global hotkey or tray activation | Drop Shelf (`Visible`) | Cursor monitor / default work area |
| Edge interaction | Edge Rail (`EdgeDocked`) | Remembered docked monitor and edge |

Placement remains independent so display and monitor-position behavior do not become one oversized state model.

---

# 14. Dismissing the Drop Shelf

The hotkey toggles the shelf. `Esc` and the `×` button also dismiss it; losing focus does not. If an item popup is open, the first `Esc` closes the popup while keeping the shelf visible; a subsequent `Esc` dismisses the shelf.

While visible, the Drop Shelf reasserts its topmost z-order when the foreground application changes without activating itself. Closing an external viewer with `Alt+F4` must not leave the shelf behind the application that regains focus.

Dismissing the visible shelf preserves every Shelf Batch. With Edge Rail available, the resulting state is:

```text
Items.Count == 0 → Hidden
Items.Count > 0  → EdgeDocked
```

The remaining items may be Temporary, Pinned, or a mixture of both. Reopening from the rail restores the single `Visible` Drop Shelf on the remembered monitor with its preferred size intact.

---

# 15. Responsive Resizable Drop Shelf

The Drop Shelf provides one resizable surface rather than fixed Compact and Expanded modes.

## Shell appearance

- The Drop Shelf uses the Edge Rail's theme-aware `LayerFillColorDefaultBrush` as one continuous surface. Header and footer backgrounds are transparent, with no contrasting bands or separators. Shelf Batch cards, Shelf Item rows, status surfaces, drag feedback, and popup surfaces retain their existing visual hierarchy.
- Only the Drop Shelf requests native Windows 11 rounded corners through `DWMWCP_ROUND`. XAML fill and outline geometry use the system overlay corner radius; the actual window corners and shadow remain DWM-controlled. An accepted native hint is not proof of visible rounding. There is no Mica, Acrylic, custom shadow, transparent helper window, region, or clipping fallback, and Windows policy exceptions remain authoritative.
- A theme-aware, one-logical-pixel neutral card-stroke outline overlays the shell. It is inset one pixel to stay inside DWM's outermost window-border pixel and never consumes content space or changes padding, responsive columns, or row fit. The existing accent resize outline uses the same overlay geometry.
- Header and footer remain `32` logical pixels high. Settings, dismissal, and Clear Temporary Items retain `32 × 32`, `36 × 32`, and `28 × 28` interaction rectangles. Their glyph sizes are `11`, `10`, and `11` logical pixels, and only these three controls receive the standard rounded hover/pressed treatment and WinUI system focus brushes. Names, tooltips, commands, keyboard navigation, and shared card/popup action styles remain unchanged. The system focus brushes retain a legible keyboard cue in High Contrast instead of using a background-oriented accent-fill brush as a focus stroke.
- This visual change does not restyle or resize the Edge Rail, change saved geometry, or alter the `180/236/292/348` Automatic Shelf Growth tiers. Same-build Light/Dark evidence runs on the isolated host profile; actual High Contrast shell, target rectangles, and keyboard focus evidence runs only in a disposable Windows Sandbox guest. Host contrast is never enabled by qualification, and its exact theme/contrast baseline is preserved. Unavailable DPI, monitor, and native-policy configurations remain unobserved, not inferred passes.


## Window resizing and geometry

The borderless window supports native pointer resizing from all eight borders and corners (left, top, right, bottom, and four corners) without aspect-ratio locking. The borderless resize hit target is approximately `8` logical pixels around the window perimeter. Pointer hover shows the directional resize cursor and an accent outline over the neutral boundary. Moving into the caption, content, or outside the window restores the neutral outline and normal cursor without moving content. Non-client region enter/move/exit events own this hover treatment, so a transition into the input-sink window cannot clear a still-active resize hover.

Key sizing and geometry rules:

- **Minimum dimensions**: The shelf has a minimum preferred size of `180 × 180` logical pixels. Resizing cannot shrink the shelf below this minimum unless the monitor work area itself is smaller than `180 × 180`, in which case keeping the full window reachable takes precedence.
- **Maximum bounds**: The shelf expands up to the active monitor's work area. Resizing never permits actual window bounds to extend outside the current monitor work area.
- **Initial sizing**: The first time the shelf opens, it uses `180 × 180` logical pixels.
- **Sizing intent vs. actual bounds**: Preferred logical width and optional Manual Height Override are tracked separately from DPI-scaled, work-area-clamped actual bounds. Temporary clamping never overwrites either preference.
- **Shared preferences**: Width and explicit height are shared across monitors. A width-only resize preserves the optional height override and never changes height during reflow; a height-changing top/bottom or corner resize establishes a Manual Height Override that suppresses Automatic Shelf Growth.
- **Preference lifecycle & persistence**: User sizing intent persists in SQLite while at least one Shelf Item exists. Empty-shelf choices remain in memory until first acceptance, which persists width and nullable manual height atomically with content. Restart before acceptance discards pending choices. Existing retained-content size records are preserved as manual sizes on cutover.
- **Final-item reset**: When the final Shelf Item is removed (via manual removal, Clear Temporary Items, drag-out, or Missing cleanup) from either the visible Drop Shelf or Edge Rail, the shelf transitions to `Hidden`, closes any open popup, and deletes the persisted preferred-size record, resetting the next opening to `180 × 180`.
- **Automatic Shelf Growth**: With no Manual Height Override, an accepted drop or opening derives height from responsive rows at the current actual width: zero/one row uses `180`, two use `236`, three use `292`, and four or more use `348` logical pixels. Existing-row capacity does not grow the shelf; overflow scrolls vertically. Automatic growth preserves width and the top edge unless work-area reachability requires an upward correction, and closes an open batch popup before changing bounds.
- **No live shrink**: Removing content retains visible height while Shelf Items remain. A later show or restart recalculates the tier and can reclaim unused height. Hidden or EdgeDocked additions defer full-shelf geometry until opening; Edge Rail sizing and hover behavior are unchanged. Automatic height is never saved as user sizing intent.

## Responsive layout

Shelf Batches reflow within a responsive wrapping grid with natural vertical scrolling and disabled horizontal scrolling:

- **Padding and gaps**: The content surface maintains `8` logical pixels of horizontal padding on each side (`16` logical pixels total) and a `4` logical-pixel gap between adjacent columns and rows.
- **Card width**: Every batch card has a minimum width of `164` logical pixels. Cards stretch evenly to consume the available row width.
- **Exact column boundaries**: Column count is the largest positive integer $n$ such that $n \times 164 + (n - 1) \times 4 \le \text{window width} - 16$:
  - Width `180–347`: 1 column
  - Width `348–515`: 2 columns
  - Width `516–683`: 3 columns
  - Continuing by the same formula for wider windows.
- **Ordering**: Batches are ordered newest-first, filling left-to-right, then top-to-bottom.
- **Header & footer**: The header exposes Settings and dismissal (`×`). The footer bar provides `Clear Temporary Items` at the bottom-left. No window-level expand or compact toggle controls exist.

---

# 16. Batch Visuals

Every batch card maintains a consistent compact presentation regardless of shelf width or column count.

## Single-item batch

A single-item batch displays the item's icon/thumbnail and truncated name. It retains direct whole-card drag, Pin/Unpin, and Remove actions. It does not show a chevron or item popup trigger.

Example:

```text
╭──────────╮
│    🖼    │
│ logo.svg │
╰──────────╯
```

## Multi-item batch

A multi-item batch displays a stacked visual and item count to communicate grouped contents. It provides a whole-batch drag handle and a chevron toggle button that serves as the trigger for the item details popup. Inline expansion inside the shelf surface is not used.

Example:

```text
╭──────────╮
│ ╭────╮   │
│╭────╮│ ⌵ │
││ +4 ││   │
│╰────╯╯   │
╰──────────╯
```

---

# 17. Multi-Item Batch Popup

Multi-item batches provide detailed inspection and item management through a non-modal anchored popup rather than inline list expansion or a separate management window.

## Popup lifecycle and placement

- **Anchoring**: The popup is anchored to its originating batch card. It is non-modal and allowed to extend outside the Drop Shelf window boundaries, but remains clamped inside the monitor work area (with at least `16` logical pixels of margin where work area permits).
- **Placement preference**: Right of the card, then left, then below, then above, followed by work-area clamping.
- **Single instance**: Exactly one batch popup can be open at a time across the entire shelf. Activating another batch's chevron replaces the current popup.
- **Dimensions**: Width sizes to content between `320` and `480` logical pixels. Height sizes to content up to `480` logical pixels, with vertical scrolling enabled for additional items (horizontal scrolling disabled).
- **Opening and dismissal**:
  - Opens via pointer activation or keyboard `Enter` / `Space` on the batch chevron.
  - Closes via clicking the same chevron, clicking outside the popup, or pressing `Esc`.
  - Outside dismissal runs on pointer release, not press, so the underlying Settings/button action can activate with one click.
  - Activating another application does not close the popup; it remains anchored until an explicit dismissal or owner-lifecycle condition occurs.
  - Closes automatically when the Drop Shelf hides, when its owning batch is removed, when shelf resizing begins, or when whole-batch drag starts.
  - Long names and paths truncate visually with full text accessible via tooltips and accessibility properties.

## Item management and drag interactions

- **Content**: Each popup row displays the item's icon/thumbnail, name, path reference, availability classification, and pinned indicator.
- **Bulk Pinning header**: A fixed `X of N pinned` summary and tri-state bulk toggle sit above the item scroller. The header consumes part of the existing `480`-pixel overall height limit. It shares the card action's lifecycle, atomic persistence, accessible names, and Mixed pin badge. In-popup success or failure keeps the popup open and the bulk control focused; a card bulk action closes it under the normal outside-click rule.
- **Actions**: Users can Pin/Unpin individual items, Remove Item, or initiate native drag-out of an individual item directly from the popup row.
- **Drag behavior**:
  - Starting a drag of an individual item does not close the popup, allowing users to continue managing every remaining item. Canceled drags leave references intact. Successful drag-out applies the standard Temporary/Pinned lifecycle and refreshes the anchored popup while the batch remains; dragging or removing the final item removes the batch and closes its popup.
  - While an individual native drag is active, outside pointer release, app deactivation, and `Esc` do not dismiss the popup. `Esc` cancels that drag; the next ordinary `Esc` closes the popup. Drag completion clears this lifetime guard, including rejected and canceled operations.
  - Starting a drag of the entire batch from its card closes the popup immediately.

## Keyboard accessibility

When opened via keyboard (`Enter` or `Space` on the chevron), focus moves to the fixed header's bulk pin control. Pointer opening does not force focus into the popup. `Enter` and `Space` activate the focused bulk control. Pressing `Esc` closes the popup and restores keyboard focus to the originating chevron. A second `Esc` dismisses the Drop Shelf.

Popup `Tab`/`Shift+Tab` traversal visits the fixed header first, then each item's Pin/Unpin and Remove actions, wraps in both directions, and realizes only the next offscreen row on demand. It does not depend on global next-focus lookup across the popup's native visual root and never resizes the Drop Shelf.

## Deferred projections & performance

Full item view models, visual elements, and thumbnail requests are deferred until the popup opens. Closing the popup, hiding the shelf, or removing the batch releases popup item projections. The compact batch card maintains only bounded preview metadata.
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

Foreground monitoring is installed only while the rail is requested and fullscreen exclusion is enabled. It remains active while a fullscreen application temporarily hides the requested rail, so leaving fullscreen restores it. Hiding the rail explicitly releases the hooks. Location notifications are scoped to the current foreground window's thread and retargeted on foreground changes; background-window location traffic does not enqueue rail repositions.

If location-hook registration fails during a foreground notification, the native callback contains the `Win32Exception`, clears retry state, and retries on a later foreground notification. Initialization failures still report an exception after cleaning up the foreground hook.

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

It is not yet the full Drop Shelf.

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

## Level 3 — Drop Shelf

```text
≥ 180 × 180 px (resizable to work area)
```

Purpose:

```text
management and multi-column holding
```

Typical progression:

```text
Edge Rail
    ↓ hover

Interactive Edge Rail
    ↓ open

Drop Shelf
```

---

# 27. Shake State Restoration

Shake invocation temporarily overrides the current placement.

Example:

```text
EdgeDocked
   ↓
drag + shake
   ↓
Drop Shelf near cursor
```

If the user cancels the drag without dropping anything:

```text
Drop Shelf near cursor
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
Drop Shelf near cursor
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

## Popup appearance and dismissal

Use:

- opacity;
- scale.

Resize animations must never lag behind pointer interaction or block active drag-and-drop.
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
Idle working set target             < 160 MB
```

Thumbnail loading and SQLite I/O must not block the UI thread. V1 should remain responsive with at least 100 batches and 1,000 items by using lazy thumbnail loading and UI virtualization. The shelf realizes only visible item elements; native icons are requested independently, and image thumbnails are discarded when their visible realization ends.

At the 100-batch / 1,000-item scale, `DropShelfManager` remains the durable in-memory state owner while WinUI presentations retain only their active projections. Batch cards display compact preview summaries, full item view models and visuals are materialized only when a batch popup opens, Edge Rail item summaries are deferred until a flyout requests them, and closing a popup or hiding the Drop Shelf releases presentation projections. The default installed Release measurement remains untrimmed and uses `WorkingSet64`.

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
│   └── BatchPopup.xaml
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
│    ├── Visible
│    └── EdgeDocked
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

- Responsive resizable Drop Shelf with native 8 px border hit-testing, minimum 180 × 180 logical bounds, and work-area clamping.
- Responsive wrapping grid with min-card 164 px, 4 px gap, 8 px padding, and deterministic column boundaries.
- Anchored non-modal multi-item batch popup with keyboard focus, outside-click dismissal, and deferred projections.
- Multi-file stacked visuals and direct single/multi-item drag operations.
- Composition animations added after behavior is stable and respecting reduced motion.
- Full Edge Rail multi-monitor polish, accessibility, and installed-EXE release verification.

## Compatibility gate

Before V1.0, verify drag-in from Explorer/Desktop and drag-out to Explorer, Edge/Chrome file inputs, VS Code, and at least one common chat application. Folder drag-out must work with Explorer. Elevated cross-integrity drag/drop is not supported.

## Verification strategy

Automated tests cover shelf state transitions and SQLite persistence. WinUI, OLE interoperability, global hotkeys, window activation, DPI, hooks, and cross-application behavior are verified with actual smoke scenarios against the installed EXE release build rather than mocks alone.

### Isolated test-only launch

```powershell
DropCove.exe --test-profile "C:\Temp\DropCove-qualification"
```

- The explicit option places `shelf.db` and `settings.json` in the chosen absolute directory. Missing, relative, repeated, inline `--test-profile=...`, or normal-profile paths are rejected rather than silently using live data.
- Normal launch keeps `%LOCALAPPDATA%\DropCove` and the existing resident mutex/activation event unchanged. Test launch derives separate mutex/event names from the normalized directory; repeated launches of the same test profile activate its existing instance, while different test profiles can coexist with the normal app.
- A test window is titled `DropCove — Test profile`. Startup registration is never written on test startup, Settings save, or rollback; the setting can persist in test JSON without affecting the user's Windows Run entry.
- Use a separate real directory, not filesystem aliases to the normal profile. This is a test-only storage/instance seam, not general user-profile support or an OS sandbox. Global hotkeys still follow normal registration/conflict behavior; use distinct test hotkeys when running profiles concurrently.
- Drag acceptance evidence must correlate every destination that can receive the drop. A failed primary followed by a backing target's real Copy acceptance is a successful drag, not proof that the failed primary accepted it. Side-by-side published EXE smoke is not final NSIS installer qualification.

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

# 48. MSIX / Microsoft Store Submission

The unpackaged x64 self-contained EXE installer and the Store-oriented MSIX pipeline are separate delivery paths. The Store path is implemented by `scripts/build-msix.ps1` and documented in [`docs/store-msix.md`](store-msix.md).

Development uses the distinct self-signed `DropCove.Development` identity. Production generation requires Partner Center `IdentityName` and `Publisher` inputs at build time and emits an unsigned x64 self-contained MSIX/MSIXBundle for Microsoft Store signing. Private keys and Partner Center secrets are never stored in the repository.
