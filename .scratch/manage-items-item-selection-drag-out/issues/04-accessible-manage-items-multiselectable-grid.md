# 04: Accessible Manage Items multiselectable grid

**What to build:** Make Manage Items Item Selection fully operable and understandable by keyboard and assistive technology. The popup exposes a valid multiselectable grid around rows with independent actions, provides stable roving item focus across virtualization, mirrors pointer selection with keyboard commands, announces the final selected count, and recovers focus deterministically after item or batch mutation.

**Blocked by:** 02: Manage Items pointer Item Selection Copy round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** Ticket 02's stable Item Selection owner, the existing virtual collection and focus helpers, native Pin/Remove controls, popup open-origin information, and the polite announcement channel.

**Testing seam:** Frontend DOM interaction/accessibility coverage plus the actual Windows Manage Items accessibility tree or a screen reader on ordinary and virtualized collections, with observed focus owner, Tab sequence, selected state, and announcements.

**Demo path:** Keyboard-open Manage Items from the Drop Shelf, form selections with Space variants and `Ctrl+A`, navigate beyond the mounted viewport, traverse the complete Tab cycle, remove focused items at several positions, reduce the batch to one item, close the popup, and inspect the accessibility tree and announcements.

- [x] The item collection exposes `role="grid"` and `aria-multiselectable="true"` with an accessible name.
- [x] Every item row exposes explicit `aria-selected="true"` or `"false"`; selection is not communicated only through color.
- [x] One row drag region is the roving grid focus target while Pin/Unpin and Remove remain independent native buttons with existing names and pressed state.
- [x] Keyboard opening from the Drop Shelf focuses the first item drag region; pointer opening does not force focus or foreground activation.
- [x] Up/Down moves by one source item; Home/End reaches complete collection boundaries and mounts virtualized targets before focus moves.
- [x] `Space` selects one, `Ctrl+Space` toggles, `Shift+Space` selects the anchor range, and `Ctrl+Shift+Space` uses replacement-range behavior.
- [x] `Ctrl+A` operates only while the item grid owns focus, selects every item including virtualized rows, and neither changes a valid anchor nor creates an absent one.
- [x] The cyclic Tab order is Bulk Pinning, Close, the grid's primary drag-region stop, then item Pin/Unpin and Remove controls in source order.
- [x] Virtualized action targets are mounted and scrolled into view before Tab focus moves.
- [x] Selection changes announce `1 item selected` or `N items selected`; clearing announces `Selection cleared`; synchronous changes coalesce to the latest message.
- [x] If the focused item disappears while the batch survives, focus moves to the item at the same source index, otherwise the previous item, otherwise Bulk Pinning.
- [x] Surviving selected identities remain selected after unrelated item removal, and a removed anchor is cleared without automatic replacement.
- [x] If explicit close occurs after the batch becomes a one-item card, focus returns to that card's drag region.
- [x] If the batch disappears, the Drop Shelf recovers to the batch at the same displayed index, otherwise the previous batch, otherwise More.
- [x] A pointer-opened Edge Rail origin receives no keyboard focus or live-announcement ownership.
- [x] Keyboard support stops at selection and ordinary controls; no keyboard-only native OLE destination drag is added.
- [x] Dark, Light, Windows Contrast, and reduced-motion presentation retain distinct selected, focused, hovered, disabled, and ordinary states.

## Evidence — 2026-10-26

- Actual installed accessibility tree exposed the multiselectable grid, selected rows, gridcells, independent Pin/Remove buttons, and full path names. Keyboard opening focused item **1**; the polite channel reported `65 items selected`.
- Exact installed **65-item** grid selected the complete first-to-last range while mounting **10 rows** at the end; popup measured **240 × 480**, and every mounted row measured **54 pixels** high.
- Actual Tab traversal followed Close, grid stop, then source-order Pin/Remove controls after Bulk Pinning. Ordinary removal recovered the surviving same-index item; virtualized last-item removal recovered the previous item; disappearing batch recovered the surviving parent batch.
- Dark and Light resource projection were visually inspected. Forced-colors and reduced-motion were verified through active Chromium media emulation, including system Highlight selected fill and distinct focus. Actual Windows Contrast Theme switching, physical high/mixed DPI, touchpad input, and spoken screen-reader output were not exercised; accessibility-tree inspection was used.
- Shared final release evidence: [Ticket 06](06-release-dropcove-0.0.26.md).
