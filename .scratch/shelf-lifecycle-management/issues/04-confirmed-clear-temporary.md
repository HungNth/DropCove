# 04: Confirmed Clear Temporary Items across the Shelf

Status: ready-for-agent
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can deliberately clear Temporary Items across all batches while retaining Pinned Items. Deliver the Shelf footer action, real confirmation, atomic lifecycle mutation, and management-view synchronization as one workflow.

## Acceptance criteria

- [ ] Clear Temporary Items is present only when Temporary content exists.
- [ ] Confirmation is required; canceling changes no content, order, or sizing intent.
- [ ] Confirmation removes all Temporary references across all batches, including unavailable ones, while retaining every Pinned Item.
- [ ] Empty batches are pruned; surviving batches and items retain their order and identities.
- [ ] Source files/folders are never deleted, moved, or overwritten.
- [ ] An open Manage Items view refreshes if its batch survives and closes if its batch disappears.
- [ ] Persistence failure leaves prior visible and durable content unchanged and reports the failure.
- [ ] Removing the final reference clears retained sizing state and preserves the existing empty-size reset; later manual-height behavior uses the same final-item invariant.
- [ ] Existing Remove Batch is verified as a regression, not rewritten as a new feature.

## Testing seam

Use real SQLite through Core Shelf State for mixed pinned/temporary batches, overlapping path occurrences, all-Pinned/no-Temporary state, all-Temporary final removal, pruning, order preservation, and failed clear rollback. Smoke confirmation, footer visibility, management synchronization, and source-object safety through the Windows surface.

## Demo path

Create two batches, Pin one item in each through Manage Items, cancel Clear once, then confirm it. Observe only the Pinned Items remain. Explicitly remove the survivors and verify final sizing reset and untouched sources.

## Non-goals

Bulk Pinning is not a true prerequisite: mixed lifecycle state can be created with ticket 02's individual controls. No Edge Rail flyout or resident dismissal behavior is added here.

## Comments
