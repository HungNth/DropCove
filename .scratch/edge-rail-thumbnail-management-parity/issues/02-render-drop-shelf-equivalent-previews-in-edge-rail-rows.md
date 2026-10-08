# 02: Render Drop Shelf-equivalent previews in Edge Rail rows

**What to build:** Make each expanded Edge Rail Shelf Batch row use the shared Drop Shelf preview and text hierarchy while preserving the Edge Rail's compact geometry, virtualization, drag behavior, no-activate window contract, and bounded visual lifetime.

**Blocked by:** 01: Extract shared shelf visual presentation primitives.

**Status:** needs-info

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] Every expanded Edge Rail row reserves a 42 by 42 logical-pixel preview area without changing the fixed 64-pixel row height or 280-pixel expanded width.
- [ ] A single-item Shelf Batch shows one eligible Windows Shell thumbnail or native file/folder icon fallback.
- [ ] A multi-item Shelf Batch shows the shared stacked preview for at most its first three Shelf Items in batch order, regardless of total item count.
- [ ] Thumbnail failure, Missing references, Unavailable references, non-image files, and folders retain a useful native icon or fallback glyph rather than a blank visual.
- [ ] Single-item title/subtitle content matches the Drop Shelf projection: Shelf Item name plus type and pinned lifecycle when relevant.
- [ ] Multi-item title/subtitle content matches the Drop Shelf projection: `N items`, the first two item names, and pinned count when relevant.
- [ ] Long and localized text trims without wrapping, widening the rail, overlapping future action controls, or participating in native-window measurement.
- [ ] Row realization starts thumbnail work only for virtualized rows that are actually prepared; at most three preview requests exist per realized row.
- [ ] Collapsing or hiding the Edge Rail cancels in-flight row visual work, clears row ImageSource references, and retains no thumbnails in the plain Rail Handle.
- [ ] Re-expanding the Edge Rail reloads visible visuals correctly through the shared provider/coordinator rather than depending on hidden retained images.
- [ ] Whole-Shelf-Batch drag remains available from the non-action row surface with current Copy-only lifecycle behavior and no foreground activation.
- [ ] Existing geometry remains unchanged: current 16 by 280 Rail Handle source contract, 280-pixel expanded width, 64-pixel rows, adaptive heights, centered placement, grow-only live behavior, and 640-pixel cap.
- [ ] Existing visual-coordinator and sizing-policy tests remain green; no permanent test asserts XAML values or realized visual-tree internals.
- [ ] A running Edge Rail smoke demonstrates single image, non-image, folder, two-item, three-item, and larger Shelf Batches on representative Left and Right placement without row clipping or blank previews.

## Testing seam

Use the shared visual coordinator for thumbnail/fallback/cancellation policy, the existing sizing policy for geometry invariants, and the running Edge Rail for actual WinUI realization, no-activate behavior, and ImageSource release/reload observation.

## Demo path

Dismiss a populated Drop Shelf into the Edge Rail, expand it, and show representative single and multi-item rows. Collapse and re-expand the rail to demonstrate that previews reload while row dimensions, drag, focus preservation, and adaptive sizing remain unchanged.

## Comments

- Shared 42-pixel batch preview and title/subtitle projection are wired into the virtualized Edge Rail. Realized rows use the shared coordinator with at most three requests; collapse/hide detach list content, cancel requests and clear image ownership.
- Release build passes with zero warnings/errors; full regression suite passes 226 tests. Both review axes report no confirmed new code defects.
- Acceptance is blocked, not complete: isolated left-edge rail remains 16 by 280 despite the cursor reaching an input HWND belonging to the test process and native mouse messages sent along its ancestor route. Expanded preview rendering, collapse/reload and real drag were not observed. The existing product documentation also records a baseline hover-qualification failure; the current cause remains unresolved.
- Tickets 03 through 06 have not started. The approved strict dependency chain is unchanged. Resume requires a reliable interactive verification route or explicit approval to investigate/fix the existing hover blocker before completing this ticket.
- Evidence: `artifacts/parity-ticket01-02-progress.json`. No commit or same-payload residency acceptance is claimed; Ticket 19 remains independent.
