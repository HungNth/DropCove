# 03: Rail Management and Copy Drag-Out

Status: resolved
Type: task
Version: 0.0.5
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by user on 2026-10-13

## What to build

Complete the expanded Edge Rail as a useful no-activate management surface: item and batch Copy drag-out, rail-constrained Manage Items, pinning/removal, Clear Temporary Items and Open Shelf, all routed through the existing Core Shelf State lifecycle.

## Acceptance criteria

- [x] Expanded rail exposes summaries, whole-batch drag, Bulk Pinning, Manage Items, Remove Batch, Clear Temporary and Open Shelf.
- [x] Single-item batches expose direct drag, Pin/Unpin and Remove Batch without redundant Manage Items.
- [x] Rail Manage Items is width-bounded (<=280 px) with vertical scroller and canonical actions.
- [x] Pointer opening and management remain no-activate; no global keyboard hook is added.
- [x] Manage Items and Clear confirmation hold the rail expanded; explicit close closes transients.
- [x] Item drag keeps Manage Items open; whole-batch drag closes it; active item drag dismissal is guarded.
- [x] Item/batch drag requests Windows Copy only using existing preparation/completion boundaries.
- [x] Missing removed before drag; Unavailable excluded; mixed confirmation and cancellation preserve documented state.
- [x] Temporary drag consumes participants; Pinned remains; cancellation and rejection retain references.
- [x] Pin/Unpin, tri-state Bulk Pinning, Remove Item, Remove Batch and confirmed Clear preserve order, identities and duplicate occurrences.
- [x] Clear confirmation reports all-Pinned no-op, mixed and all-Temporary outcomes; unconfirmed Clear never mutates state.
- [x] Open Shelf follows docked monitor, closes rail transients, transitions to Visible and focuses Drop Shelf.
- [x] Final item removal hides rail immediately and resets Shelf sizing.
- [x] Mutation and drag failures leave state coherent and expose actionable status/warning.

## Testing seam

Reuse real temporary Shelf storage for item lifecycle and deterministic native drag outcomes. Rail tests cover transient hold-open/closure around committed state rather than re-testing copied command wiring. Native Windows smoke is required for OLE item/batch Copy, cancellation/rejection, focus preservation, Manage Items anchoring and Clear confirmation behavior on the actual rail surface.

## Demo path

Populate single- and multi-item batches. Pin/unpin, remove an item and batch, use tri-state Bulk Pinning, cancel then confirm Clear, drag a Temporary item successfully, drag a Pinned item, cancel a drag and exercise mixed unavailable input. Keep Manage Items open during item drag, then choose Open Shelf and verify placement/focus.

## Non-goals

Fullscreen policy, display fallback reflow finalization, Shake-to-open and release packaging are out of scope.


## Answer

Ticket 03 resolved. Full rail management, item/batch Copy drag-out, no-activate Manage Items flyout, non-focus-stealing confirmations, and transient hold-open/outside-release lifecycle are implemented and verified. Version metadata remains 0.0.4; no commit created.

### Exercised evidence

- Fresh full native suite passed **74/74** tests without filtering. Real-store coordinator test verified independent hold reasons, multi-reason retention, whole-batch drag closing Manage Items, OLE drag hold suppression, natural 300 ms delayed collapse upon hold release while pointer is outside, and immediate rail hiding upon final item removal.
- vue-tsc/Vite and cargo-xwin release builds passed. Pinned idempotent executable: 13,171,200 bytes, SHA-256 `b34643597721b18cbe8f484512555ca045a14472e0b3da25a21a4ba536b5dcf7`.
- Native smoke on PID 2268 verified:
  - Natural 300 ms collapse resumes upon transient hold release with pointer outside at `(1500, 500)` without synthetic pointer event.
  - Outbound batch drag prepared for Batch 0 (`available: 2, unavailable: 0`, token `7b5db167...`); cancel preserved batches 2 -> 2. Mixed batch drag triggered confirmation (`available: 1, unavailable: 1`); cancel preserved batches 3 -> 3.
  - Outbound Copy drag-out consumed Temporary items (batch 2 -> 1) and strictly retained Pinned items (`isPinned: true`).
  - Tri-state Bulk Pinning toggled All Pinned (`[True, True]`), All Unpinned (`[False, False]`), and Indeterminate (`[False, True]`).
  - `remove_item` decremented batch items 2 -> 1; `remove_batch` removed entire mixed batch 3 -> 2.
  - Clear Temporary verified: mixed state removed Temporary and retained Pinned; all-Pinned produced exact no-op; all-Temporary emptied shelf (batches 0) and immediately hid the rail.
  - `open_shelf_from_rail` hid rail and positioned Drop Shelf at work area rect `[1724, 391, 1920, 580]`.
  - Subclass reopen bug identified and repaired: `RemoveWindowSubclass` before `SetWindowSubclass` ensures idempotent no-activate flyout initialization.
  - OS foreground focus remained locked to external probe (`HWND 8720160`) across all interactions.
- Clean native Exit returned 0; profile and WebView directories cleaned up; absent Run entry and five unrelated entries verified untouched. Reusable binary and harness scripts preserved at `C:\DropCove_005_RailSmoke\Bin`.

### Limits

- Tested on host's 96-DPI displays; mixed-DPI remains covered by deterministic geometry tests. Fullscreen occlusion policy belongs to Ticket 04.
## Comments
