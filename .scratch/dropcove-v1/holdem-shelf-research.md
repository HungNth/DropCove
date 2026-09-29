# Holdem shelf/window research

## Scope and source version

This note examines the public `iamzubin/holdem` repository, using the current `master` source (desktop app version `3.1.1`) plus the `3.1.1` release notes and first-party website assets. Holdem's current UI has a compact main shelf and a separate fixed-size popup list; it does **not** model file groups/batches or progressively resize the shelf as items accumulate.

Primary sources:

- [Repository README](https://github.com/iamzubin/holdem/blob/master/README.md)
- [Tauri window configuration (`tauri.conf.json`)](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/tauri.conf.json)
- [Main shelf UI (`App.tsx`)](https://github.com/iamzubin/holdem/blob/master/app/src/App.tsx)
- [Stacked compact visual (`StackedIcons.tsx`)](https://github.com/iamzubin/holdem/blob/master/app/src/components/StackedIcons.tsx)
- [Expanded popup UI (`PopupWindow.tsx`)](https://github.com/iamzubin/holdem/blob/master/app/src/PopupWindow.tsx)
- [Popup/window creation (`window_ops.rs`)](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/commands/window_ops.rs)
- [File storage/append behavior (`file_ops.rs`)](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/commands/file_ops.rs)
- [Release 3.1.1](https://github.com/iamzubin/holdem/releases/tag/3.1.1)
- [Project changelog](https://github.com/iamzubin/holdem/blob/master/CHANGELOG.md)
- [First-party docs](https://holdem.iamzub.in/docs)
- [First-party marketing image (`og.png`)](https://holdem.iamzub.in/og.png)

## Sourced dimensions and layout constants

### Compact/main shelf

The Tauri configuration defines a **165 × 175 logical-pixel inner window**. It is undecorated, non-resizable, always-on-top, not focus-stealing at creation, skipped from the taskbar, and visible on all workspaces. The same `165 × 175` size is present in the `3.0.0`, `3.1.0`, and `3.1.1` configurations.

Source: [`tauri.conf.json`](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/tauri.conf.json).

The main React root uses `p-2`, which is **8 px per side with the default Tailwind spacing scale**. It therefore leaves approximately **149 × 159 px** inside the 165 × 175 window for the root's content (before considering child layout). This is a CSS/layout calculation, not a separately declared window constant.

Source: [`App.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/App.tsx); [Tailwind config](https://github.com/iamzubin/holdem/blob/master/app/tailwind.config.js).

Within that compact window:

- The title/drag row is `h-5` = **20 px**.
- The centered drag handle is `w-10 h-0.5` = **40 × 2 px**.
- Settings and close buttons are explicitly `h-5 w-5` = **20 × 20 px**, with 16 × 16 px icons.
- The compact file visual is a fixed `w-10 h-10` = **40 × 40 px** container.
- The bottom count button uses the shared default button size `h-9` = **36 px**, with `px-4 py-2` (16 px horizontal and 8 px vertical padding), plus the wrapper's `mt-1` = **4 px** top margin.

Sources: [`App.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/App.tsx), [`button.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/components/ui/button.tsx), and [`global.css`](https://github.com/iamzubin/holdem/blob/master/app/src/global.css).

### Compact multi-item visual

`StackedIcons` always renders `files.slice(-5).reverse()`: the compact shelf shows **at most five visual layers**, regardless of the total file count. Each layer fills the same 40 × 40 container. Layers use these explicit transforms:

- rotation: `0`, `-10`, `-20`, `-30`, `-40` degrees;
- translation: `(0,0)`, `(-1,-1)`, `(-2,-2)`, `(-3,-3)`, `(-4,-4)` px;
- latest items are rendered first in the reversed slice and receive the highest visible stacking order.

The count button still reports the **full `files.length`**, so the stack is a bounded summary rather than a count-limited data model. The compact window itself does not grow when the count exceeds five.

Source: [`StackedIcons.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/components/StackedIcons.tsx), [`App.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/App.tsx), and English plural labels in [`en.json`](https://github.com/iamzubin/holdem/blob/master/app/src/i18n/locales/en.json).

### Popup/list window

Clicking the compact count button creates a second popup with a fixed **450 × 350 logical-pixel inner size**. It is undecorated, non-resizable, shadowless, always-on-top, not focused at creation, and positioned **5 logical px below** the main shelf. Its horizontal position is centered under the main window; the code converts the main window's physical position/size to logical pixels using the current scale factor before calculating the popup position.

Source: [`window_ops.rs`](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/commands/window_ops.rs).

The popup root adds a 1 px border and `p-2` (**8 px padding**). With the normal border-box layout, the resulting list/header region is approximately **432 × 332 px** inside the 450 × 350 window. This 432 × 332 figure is a layout calculation; the source's authoritative constant is 450 × 350.

Popup header/layout constants:

- Root: `overflow-hidden`, `p-2`, rounded border.
- Header: bottom margin `mb-2` = **8 px**.
- List/grid toggle items use the shared default toggle size `h-9` = **36 px**.
- The file list is in a `flex: 1; min-height: 0` SimpleBar viewport, so overflow scrolls inside the fixed popup rather than resizing it.

Sources: [`PopupWindow.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/PopupWindow.tsx), [`toggle.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/components/ui/toggle.tsx), and [`global.css`](https://github.com/iamzubin/holdem/blob/master/app/src/global.css).

Popup item sizes:

- **List mode:** each item is `p-1` (**4 px padding per side**), `gap-2` (**8 px**), with a fixed **32 × 32 px** icon/preview box. The row's minimum content height is therefore about **40 px** (32 px icon + 8 px vertical padding), with `space-y-1` = **4 px** between rows. Names use `text-xs` (12 px) and sizes use `text-[10px]` (10 px).
- **Grid mode:** the popup uses **two columns** (`grid-cols-2`) with **4 px** row/column gaps (`gap-1`). Each item uses 4 px padding, a **48 × 48 px** icon/preview box, and a 4 px bottom margin under the icon. Names remain 12 px and sizes 10 px. Exact rendered row height depends on browser line-height, but the icon and spacing constants are sourced.

Source: [`PopupWindow.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/PopupWindow.tsx).

## Behavior with multiple items and multiple groups

### What the source actually does

Holdem has a single flat `Vec<FileMetadata>`/`files` array, not groups or batches. `add_files` loops through incoming paths and appends each unique path to the same list; it rejects a path already present anywhere in that list. There is no incoming-drop ID, group ID, group header, per-drop separator, or per-group visual treatment.

Source: [`file_ops.rs`](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/commands/file_ops.rs), [`types.ts`](https://github.com/iamzubin/holdem/blob/master/app/src/types.ts), and [`useFileManagement.ts`](https://github.com/iamzubin/holdem/blob/master/app/src/hooks/useFileManagement.ts).

Consequences:

1. A single drop containing many files becomes many flat items.
2. Later drops append to the same flat list; they do not create a separate visible group.
3. Duplicate paths are globally suppressed, not merely deduplicated within one drop.
4. The compact shelf remains 165 × 175 and shows only a five-layer summary; it does not progressively widen, heighten, or add rows as items arrive.
5. The popup remains 450 × 350. In list mode, additional items extend the scrollable vertical content; in grid mode, additional items fill the two-column grid and extend its scrollable vertical content.
6. Popup selection is item-level: the user can select one item, Ctrl/Cmd-toggle items, or Shift-select a range, then drag the selected flat set or remove it. This is not group selection.

Sources: [`PopupWindow.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/PopupWindow.tsx), [`file_ops.rs`](https://github.com/iamzubin/holdem/blob/master/app/src-tauri/src/commands/file_ops.rs), and [`useFileManagement.ts`](https://github.com/iamzubin/holdem/blob/master/app/src/hooks/useFileManagement.ts).

### Release-documented overflow behavior

Release `3.1.1` specifically says it fixed “popup shelf clipping the last row” so that the file list scrolls correctly without cutting off the final item. The first-party changelog repeats this. That confirms the intended multiple-item behavior is a bounded popup with scrolling, not automatic window growth.

Sources: [GitHub release 3.1.1](https://github.com/iamzubin/holdem/releases/tag/3.1.1) and [first-party changelog](https://holdem.iamzub.in/changelog).

### Auto-close behavior

The popup starts a 3-second inactivity timer. If the user has not interacted, it closes. Once interacted with, it closes on blur. This applies regardless of how many items are in the list.

Source: [`PopupWindow.tsx`](https://github.com/iamzubin/holdem/blob/master/app/src/PopupWindow.tsx).

## Growth direction and limits

| Surface | Growth direction | Source-backed limit/behavior |
|---|---|---|
| Compact shelf | None; fixed window and fixed 40 × 40 stack area | Window 165 × 175; only last 5 layers rendered; full count remains available through button |
| Popup list mode | Downward content growth inside a scroll viewport | Window fixed at 450 × 350; one column; ~40 px minimum item row plus 4 px row gap |
| Popup grid mode | Downward content growth inside a scroll viewport; two columns | Window fixed at 450 × 350; 48 × 48 icon boxes; 4 px grid gaps |
| Number of stored items | Flat list growth, not visual window growth | No source-declared maximum item count in the inspected UI/storage code |
| File groups/batches | Not represented | No group concept or group-specific expansion exists in Holdem |

## Visual estimates (not runtime constants)

The first-party `og.png` marketing composition is 1280 × 640 and shows a small dark shelf containing a visibly stacked image and a `6 files` button. Measuring the displayed composite by eye gives an approximately **238 × 252 px** shelf silhouette (roughly x=849–1087, y=193–445 in the 1280 × 640 image). This is an image-composition estimate, not a runtime measurement; the source configuration's authoritative runtime size is 165 × 175 logical px.

Source image: [`holdem.iamzub.in/og.png`](https://holdem.iamzub.in/og.png). The repository README also presents the animated demo asset at a Markdown display width of 720 px, but that display width is documentation markup, not the app's window size: [README](https://github.com/iamzubin/holdem/blob/master/README.md).

## Safe design references for DropCove

Borrow the **interaction pattern**, not the implementation:

- A deliberately bounded compact shelf can stay visually small while still exposing the total item count.
- A fixed compact icon area with a shallow, rotated/offset stack communicates “multiple items” without rendering every item.
- A separate management surface can expose all items and use a fixed viewport with scrolling rather than allowing the window to grow indefinitely.
- A two-column grid is a reasonable alternative to a dense list when item previews matter; a list is better for names and metadata.
- Keep compact and management presentations fed by the same underlying state. Holdem's flat-list limitation is not a reason to copy its lack of Shelf Batch semantics into DropCove.

For DropCove specifically, the safe borrowing boundary is the visual language: **compact fixed footprint, bounded stack summary, explicit count, then scrollable expansion**. Do not copy Holdem's global duplicate suppression or flat item model: DropCove's specification requires separate Shelf Batches for separate accepted drops and preserves item order within each batch.

## Uncertainties and non-findings

- The source declares Tauri logical-pixel sizes; actual physical pixels vary with Windows display scaling/DPI.
- The inspected Holdem source contains no progressive compact-window resize formula, no group/batch layout, and no documented maximum number of stored files.
- Marketing screenshots/video are illustrative and composited; they should not override the source constants.
- The popup's exact text line heights and final row heights depend on browser/Tailwind defaults, so only the explicit icon, padding, gap, and window constants above should be treated as sourced dimensions.
