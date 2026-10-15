# 02: Correct Drop Shelf Shelf Batch Viewport Spacing

**What to build:** Give the populated Drop Shelf equal 7-pixel left-border-to-card and card-to-scrollbar-channel clearances, plus 8-pixel top and bottom clearance, while preserving native scrolling, responsive columns, virtualization, resizing, drag workflows, and final-card reachability.

**Blocked by:** None (can start immediately).

**Status:** ready-for-human

**Parent specification:** [Lucide Vue Migration and Shelf Batch Spacing Correction](../spec.md)

## Acceptance criteria

- [x] Both Drop Shelf horizontal collection insets are 7 logical pixels; measured left-border and right-scrollbar-channel clearances are equal.
- [x] Insets remain symmetric with and without overflow; when visible, the rounded 4-pixel painted thumb starts at the content-side edge of its 8-pixel hit channel, keeping the visible right gap equal to the 7-pixel left gap.
- [x] The first Shelf Batch is separated from the top collection viewport edge by a fixed 8-pixel boundary outside the actual scrollport.
- [x] Fully and partially visible Shelf Batches cannot paint into the fixed top boundary at the start, middle, or maximum-scroll position.
- [x] The final Shelf Batch has 8 pixels of bottom clearance inside the scroll range and remains fully reachable at maximum scroll without extra blank range.
- [x] Short collections remain top-aligned; Shelf Batches are not centered or distributed through unused height.
- [x] The grid retains a 164-pixel minimum card width, responsive auto-fill columns, 4-pixel card gaps, and horizontal-overflow suppression.
- [x] Responsive column capacity follows symmetric insets without width-compensation logic or reducing the 164-pixel minimum card width.
- [x] Ordinary collections with at most fifty Shelf Batches and virtualized collections with more than fifty Shelf Batches use the same visual spacing contract.
- [x] Virtual coordinates, mounted-entry bounds, total height, focus retention, keyboard reveal, and scroll positions remain zero-based against the real inner scrollport.
- [x] Usable virtual width and horizontal origin continue to derive from computed scrollport padding without a Drop Shelf-specific offset parameter.
- [x] Drop Shelf resizing, one-, two-, and three-column layouts, automatic growth, and the existing transparent border geometry remain usable.
- [x] Empty Drop Shelf presentation, Clear Temporary Items, confirmation surfaces, status presentation, Shelf Batch actions, and card internals remain unchanged.
- [ ] Drag-in, Shelf Item drag-out, Shelf Batch drag-out, pinning, removal, management, and Copy-only source-file safety remain unchanged.
- [x] Native scrollbar hit-channel width, painted-thumb width, semantic colors, hover/active states, transparent track, and Windows Contrast mappings remain unchanged; only the approved paint alignment changes.
- [x] Complete Vue single-file-component typechecking and production bundling pass after the spacing correction.
- [x] Real Windows WebView2 inspection confirms the approved insets and clearances with fitting and overflowing content at minimum and representative multi-column widths.
- [x] No Manage Items, Settings, Edge Rail, shared virtualization algorithm, release metadata, installer, or unrelated styling change is included.
- [x] No Git commit is created without separate explicit user authorization.
- [x] The user-approved runtime amendment is implemented: Drop Shelf minimum/default/empty-reset width is 191 logical pixels, while minimum height remains 180 pixels; native validation, restored-width bounding, initial window sizing, native resize limits, behavioral tests, and the living Product Specification agree.
- [x] The approved 191-pixel window minimum remains a conservative floor; the 164-pixel card and symmetric 7-pixel clearances fit even with an 8-pixel scrollbar, without horizontal overflow or clipping.
- [x] Automatic-growth column capacity matches the no-scroll grid thresholds (two columns at 348 pixels, three at 516).

## Testing seam

Use the actual Windows Resident Instance and real WebView2 Drop Shelf. Exercise fitting, overflowing, ordinary, and more-than-fifty-batch collections at start, middle, and maximum scroll. Assert equal 7-pixel left-border and right-to-painted-thumb clearances with computed geometry and screenshot pixel inspection; a channel-only assertion is insufficient. Retain the 8-pixel hit channel, 4-pixel painted thumb, rounded corners, vertical boundaries, columns, focus reveal, resizing, and reachability. No permanent visual-test framework.

## Demo path

Open the populated Drop Shelf at minimum width and representative two- and three-column widths. Show content that fits without a scrollbar, content that overflows, a partially visible Shelf Batch beneath the fixed top boundary, maximum scroll with the final Shelf Batch fully visible, and a virtualized collection with more than fifty Shelf Batches. Resize the Drop Shelf and exercise keyboard focus, drag, Pin, Remove, Manage Items, and Clear without layout regressions.

## Non-goals

Expanded Edge Rail spacing or height; Manage Items or Settings spacing; collapsed Rail Handle changes; scrollbar redesign beyond the approved paint alignment; card density changes; shared virtualization refactoring; Shelf state or persistence changes; release versioning; NSIS packaging; or Git commit creation.

## Earlier verification — superseded horizontal gutter

Source implementation complete with the user-approved 191-pixel minimum-width amendment. Real WebView2 measured 7/10-pixel horizontal insets, fixed 8-pixel top clearance, 8-pixel maximum-scroll bottom clearance, one/two/three columns at 191/359/527 widths, and bounded 65-batch virtualization with final batch reachable. Tab and Shift+Tab revealed virtual controls correctly. Native WM_SIZING clamped 180 to 191 pixels without changing height. Width validation regression failed before and passed after the fix; complete native suite passed 102/102.

Physical Explorer/OLE Copy/cancel acceptance remains unchecked; native regression coverage and unchanged drag code are not fresh physical interaction evidence. See [shared verification evidence and cleanup limits](../verification.md), including Windows theme-cache and startup-registry restoration limits. No commit or release packaging. Remaining work is human acceptance, not deferred source implementation.

## Answer

The user-approved horizontal-balance correction is implemented and verified on rebuilt native bytes: left-border and right-scrollbar-channel clearances both measure 7 pixels. Ordinary grids passed gutter checks at widths 191, 356, and 524; 65-batch virtualized grids also passed start/middle/end scroll at all three widths, with no horizontal overflow and the final batch reachable with 8-pixel bottom clearance. The permanent native column-boundary regression failed before the budget change and passed afterward; full native suite passed 102/102. Minimum/default/reset width stays 191 pixels. See the dated [symmetric-gutter verification](../verification.md) for source hash and exact cleanup evidence. Broader physical-interaction acceptance remains human-owned; no commit.


### Painted-thumb follow-up — 2026-10-15

Channel-only acceptance was insufficient: the thumb's transparent left border added 2 pixels to the visible gap. The user approved moving only the 4-pixel paint to the content edge of the unchanged 8-pixel channel. Rebuilt screenshot pixels place the paint at columns 182–185 against card edge 175, proving a 7-pixel visible gap. Fitting content and 65-batch start/middle/end checks at widths 191/356/524 passed. No collection or native geometry changed. See the latest [painted-thumb evidence and acceptance limits](../verification.md); physical hover/drag and fractional DPI remain unclaimed.

