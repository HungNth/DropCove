# 05: Accessible Shelf and Rail Batch Virtualization

**What to build:** Keep the Drop Shelf batch grid and expanded Edge Rail bounded when hundreds of Shelf Batches are retained. Small collections continue to render directly; large collections render viewport and overscan rows/cards while preserving batch order, first-three Shell Visual previews, focus continuity, no-activate Rail behavior, drag/drop, management, pinning, removal, hover timers, and opening behavior.

**Blocked by:** 03 — Bounded Shell Visuals Across All Surfaces.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] Drop Shelf and expanded Edge Rail render all Shelf Batches when their collection contains 50 or fewer entries.
- [ ] Above 50 batches, only viewport and overscan cards/rows own mounted presentation and active Shell Visual requests.
- [ ] A 200-batch Shelf remains responsive while opening, scrolling, resizing, docking, expanding, collapsing, and reopening.
- [ ] Multi-item cards continue to request at most the first three Shell Visuals, and offscreen cards request none until entering viewport or overscan.
- [ ] Virtualized entries expose collection size and correct one-based position; visual and accessible ordering matches persistent Shelf Batch order.
- [ ] Keyboard traversal of Drop Shelf actions retains logical order and preserves the focused card/control while scrolling or resizing changes the virtual window.
- [ ] The Edge Rail remains no-activate and does not add keyboard traversal or a second global keyboard hook; keyboard-only users continue to open the Drop Shelf through existing access paths.
- [ ] Single-item and whole-batch drag-out, Manage Items opening, Bulk Pinning, Remove Batch, Clear Temporary, incoming OLE drop, resize, Rail hover/drag timers, Open Shelf, fullscreen policy, and dismissal retain their 0.0.6 outcomes.
- [ ] Hiding the Shelf or collapsing the Rail releases mounted presentation and active visual ownership outside the next surface generation.

## Testing seam

Exercise the real Drop Shelf and Edge Rail with 200 ordered batches in an isolated profile. Observe rendered/UI Automation collection bounds, active Shell Visual requests, focus, drag results, and Core Shelf State through the Resident Instance rather than asserting internal range calculations.

## Demo path

Open a 200-batch Shelf, scroll and resize through distant batches, open Manage Items, drag and remove entries, dismiss to the Edge Rail, expand and scroll the Rail, then reopen the Shelf. DOM and visual work remain bounded and ordering/actions remain correct.

## Non-goals

Manage Items row virtualization, new Rail keyboard navigation, changing grid/card geometry, alternate sorting, paging controls, or persistence changes.

## Answer

Delivered in 0.0.7:
- `useVirtualCollection.ts` integrated into `DropShelf.vue` and `RailHandle.vue`.
- Collections <= 50 batches render directly; > 50 batches render viewport + overscan with `aria-setsize` and `aria-posinset`.
- Verified live against 61-batch intake test harness.
