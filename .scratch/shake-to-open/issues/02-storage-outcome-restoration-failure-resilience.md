# 02: Storage Outcome Restoration and Failure Resilience

Status: resolved
Type: task
Version: 0.0.6
Parent: [Specification](../spec.md)
Blocked by: 01
Implementation approval: approved by user on 2026-10-14

## What to build

Complete the Shake session around actual main-window OLE drag boundaries and every documented intake outcome. Ensure asynchronous Shelf persistence cannot race button release into the wrong presentation, and prove hook failure never weakens non-Shake workflows.

## Acceptance criteria

- [x] Main Drop Shelf DragEnter/DragLeave/Drop callbacks mark one Shake candidate/pending session without changing Edge Rail drop behavior.
- [x] A newly accepted Shelf Batch promotes the Shake presentation to normal Visible state at the near-pointer placement.
- [x] Duplicate-only input restores prior Hidden or EdgeDocked state without resize, reorder, scroll or durable mutation.
- [x] Unsupported input, rejected effect, cancellation, DragLeave and button release restore prior presentation.
- [x] Mixed supported/unsupported input follows existing Core Shelf State skipped-count behavior; a new batch keeps the Shelf visible.
- [x] Persistence or processing failure restores prior presentation and publishes one actionable warning.
- [x] Release/drop ordering is deterministic even though Shelf persistence runs off the UI thread.
- [x] Disabling the hook during an active transient session restores presentation before teardown; rollback can reinstate the prior runtime.
- [x] Forced hook installation failure leaves tray, hotkey, Drop Shelf, Edge Rail, settings retry, drag-in and drag-out usable.
- [x] Clean exit removes the hook and queued dispatch safely with no callbacks after owner teardown.

## Testing seam

Permanent coordinator tests drive release, drag enter/leave, drop pending, accepted, duplicate and failure events in both orderings. The real temporary Shelf store supplies accepted and duplicate outcomes. Windows smoke uses actual OLE drop and native pointer events; source inspection or DOM events are not substitutes.

## Demo path

Trigger from Hidden and EdgeDocked, then exercise: release with no drop, drag leave, unsupported data, duplicate-only drop, mixed drop with a new supported path, and accepted new batch. Observe exact restoration or Visible promotion. Force hook registration failure, verify warning and all existing access/storage paths, then retry enable successfully.

## Non-goals

Changing Core Shelf intake semantics, proving an external file drag before OLE reaches DropCove, release packaging, or later recovery/distribution work.

## Comments

## Answer

Resolved. Bound `StorageDropTarget` (`DragEnter`, `DragLeave`, `Drop`) on `main` to `ResidentState` Shake session IDs so asynchronous `accept_drop` completion deterministically promotes accepted new batches to `Visible` and restores prior `Hidden` or `EdgeDocked` state on duplicate-only, unsupported, rejected, canceled, or failed drops. Fixed `DragLeave` to update `shake_drag_inside(false)` instead of prematurely canceling an in-progress drag while the left mouse button remains held.

### Exercised evidence

- Coordinator tests (`shake_session_restores_hidden_or_rail_and_promotes_only_accepted_batches`) and full **82/82** native suite passed.
- Live Windows OLE `DoDragDrop` smoke (`8f1d76164de3feddb40166285328c35ccf5ec255ef446e547eff004f5855c917`) proved: accepted new file batch (`alpha.txt`, `DRAG_EFFECT:Copy`) promotes Shake Shelf to `Visible` at `[627, 406, 823, 595]` and persists batch in `shelf.db`; duplicate-only file drop from `EdgeDocked` restores Rail at `[1904, 376, 1920, 656]` with batch count unchanged (`1`); unsupported text OLE drop (`DRAG_EFFECT:None`) restores `EdgeDocked`; `DragLeave` + outside release restores `EdgeDocked`; clean exit (`0`) and ambient state verified.
- Final installed-artifact checks also proved Escape cancellation, owned-drag Shake exclusion, actual card whole-batch Copy, Pinned retention, owner-thread Manage Items setup, and fullscreen reevaluation after restoration. See [ticket 03](03-release-0.0.6.md#answer) for final hashes and precise failure-injection limits.
