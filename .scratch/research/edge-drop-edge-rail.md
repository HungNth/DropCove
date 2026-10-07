# Edge-Drop Edge Rail & Interaction Research

## Scope and source version

This research note examines the primary source code and assets of [Deepender25/Edge-Drop](https://github.com/Deepender25/Edge-Drop) at commit [`469fd0d56d8d4073ed5a6fbcab36953db8203cae`](https://github.com/Deepender25/Edge-Drop/tree/469fd0d56d8d4073ed5a6fbcab36953db8203cae) (latest commit on `main`, repository release v0.1.4). It evaluates Edge-Drop's architecture and user interaction design against the six proposed DropCove Edge Rail behaviors:
1. Compact idle size with hover expansion
2. Rounded corners and curvature
3. Item-driven size up to maximum then scrolling
4. Resize/expand icon placement
5. Trash / batch clear icon placement and scoping
6. Reusable batch-deletion code and architecture seam

Primary sources:
- [Repository README](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/README.md)
- [Architecture & Features documentation (`FEATURES.md`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/FEATURES.md)
- [Window creation and Win32 interop (`electron/main/window.ts`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/window.ts)
- [Screen geometry calculation (`electron/main/geometry.ts`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/geometry.ts)
- [Design tokens (`src/styles/tokens.css`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css)
- [Panel component (`src/components/Panel.tsx`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx)
- [Panel stylesheet (`src/styles/panel.css`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/panel.css)
- [Edge hover hook (`src/hooks/useEdgeHover.ts`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/hooks/useEdgeHover.ts)
- [Header component (`src/components/Header.tsx`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Header.tsx)
- [Clear menu component (`src/components/ClearMenu.tsx`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClearMenu.tsx)
- [Clipboard item component (`src/components/ClipboardItem.tsx`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClipboardItem.tsx)
- [Item stylesheet (`src/styles/item.css`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/item.css)
- [Batch deletion store implementation (`electron/store/ItemStore.ts`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/store/ItemStore.ts)
- [Settings pane component (`src/components/Settings.tsx`)](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Settings.tsx)

---

## 1. Compact Idle Size & Hover Expansion

### Observed Behavior vs. Implementation Mechanics
- **Native window remains full-size; visual clipping creates the idle state**: In Edge-Drop, the OS window does **not** physically resize or reposition when expanding or retracting. The Electron `BrowserWindow` is instantiated at full display work area height (`wa.height`) with width `PANEL_WIDTH = 384` (or `820` when preview is active), set to transparent, frameless, and `WS_EX_NOACTIVATE`.
  - Source: [`electron/main/window.ts:4-11, 116-118, 628`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/window.ts#L4-L11), [`electron/main/geometry.ts:115-125`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/geometry.ts#L115-L125).
- **Click-through idle state**: When collapsed/idle, the window invokes `setIgnoreMouseEvents(true, { forward: false })` (or `forward: true`). The visual UI is clipped to an invisible edge sensor strip via CSS `clipPath`.
  - Source: [`electron/main/window.ts:6`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/window.ts#L6), [`src/components/Panel.tsx:327-341`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L327-L341).
- **Idle dimensions**:
  - The invisible trigger sensor thickness is `hotZoneWidth = 3px` by default (configurable in Settings from 1px to 7px).
  - The vertical trigger height is `triggerHeightPx = Math.round(window.innerHeight * settings.hotZoneHeight)`, with options for 25%, 40% (default), or 60% of screen height.
  - Source: [`src/components/Panel.tsx:88, 327`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L88), [`src/hooks/useEdgeHover.ts:27`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/hooks/useEdgeHover.ts#L27).
- **Expanded blade dimensions**:
  - Blade width: Fixed at `--panel-width: 270px;` (CSS custom property).
  - Blade vertical height: Configurable fraction of viewport: `(settings.panelHeight || 0.6) * 100}vh` (options: 50%, 65% default, 80%). The blade is vertically centered via `top: 50%; transform: translateY(-50%)`.
  - Source: [`src/styles/tokens.css:55`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css#L55), [`src/components/Panel.tsx:92, 283-295`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L92), [`src/components/Settings.tsx:2285-2289`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Settings.tsx#L2285-L2289).
- **Expansion trigger and timing**:
  - Main-process OS cursor poll detects cursor at screen edge (`x <= TRIGGER_PX = 3`).
  - Dwell threshold: `DWELL_MS = 40ms` (standard edge) or 140ms seam policy for multi-monitor interior boundaries.
  - On open: Window switches to interactive (`setIgnoreMouseEvents(false)`), and container clips open via CSS transition: `clip-path 0.32s cubic-bezier(0.22, 1, 0.36, 1)`.
  - Source: [`src/hooks/useEdgeHover.ts:27-29`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/hooks/useEdgeHover.ts#L27-L29), [`src/components/Panel.tsx:247-249`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L247-L249).
- **Retraction and hysteresis**:
  - Uses a dead-band hysteresis: `KEEP_OPEN_PX = 255` (inside blade), `START_CLOSE_PX = 290` (20px outside visual blade). Cursor within 255–290px is ignored to prevent jitter.
  - Retraction delay: `GRACE_MS = 250ms`. Retraction animation: `clip-path 0.26s cubic-bezier(0.22, 1, 0.36, 1)`.
  - Source: [`src/hooks/useEdgeHover.ts:37-39`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/hooks/useEdgeHover.ts#L37-L39), [`src/components/Panel.tsx:256`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L256).

### Contrast with DropCove
- DropCove currently resizes its native Win32 window between `64 × 112` and `320 × 640` via `SetWindowPos` (`WindowInterop.PositionEdgeRail`) on a 200ms timer.
- Edge-Drop's approach maintains a static, transparent window viewport and animates a GPU-accelerated CSS `clipPath` mask, achieving 0ms window allocation/repositioning latency at the expense of an active transparent window covering part of the screen work area.

---

## 2. Rounded Corners & Edge Curvature

### Sourced Radii and Visual Assets
- **Blade outer radii**:
  - Token: `--radius-blade: 24px;` ([`src/styles/tokens.css:35`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css#L35)).
  - Left dock (`.blade`): `border-top-right-radius: 24px; border-bottom-right-radius: 24px;` (screen edge side is 0).
  - Right dock (`.blade-container.blade-right .blade`): `border-top-left-radius: 24px; border-bottom-left-radius: 24px;` (screen edge side is 0).
  - Source: [`src/styles/panel.css:25-26, 37-42`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/panel.css#L25-L26).
- **Reverse curve flares ("beak/bracket" screen transitions)**:
  - Edge-Drop bridges the visual transition between the 24px rounded corners and the straight monitor edge using SVG flares (`.flare-top`, `.flare-bottom`).
  - Geometry: 30 × 30 px vector path forming a tangential reverse curve:
    - Left side: `<path d="M 0 0 L 0 30 L 30 30 A 30 30 0 0 1 0 0 Z" fill="#000000" />` (top) and `<path d="M 0 30 L 0 0 L 30 0 A 30 30 0 0 0 0 30 Z" fill="#000000" />` (bottom).
    - Right side: `<path d="M 30 0 L 30 30 L 0 30 A 30 30 0 0 0 30 0 Z" fill="#000000" />` (top) and `<path d="M 30 30 L 30 0 L 0 0 A 30 30 0 0 1 30 30 Z" fill="#000000" />` (bottom).
  - Source: [`src/components/Panel.tsx:415-437`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L415-L437), [`src/styles/panel.css:57-88`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/panel.css#L57-L88).
- **Internal content radii**:
  - Cards: `--radius-card: 18px;` ([`src/styles/tokens.css:36`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css#L36)).
  - Pills / capsules: `--radius-pill: 999px;` ([`src/styles/tokens.css:37`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css#L37)).

### Contrast with DropCove
- DropCove's current `EdgeRailWindow` has rectangular window boundaries and `CornerRadius="5"` on batch items (`EdgeRailWindow.xaml:58`). Applying a 24px outer curve on the outer edge (with 0px along the docked monitor edge) directly matches Edge-Drop's visual language.

---

## 3. Adaptive Sizing vs. Fixed Viewport with Scrolling

### Sourced Sizing Behavior
- **Edge-Drop DOES NOT use item-driven adaptive sizing**:
  - The blade height is fixed to a user-configured viewport percentage: `(settings.panelHeight || 0.6) * 100}vh` ([`src/components/Panel.tsx:92`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L92)).
  - The blade width is fixed at `270px` ([`src/styles/tokens.css:55`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/tokens.css#L55)).
  - When history contains 0 items, the blade still expands to full configured height and renders `<EmptyState />` ([`src/components/ItemList.tsx`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ItemList.tsx)).
- **List overflow and scrolling**:
  - Content container (`.list`) uses flex layout with hidden native scrollbars:
    ```css
    .list {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow-y: auto;
      overflow-x: hidden;
      scrollbar-width: none;
    }
    ```
  - Source: [`src/styles/panel.css:273-290`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/panel.css#L273-L290).
- **Virtualization**:
  - Items are virtualized with `content-visibility: auto; contain-intrinsic-size: 84px;` in [`src/styles/item.css:25-27`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/item.css#L25-L27).

### Decision Relevance for DropCove
- The proposal for "item-driven size up to maximum then scrolling" is an **invented hybrid** not present in Edge-Drop.
- Edge-Drop's design is purely viewport-proportional (e.g. 65vh). If DropCove adopts content-driven growth (growing vertically as batches are added until reaching a 640px maximum, then scrolling), this represents an intentional product deviation from Edge-Drop.

---

## 4. Control Placement & Resize / Expand Icon

### Sourced Control Layout
- **Shelf window resize/expand control is ABSENT in Edge-Drop**:
  - Edge-Drop has **no button or icon** on the shelf to expand, resize, or open a larger window.
  - Hovering or pressing the global hotkey (`Alt+C`) is the sole trigger to open or close the shelf ([`README.md:280-285`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/README.md#L280-L285)).
- **Header control placement (`Header.tsx`)**:
  - Left / center: Category filter bar (`RubberSegment`, height 28px, pills for All, Text, Links, Images, Files, Colors, Emoji).
  - Right: Settings gear button (`32 × 32 px`, toggles Settings view) and Changelog info button (`32 × 32 px`).
  - Source: [`src/components/Header.tsx:76-89, 238-375`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Header.tsx#L76-L89).
- **Where Expand/Contract icons (`Maximize2` / `Minimize2`) actually appear**:
  - Located exclusively on **individual item cards** in a hover action pill (`.actions` in [`src/styles/item.css:444-470`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/item.css#L444-L470)).
  - Contains: Pin button, Expand button (`ExpandIcon` / `ContractIcon`), Copy button, and Trash button (`TrashIcon`).
  - Clicking this Expand icon opens a secondary 440px wide `PreviewFlyout` next to the blade.
  - Source: [`src/components/ClipboardItem.tsx:364-378`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClipboardItem.tsx#L364-L378), [`src/components/PreviewFlyout.tsx:285-290`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/PreviewFlyout.tsx#L285-L290).

### Decision Relevance for DropCove
- DropCove's `EdgeRailWindow` currently features an `OpenShelfButton` (`FontIcon Glyph="&#xE8A7;"`, size 56×32) centered at the top ([`EdgeRailWindow.xaml:25-35`](src/DropCove.App/EdgeRailWindow.xaml#L25-L35)).
- The question of moving a smaller resize/expand icon to the **top-left for a right rail** and **top-right for a left rail** (i.e. the outer corner facing into the desktop) is a DropCove ergonomic refinement: it places the affordance away from the screen edge seam while freeing the center for content or badges. Edge-Drop has no equivalent window-level control.

---

## 5. Trash / Clear Icon Placement & Scoping

### Sourced Deletion UI
- **Shelf-level footer deletion**:
  - The vertical blade includes a footer: `.footer { display: flex; align-items: center; justify-content: space-between; }` ([`src/styles/panel.css:574-583`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/styles/panel.css#L574-L583)).
  - **Left side of footer**: `.footer-capsule` displaying the current item count (`filteredCount`) ([`src/components/Panel.tsx:513-517`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L513-L517)).
  - **Middle**: `.spacer` (`flex: 1`) ([`src/components/Panel.tsx:518`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L518)).
  - **Right side of footer**: `<ClearMenu>` button with `<TrashIcon width={13} height={13} /> <span>Clear</span>` ([`src/components/Panel.tsx:519-532`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L519-L532), [`src/components/ClearMenu.tsx:112-117`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClearMenu.tsx#L112-L117)).
  - **Clear Menu interaction**: Clicking does not delete immediately. It opens an anchored popup menu ([`src/components/ClearMenu.tsx:120-169`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClearMenu.tsx#L120-L169)) offering:
    1. "Last 1 hour"
    2. "Last 6 hours"
    3. "Last 24 hours"
    4. Divider
    5. "Clear all" (swaps to "Are you sure?" on first click, requiring two clicks to confirm).
  - **Filter scoping & Pin safety**:
    - Pinned items (`item.pinned === true`) are strictly preserved and never cleared by time-window or Clear All actions.
    - If a search query or category filter is active, clearing is scoped only to visible unpinned items ([`src/components/Panel.tsx:525-530`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/Panel.tsx#L525-L530)).
- **Item-level deletion**:
  - Each item card has a red hover action button `<button className="act danger"><TrashIcon /></button>` that calls `remove(item.id)` accompanied by a synthesized mechanical thud sound (`playDeleteSound()`).
  - Source: [`src/components/ClipboardItem.tsx:407-420`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/components/ClipboardItem.tsx#L407-L420).

### Decision Relevance for DropCove
- In Edge-Drop, the shelf trash button is on the **bottom-RIGHT**, NOT bottom-left. The bottom-left holds the count pill.
- In DropCove's `MainPage` (Drop Shelf), the trash button (`ClearTemporaryButton`, glyph `&#xE74D;`, 28×28) is placed on the **bottom-LEFT** ([`MainPage.xaml:411-423`](src/DropCove.App/MainPage.xaml#L411-L423)).
- If DropCove adds a trash button to `EdgeRailWindow`, placing it on the bottom-left mirrors DropCove's own Drop Shelf visual alignment rather than Edge-Drop's bottom-right placement.

---

## 6. Reusable Batch-Deletion Code & Architecture Seam

### Sourced Edge-Drop Implementation
- **IPC bridge**:
  - Renderer calls `window.edge.deleteBatchItems(ids)` ([`src/store/appStore.ts:572`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/store/appStore.ts#L572), [`src/preload/index.ts:183`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/src/preload/index.ts#L183)).
  - Main IPC handler `item:delete-batch` passes `ids` directly to `getStore().deleteBatch(ids)` ([`electron/main/ipc.ts:278`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/main/ipc.ts#L278)).
- **Store architecture (`ItemStore.ts:323-345`)**:
  ```ts
  deleteBatch(ids: string[]): void {
    if (!ids || ids.length === 0) return
    const set = new Set(ids)
    const toRemove: ClipboardItem[] = []
    this.items = this.items.filter((it) => {
      if (set.has(it.id)) {
        toRemove.push(it)
        return false
      }
      return true
    })

    for (const removed of toRemove) {
      this.sigToId.delete(contentSignature(removed.data))
      if (removed.data.kind === 'image') this.removeImageFile(removed.data.imageId)
      if (removed.data.kind === 'image-collection') {
        removed.data.images.forEach((img) => this.removeImageFile(img.imageId))
      }
      if (removed.data.kind === 'text') this.removeTextPayload(removed.id)
    }
    this.persistSync()
    this.notifyRemoved(toRemove)
  }
  ```
  - **Key architectural choices**:
    1. Single-pass array filter using an in-memory `Set`.
    2. Explicit cascade cleanup for external disk payloads (thumbnails, raw images, text payload files).
    3. Synchronous atomic file persistence (`persistSync()`).
    4. Single notification emit (`this.notifyRemoved(toRemove)`) to notify subscribers once for the batch rather than per item.
  - Source: [`electron/store/ItemStore.ts:323-345`](https://github.com/Deepender25/Edge-Drop/blob/469fd0d56d8d4073ed5a6fbcab36953db8203cae/electron/store/ItemStore.ts#L323-L345).

### Application to DropCove Seam
- DropCove's core lifecycle manager `DropShelfManager` already defines the required batch removal operations:
  - `RemoveBatchAsync(Guid batchId)` removes one batch from memory and `ShelfDatabase` ([`DropShelfManager.cs:278-291`](src/DropCove.Core/DropShelfManager.cs#L278-L291)).
  - `ClearTemporaryItemsAsync()` removes all unpinned temporary batches ([`DropShelfManager.cs:294-315`](src/DropCove.Core/DropShelfManager.cs#L294-L315)).
- Currently, `EdgeRailWindow` does not expose or call any deletion methods.
- To create a reusable batch-deletion seam in DropCove:
  - Provide a shared batch deletion delegate or pass `DropShelfManager` directly into `EdgeRailWindow` (which already receives `DropShelfManager manager` in its constructor: [`EdgeRailWindow.xaml.cs:50`](src/DropCove.App/EdgeRailWindow.xaml.cs#L50)).
  - Invoking `_manager.RemoveBatchAsync(batchId)` or `_manager.ClearTemporaryItemsAsync()` followed by `_refreshAfterMutation()` mirrors the exact seam used by `MainPage.xaml.cs:457-458, 477-483`.

---

## Explicit Unknowns and Static Source Limitations

1. **DWM Render Composition Overhead**: Edge-Drop uses GPU layer promotion (`transform: translateZ(0)`) and `-webkit-background-clip: padding-box` inside Chromium. Whether rendering 30×30 SVG flare reverse curves inside a WinUI 3 XAML window or composition brush introduces antialiasing artifacts or increases resident working set cannot be determined from static source.
2. **Dwell Duration Tuning**: Edge-Drop uses a 40ms edge dwell (`DWELL_MS = 40`) with 140ms on interior monitor boundaries, whereas DropCove currently uses 200ms (`_expandTimer.Interval = TimeSpan.FromMilliseconds(200)`). The optimal human-factors threshold to balance intentional summoning vs. accidental activation cannot be verified without live user testing.
3. **Multi-DPI DWM Window Sizing**: Edge-Drop avoids native window resizing entirely by using a full-height transparent window. DropCove resizes the native WinUI `AppWindow`. Whether dynamic XAML item-driven resizing causes visual stutter during fast cursor entry cannot be proven from Edge-Drop's Chromium-based implementation.
