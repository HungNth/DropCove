# 05: Cross-origin Item Selection lifecycle integration

**What to build:** Integrate mixed item-set transfer and accessible Item Selection into one predictable Manage Items lifecycle from both Drop Shelf and Edge Rail origins. Escape, popup replacement, one-item survival, focus restoration, owner hide, resize, whole-batch drag, active native drag, and delayed close all preserve the recorded transfer and discard presentation selection at the approved boundaries without stealing Edge Rail foreground focus.

**Blocked by:** 03: Mixed availability and ownership-safe Item Selection drag; 04: Accessible Manage Items multiselectable grid.

**Status:** resolved

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** Tickets 03 and 04, the existing Manage Items owner/origin state, Edge Rail transient hold and no-activate behavior, Drop Shelf focus recovery, native drag-session guard, shelf resize lifecycle, and whole-batch drag coordination.

**Testing seam:** One real Windows Resident Instance exercising the shared popup from both origins while observing foreground/focus ownership, ordinary and virtualized batches, mixed availability confirmation, active OLE drag, hide/close/resize, one-item survival, and final-batch disappearance.

**Demo path:** Open Manage Items by keyboard from the Drop Shelf and by pointer from the Edge Rail. Exercise confirmation Escape, native-drag Escape, selection Escape, popup close, active-drag pending close, owner hide, replacement by another batch, reduction to one item, final-item consumption, and focus or no-focus restoration from each origin.

- [x] `Escape` cancels an active item availability confirmation and prepared token before any selection or popup action.
- [x] During active native item drag, `Escape` cancels the drag before a later press can clear Item Selection.
- [x] With no active confirmation or drag, `Escape` clears non-empty Item Selection before a later press closes Manage Items.
- [x] Explicit close, opening another batch, owner-driven popup closure, and restart clear Item Selection rather than transferring it.
- [x] The popup remains open while its Shelf Batch exists, including when exactly one Shelf Item remains.
- [x] The popup closes when the batch disappears, the shelf hides, shelf resizing starts, or whole-batch/Batch Selection drag starts.
- [x] A close or hide request during active native drag becomes pending; the recorded transfer completes independently and the popup closes afterward.
- [x] Rejected or canceled completion after a pending close does not restore discarded Item Selection.
- [x] Successful Copy clears Item Selection even when Pinned or Unavailable items survive in the popup.
- [x] Drop Shelf keyboard origin restores focus to the Manage Items control when it exists, the one-item card drag region when that control disappears, or the approved same-index/previous/More fallback when the batch disappears.
- [x] Pointer-opened Edge Rail origin never receives keyboard focus, live-announcement ownership, or foreground activation during selection, confirmation, drag, or close.
- [x] Edge Rail transient hold and collapse timing survive confirmation, cancel, active drag, and popup close without leaving a stale hold.
- [x] Popup replacement, Shelf mutation, or origin transition leaves no stale token, anchor, selected identity, focused identity, or virtual-row reference.
- [x] Ordinary and virtualized item collections follow identical lifecycle and focus rules.
- [x] Existing Shelf sizing, fullscreen, Shake, Shell Visual, Clear Temporary Items, and Copy-only contracts remain unchanged.

## Evidence — 2026-10-26

- Frontend Escape-order and mixed-confirmation behavior tests passed within the **43/43** suite. Native lifecycle/sizing/fullscreen/Shake regressions passed within **185/185** native tests.
- Exact installed Rail popup reported origin `edge-rail`, keyboard activation `false`, and **zero** selection live regions. Native foreground HWND remained the owned external destination through pointer opening, selection, mixed confirmation, direct confirmation Copy, cancellation, and close.
- Exact installed pending-close smoke held the native left-button drag, requested popup close through the real command, observed the popup remain visible during OLE, canceled the drag, then observed the popup disappear without changing the external foreground owner.
- Closing management released the Rail transient hold and returned the expanded **280-pixel** Rail to its **16-pixel** handle. One-item survivors stayed open; explicit close returned focus to the one-item card, and final batch removal recovered the surviving parent batch.
- Restart restored **65** exact identities/paths/Pin states and reopened with zero selected rows. Final installed ordinary and virtualized surfaces preserved the same selection/focus contracts.
- Shared final release evidence: [Ticket 06](06-release-dropcove-0.0.26.md).
