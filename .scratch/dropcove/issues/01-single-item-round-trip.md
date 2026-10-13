# Single-item Drop Shelf round trip

Status: resolved
Type: task
Version: 0.0.2
Parent: `../spec.md`
Blocked by: none

## Required behavior

Deliver one complete user workflow through the real Windows surface:

- Resolve the per-user profile directory and initialize the canonical SQLite shelf schema with foreign keys, WAL, serialized access, transactional mutations, and an integrity check on open. Refuse unsafe state and report failure; automatic backup-and-reset recovery remains deferred.
- Introduce the single Core Shelf State owner and the adapter boundaries needed for persistence, Windows storage drag, and presentation.
- Accept a one-item Windows storage payload with a non-empty filesystem path as one Temporary Shelf Item inside one Shelf Batch.
- Show the floating Drop Shelf and a direct-draggable single-item card using the stored display name and file/folder flag.
- Restore the same batch and item after process restart.
- Offer copy-only item drag-out. Remove the Temporary Item only after the destination returns accepted `Copy`; retain it after rejection, cancellation, resolution failure, or pre-acceptance failure.
- Support Remove Batch without modifying the referenced source object.
- Preserve prior visible and durable state when persistence fails.

## Acceptance criteria

1. Dropping one file or folder creates one persisted batch and one visible card.
2. Restart restores the same identifier, path, metadata, ordering, and Temporary state.
3. An accepted copy-only drag removes the item and empty batch durably.
4. A canceled, rejected, or failed drag retains the item through restart.
5. Remove Batch removes only the DropCove reference; the source still exists.
6. A forced persistence failure produces no visible partial add, removal, or drag completion.
7. A failed integrity check does not expose shelf state as valid and reports the startup failure without modifying the database.

## Testing seam

- Core state is exercised against an isolated temporary SQLite database with adapter-controlled drag outcomes.
- Permanent checks cover durable accepted-drop, restart restore, accepted-copy consumption, cancellation retention, rollback on failed persistence, and failed integrity-check handling.
- The real Windows storage-item drag adapter and surface are proven by the demo path rather than mock-echo tests.

## Demo path

1. Launch DropCove and drop one file into the Drop Shelf.
2. Restart and observe the restored card.
3. Cancel one drag-out and observe retention.
4. Drag the item to Explorer with `Copy` accepted and observe the empty shelf.
5. Repeat with a folder, use Remove Batch, and confirm the folder remains on disk.

## Comments

- Approved as part of version `0.0.2`; single-item round trip is implemented.
- Native Windows core tests: 5 passed, covering restart restore, accepted Copy, cancellation retention, persistence rollback, and invalid-database refusal without byte changes.
- `pnpm build`, `cargo xwin check --target x86_64-pc-windows-msvc`, and `cargo xwin build --target x86_64-pc-windows-msvc` passed.
- Live Windows smoke: native storage-item drag-in, process restart/restore, accepted copy-only drag to a native same-integrity target, cancellation retention outside eligible targets, and folder Remove Batch with the source folder preserved.
- `cargo xwin test` defaults to Wine even on this Windows host; tests were compiled with `--no-run` and the produced `.exe` was executed directly. Missing SDK PDB linker warnings did not prevent execution.
- Ticket 02 may proceed. The release remains incomplete until tickets 02–04 are resolved.
