# 03: Bulk Pinning and keyboard management lifecycle

Status: ready-for-agent
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can manage the entire batch through tri-state Bulk Pinning on the card and management header, and perform the complete Manage Items workflow using predictable keyboard/focus behavior. Extend the already-working pointer/item management slice rather than recreating its popup adapter.

## Acceptance criteria

- [ ] Bulk state is derived from item states only: all Temporary is Off, all Pinned is On, and mixed is Indeterminate; no batch pin field is introduced.
- [ ] Off or Indeterminate activation Pins every retained item; On activation makes every retained item Temporary in one durable operation.
- [ ] Bulk Pinning includes retained Available, Missing, and Unavailable items without reclassifying, reordering, resizing, or changing source objects.
- [ ] The multi-item card and management header expose accessible tri-state controls; the header also shows an accurate X/N pinned summary and explicit Close.
- [ ] Enter/Space opening focuses Bulk Pinning; pointer opening still does not force keyboard focus.
- [ ] Tab/Shift+Tab cycle through Bulk Pinning, Close, and every row's Pin/Unpin and Remove controls in the canonical order.
- [ ] Explicit Close restores focus to the originating Manage Items control when it still exists.
- [ ] Opening another batch replaces the view with the correct new origin and data, without restoring obsolete focus as an intermediate action.
- [ ] During an active item drag, Escape cancels the native drag first; a later Escape can close Manage Items. The same Escape event does not dismiss multiple surfaces.
- [ ] Whole-batch drag closes Manage Items before native drag begins; item drag preserves it while the batch remains.
- [ ] Owner hiding/removal and batch removal close the management surface and release its projection.
- [ ] Failure during Bulk Pinning preserves the previous visible/durable tri-state and reports the failure.

## Testing seam

Core checks exercise all three bulk states, atomic transitions, availability-independent membership, pin state changes around prepared drag completion, and rollback. Native smoke exercises pointer versus keyboard opening, traversal, explicit-close focus restoration, Escape ordering, replacement, and item/batch drag lifecycle.

## Demo path

Open a multi-item batch by keyboard, Pin one row to create mixed state, activate Bulk Pinning to Pin all, then activate On to make all Temporary. Traverse in both directions, cancel an item drag with Escape, close with a later Escape, and observe restored origin focus.

## Non-goals

Resident shelf dismissal and EdgeDocked transitions remain the later roadmap versions. Ticket 05 adds closure at actual native resize start; do not substitute a programmatic-size notification or process exit for that behavior.

## Comments
