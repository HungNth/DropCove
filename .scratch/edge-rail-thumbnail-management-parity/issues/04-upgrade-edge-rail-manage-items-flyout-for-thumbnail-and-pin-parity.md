# 04: Upgrade the Edge Rail Manage Items flyout for thumbnail and pin parity

**What to build:** Replace the current fixed 260-pixel-wide drag-only Edge Rail flyout with the newly confirmed bounded Manage Items experience, giving multi-item Shelf Batches Drop Shelf-equivalent visual inspection, Bulk Pinning, per-item Pin/Unpin, keyboard access, and direct drag while keeping the rail no-activate.

**Blocked by:** 03: Manage Shelf Batches directly from Edge Rail rows.

**Status:** ready-for-agent

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] The previous fixed 260-logical-pixel flyout width is superseded by a new content-sized width bounded from 320 through 480 logical pixels.
- [ ] Flyout height remains bounded at 480 logical pixels, horizontal scrolling is disabled, and item content scrolls vertically inside the remaining height.
- [ ] The flyout opens toward the desktop for Left and Right rails and clamps inside the selected monitor work area without activating the full Drop Shelf.
- [ ] A fixed, non-scrolling header shows `X of N pinned` and a tri-state Bulk Pinning control whose semantics and automation state match the row and Drop Shelf.
- [ ] Each realized item row uses the shared 24-pixel visual presentation and displays thumbnail/native icon, name, Path Reference, availability, and Pinned or Temporary lifecycle.
- [ ] Each item row exposes Pin/Unpin and remains directly draggable with existing Missing, Unavailable, partial-availability, Copy-only, cancellation, rejection, and Successful Drag-Out semantics.
- [ ] Available, Unavailable, and confirmed-Missing references still present in the Shelf Batch participate in individual and Bulk Pinning without changing availability.
- [ ] Full item projections and visual requests are created only when Manage Items opens and only for rows needed by the virtualized/scrolling presentation.
- [ ] Closing or replacing the flyout, hiding the Edge Rail, or removing its owner cancels in-flight item visual work, clears ImageSource references, and releases item projections.
- [ ] Pin/Unpin and Bulk Pinning commit through the existing manager before refresh, keep the flyout open, preserve focus on or near the action, and report atomic failure through the existing resident warning path.
- [ ] Keyboard-opened Manage Items focuses the fixed Bulk Pinning control; pointer opening does not force focus.
- [ ] Enter and Space activate focused controls. Tab and Shift+Tab traverse the header and every item Pin/Unpin action in both directions, realizing only the target off-screen row when needed.
- [ ] Escape closes Manage Items and restores focus to its originating control without dismissing or activating the full Drop Shelf.
- [ ] An open flyout or active item drag keeps the Edge Rail expanded; normal 300 ms collapse timing resumes after the hold ends and the pointer is outside.
- [ ] Accessible names describe the target action and counts, and thumbnail/native-icon automation context remains meaningful in Light, Dark, and High Contrast.
- [ ] Existing visual-coordinator, manager, persistence, and drag tests remain authoritative; installed smoke proves actual width bounds, placement, scrolling, focus traversal, no-activate behavior, and item drag.

## Testing seam

Use the shared visual coordinator for item visual policy, DropShelfManager with real SQLite for Pin/Unpin and Bulk Pinning durability, existing drag behavior for lifecycle outcomes, and installed WinUI smoke for flyout geometry, virtualization, focus, accessibility, and no-activate interaction.

## Demo path

Open Manage Items for a long mixed Shelf Batch from both Right and Left rails. Show the fixed pin summary, image and non-image visuals, path/availability/lifecycle metadata, per-item Pin/Unpin, Bulk Pinning, scrolling to the last item, keyboard traversal, Escape focus restoration, and one accepted/canceled item drag.

## Comments

- Manage Items flyout upgraded: programmatically bounded width (320-480px) and height (max 480px), desktop-facing on both Left and Right rails.
- Fixed header displays `X of N pinned` and Bulk Pinning toggle.
- Item list uses `ShelfItemProjectionList` for lazy realization: visuals and projections created only when rows scroll into view or keyboard traverses to them; all visual requests and ImageSource references cleared on flyout dismissal.
- Keyboard navigation verified: Enter opens and focuses Bulk Pinning; Tab/Shift+Tab traverses header and all items wrapping in both directions; Space activates controls; Escape closes flyout and restores focus to Manage button.
- Focus retention verified: Enter and Space on Bulk Pinning and per-item Pin retain focus on the activating control across manager-first durable mutation.
- Normal 300 ms collapse timing resumes after flyout closes.
- Evidence: `artifacts/parity-feature-qualification-record.json`.
