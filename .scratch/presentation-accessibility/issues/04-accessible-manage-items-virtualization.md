# 04: Accessible Manage Items Virtualization

**What to build:** Keep Manage Items responsive and accessible for a Shelf Batch containing thousands of Shelf Items. Small collections retain the simple full rendering path; large collections mount and request visuals only for the viewport and overscan while preserving the documented Tab order, focus retention, item-position semantics, Escape behavior, drag behavior, and focus restoration.

**Blocked by:** 03 — Bounded Shell Visuals Across All Surfaces.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] Manage Items renders all rows for collections of 50 or fewer Shelf Items.
- [ ] For collections above 50, only viewport and overscan rows own mounted presentation and active Shell Visual requests.
- [ ] A 2,000-item Shelf Batch opens, scrolls, mutates, and closes without mounting or requesting resources in proportion to all 2,000 items.
- [ ] The focused row remains mounted until focus moves; scrolling cannot drop focus to the document body.
- [ ] Tab and Shift+Tab retain the canonical Bulk Pinning → Close → each item Pin/Unpin → Remove order, scrolling and mounting the next logical target before focus moves.
- [ ] Virtual rows expose list-item semantics, total collection size, and correct one-based position; visual, keyboard, and accessible ordering are identical.
- [ ] Escape closes Manage Items before the Drop Shelf, and focus returns to the originating Manage Items control when it still exists.
- [ ] Bulk Pinning, individual Pin/Unpin, Remove Item, direct item drag-out, status dismissal, and popup replacement retain their 0.0.6 outcomes.
- [ ] Closing or replacing Manage Items clears its full metadata projection, queued visual work, mounted rows, and browser resources.

## Testing seam

Exercise the actual Manage Items surface with deterministic large isolated profile data and the approved Shell Visual seam. Observe DOM/UI Automation collection bounds, active request counts, focus movement, commands, and final Core Shelf State rather than testing virtualizer internals.

## Demo path

Open a 2,000-item batch by keyboard, traverse controls across multiple virtual windows, scroll to the end, pin and remove distant items, perform an item drag, close with Escape, and confirm focus restoration and bounded mounted/request counts.

## Non-goals

Drop Shelf or Edge Rail virtualization, arrow-key list navigation, external keyboard drag-out, persistent visual caching, or a new management workflow.

## Answer

Delivered in 0.0.7:
- `useVirtualCollection.ts` integrated into `ManageItems.vue`.
- Fully renders collections <= 50 items; bounds DOM nodes and requests to viewport + overscan for > 50 items.
- Preserves listitem semantics, `aria-setsize`, `aria-posinset`, focused element retention, and Tab/Shift+Tab focus traversal.
