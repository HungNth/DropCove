# 04: Confirmed Clear Temporary Items across the Shelf

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can deliberately clear Temporary Items across all batches while retaining Pinned Items. Deliver the Shelf footer action, real confirmation, atomic lifecycle mutation, and management-view synchronization as one workflow.

## Acceptance criteria

- [x] Clear Temporary Items is present only when Temporary content exists.
- [x] Confirmation is required; canceling changes no content, order, or sizing intent.
- [x] Confirmation removes all Temporary references across all batches, including unavailable ones, while retaining every Pinned Item.
- [x] Empty batches are pruned; surviving batches and items retain their order and identities.
- [x] Source files/folders are never deleted, moved, or overwritten.
- [x] An open Manage Items view refreshes if its batch survives and closes if its batch disappears.
- [x] Persistence failure leaves prior visible and durable content unchanged and reports the failure.
- [x] Removing the final reference clears retained sizing state and preserves the existing empty-size reset; later manual-height behavior uses the same final-item invariant.
- [x] Existing Remove Batch is verified as a regression, not rewritten as a new feature.

## Testing seam

Use real SQLite through Core Shelf State for mixed pinned/temporary batches, overlapping path occurrences, all-Pinned/no-Temporary state, all-Temporary final removal, pruning, order preservation, and failed clear rollback. Smoke confirmation, footer visibility, management synchronization, and source-object safety through the Windows surface.

## Demo path

Create two batches, Pin one item in each through Manage Items, cancel Clear once, then confirm it. Observe only the Pinned Items remain. Explicitly remove the survivors and verify final sizing reset and untouched sources.

## Non-goals

Bulk Pinning is not a true prerequisite: mixed lifecycle state can be created with ticket 02's individual controls. No Edge Rail flyout or resident dismissal behavior is added here.

## Comments

- Implemented serialized `ShelfState::clear_temporary` and async `clear_temporary` IPC using existing `publish_shelf`. One SQLite transaction deletes Temporary references across all batches without availability checks, prunes empty batches, resets sizing only when no references remain, constructs the snapshot before commit, then updates in-memory membership/cache.
- Shelf footer derives visibility from actual Temporary content. A focused native `<dialog>` separates request, cancel and confirm; Cancel is the default focus, native Escape cancels, and only Confirm calls the mutation. Native dialog is viewport-bounded and lifecycle-cleaned; no new dependency, state owner or generic popup framework.
- Added four real-SQLite regressions: mixed batches/overlapping paths with identities/order/source safety; all-Pinned no-op; all-Temporary final sizing reset; and failure after an earlier row deletion using SQLite RAISE(FAIL), proving the outer transaction restores the entire visible/reopened state.
- `pnpm build` passed. cargo-xwin compiled Windows-targeted tests with `--no-run`; direct native execution reported **27 passed, 0 failed**. Embedded Windows x64 release build with `tauri/custom-protocol` passed; existing external CRT PDB warnings were non-fatal.
- Actual embedded app smoke: created two mixed batches, pinned one row in each through Manage Items, included a folder, a Missing local reference, an Unavailable drive reference, and an overlapping path occurrence. Footer appeared only with Temporary content; confirmation defaulted to Cancel. Both pointer Cancel and Escape preserved the exact core snapshot, including order and sizing.
- Injected a real SQLite delete trigger to fail partway through Clear: confirmation reported the persistence error and preserved all references on Shelf and the open management surface. After removing the owned trigger, confirmed Clear retained only the exact Pinned IDs in their original batch order, preserved live sizing and source bytes/folder contents, refreshed the surviving popup to one row, and hid the all-Pinned footer.
- Unpinned the managed survivor and cleared it: its now-empty batch and management surface disappeared while the other Pinned batch remained. Existing Remove Batch removed the last Pinned batch without touching sources. A separate all-Temporary final Clear after retaining a 348-pixel preferred width reset the Shelf to 180 × 180; this reset survived process restart.
- A second post-Clear restart restored the exact Pinned survivor identity/state and kept the all-Pinned footer hidden. Native screenshots verified the confirmation and final empty surface; no installer-release acceptance is inferred from this development slice.
- Closed managed CDP tabs and the known owned app process, released SQLite handles, and removed the owned profile/fixtures/WebView data. A transient WebView log sharing lock delayed cleanup; the subsequent removal succeeded and the final owned-directory existence check was false. No test script/scaffold retained; ambient user profiles/recovery data were not used.
- No commit created for this ticket. Version metadata remains 0.0.2 until release ticket 06; native manual-height intent remains ticket 05.
