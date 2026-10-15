# 02: Compact Shelf Batch and Manage Items Controls

**What to build:** Replace verbose Shelf Batch and Manage Items text actions with compact single-row icon controls while preserving every established command, keyboard path, tooltip, accessible name/state, drag workflow, and focus-restoration rule. Remove visible card drag-hint subtitles without removing full-path or drag guidance, align the Manage Items rounded surface with its window frame, and update virtualization measurements to the resulting compact geometry.

**Blocked by:** 01 — Compact Drop Shelf Surface.

**Status:** resolved

**Parent specification:** [DropCove 0.0.10 — Compact Drop Shelf and Edge Rail UI Refinement](../spec.md)

## Acceptance criteria

- [x] Single-item Shelf Batch cards show Shell Visual, truncated name, direct drag region, Pin icon, and Remove Reference icon without visible `Drag to copy` or `Pinned · Drag to copy` subtitles.
- [x] Multi-item Shelf Batch cards show bounded stacked previews, item-count title, direct batch drag region, tri-state Pin All icon, Manage Items icon, and Remove Batch icon without a visible `Drag batch to copy` subtitle.
- [x] Card actions form one compact trailing row and use `24 × 24` logical-pixel buttons with `14–16` logical-pixel inline SVG icons inheriting `currentColor`.
- [x] Pin uses outline/filled pushpin states, Manage Items uses a list icon, and Remove uses minus-in-circle rather than a trash icon; Close uses `X`.
- [x] The draggable card region retains its existing path/title tooltip and accessible `drag to copy` instruction after visible hints are removed.
- [x] Binary Pin buttons expose dynamic accessible names and `aria-pressed`; their icon and accent treatment match the current state.
- [x] Bulk Pinning remains a three-state checkbox with `aria-checked="false"`, `"true"`, or `"mixed"`. Visual selectors follow `aria-checked`, and Off/Mixed/On remain distinguishable in normal and Windows Contrast presentation.
- [x] Manage Items retains its title, item count, and `X/N pinned` summary while Bulk Pinning and Close become compact icon controls in one trailing header row.
- [x] Each Manage Items row retains Shell Visual, name, shortened path/full-path tooltip, availability, and direct drag-out while Pin and Remove become one compact trailing icon row.
- [x] Every icon action has a native `title` tooltip, concise action-specific accessible name, hover state, visible `focus-visible` indicator, disabled state, and correct forced-color presentation.
- [x] Pointer opening, keyboard opening, Tab/Shift+Tab order, Enter/Space activation, Escape guarding, virtualized focus movement, explicit close, and focus restoration remain unchanged.
- [x] Manage Items renders with a flush rounded background/border and no opaque or transparent corner artifact when opened from either the Drop Shelf or Edge Rail.
- [x] Drop Shelf, Edge Rail, and Manage Items virtualization measurements match the compact rendered card/row extents; scrolling, bounded mounting, logical item positions, and focus retention do not overlap, skip, or strand entries.
- [x] Pin, Bulk Pinning, Manage Items, Remove Item, Remove Batch, Close, item drag, and batch drag continue to invoke the existing lifecycle paths without a new state owner or compatibility path.
- [x] Package, crate, Tauri, PE, and installer metadata remain `0.0.9`.

## Testing seam

Use the actual Drop Shelf, Manage Items, and expanded Edge Rail in the Windows Resident Instance. Exercise single-item, multi-item, all-Temporary, all-Pinned, mixed, unavailable, virtualized, busy, error, pointer-opened, and keyboard-opened states. Verify visible content, native tooltips, accessibility states, Tab order, Enter/Space, Escape, focus restoration, and bounded scrolling in Dark, Light, and Windows Contrast presentation.

## Demo path

Create one single-item batch, one mixed multi-item batch, and one batch large enough to virtualize Manage Items. Operate every compact icon with pointer and keyboard, move Bulk Pinning through Off/Mixed/On, drag an item and a batch, remove an item and a batch, close and reopen Manage Items, and show that visible drag hints are gone while tooltip/accessibility guidance and focus restoration remain.

## Non-goals

New Shelf actions, altered pin/remove semantics, custom tooltips, a general component library, new keyboard navigation, Edge Rail corner/header placement, release version synchronization, packaging, or Git commit creation.

## Answer

- Implemented compact card/row/header icons, dynamic binary Pin states, Bulk Pinning checkbox states, native title tooltips, and transparent no-shadow Manage Items. Removed visible drag hints and retained path/accessibility drag guidance.
- Actual native cards measured 48 pixels; management rows measured 54 pixels. A 70-item management batch mounted 10 initial rows. Flex spacers initially clamped scrollTop to 163; `flex: none` fixed the actual path. Replayed scrollTop 1500 showed first index 23 and 12 mounted rows.
- Real pointer and keyboard management opening, Tab sequence Bulk → Close → item Pin → item Remove, Space Pin activation, Close/Escape, binary Pin and Off/Mixed/On transitions were exercised. Full Shift+Tab/focus-restoration/UI Automation and actual Windows Contrast acceptance remain unchecked.
- Final installed application rendered Manage Items in Dark, Light, and forced-colors media emulation. Real mouse/OLE Temporary Copy consumed only its reference, cancellation retained its reference, and Pinned batch Copy retained both references with exact destination/source bytes. Destination was owned Windows Forms, not Explorer.
- `pnpm build` and native regression passed; final two-axis review found no source defect. The `.action` classes remain intentionally live because keyboard focus traversal queries them. No commit created.