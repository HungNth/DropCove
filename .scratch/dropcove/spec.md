# DropCove 0.0.2 — Core Drop Shelf round trip

Parent roadmap: [`docs/ROADMAP.md`](../../docs/ROADMAP.md), version `0.0.2`.
Canonical behavior: [`docs/SPECIFICATION.md`](../../docs/SPECIFICATION.md).
Canonical architecture: [`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md).

## Goal

Deliver the first usable DropCove workflow on Windows 11 x64: accept file and folder path references into a floating Drop Shelf, persist and restore their Shelf Batches, and offer copy-only item or whole-batch drag-out with safe Temporary Item consumption.

## Release contract

- Version remains `0.0.2` across the frontend package, Rust crate, Tauri configuration, executable metadata, and installer name.
- Work is complete only after the real Windows surface and drag paths are exercised and `DropCove-Setup-0.0.2.exe` is built.
- Only this version is implemented in the current delivery cycle. Later roadmap versions remain untouched.

## Required behavior

### Shelf state and persistence

- One Core Shelf State owner serializes all accepted-drop, removal, preparation, and completion mutations.
- Shelf Batches are newest-first; Shelf Items retain source order.
- New Shelf Items are Temporary.
- The application stores path references and metadata, never source contents.
- `%LOCALAPPDATA%\DropCove\shelf.db` uses foreign keys, WAL, transactional writes, an integrity check on open, and the canonical tables required by this slice; automatic corrupt-database backup and recreation remain deferred.
- Accepted content, ordering, and relevant sizing intent survive process restart.
- Persistence failure does not leave a user-visible partial mutation.

### Drag-in and presentation

- Windows storage-item drops accept entries with non-empty filesystem paths.
- Repeated paths within one payload are removed with ordinal case-insensitive comparison while preserving the first occurrence.
- Paths are not canonicalized and separators are not rewritten.
- An exact retained path-set match is a silent no-op regardless of order; partial overlap creates a new Shelf Batch.
- Mixed payloads accept supported entries and report the skipped count; unsupported-only payloads create no batch.
- The floating Drop Shelf renders a newest-first responsive grid with the documented padding, gap, minimum card width, and basic Automatic Shelf Growth tiers.
- A batch can be removed without affecting any source filesystem object.

### Drag-out and availability

- Item and whole-batch drag-out offer Windows `Copy` only.
- Before whole-batch drag, paths are classified as Available, Missing, or Unavailable.
- Missing references are durably removed before drag; Unavailable references remain and do not participate.
- A mixed Available/Unavailable batch requires confirmation before dragging only Available Items.
- Drag preparation records exact participants so later completion cannot consume unrelated items.
- Accepted `Copy` removes participating Temporary Items; rejection, cancellation, or pre-acceptance failure retains them.
- One-item resolution failure cancels the drag and retains its reference.
- Retaining an extra reference is preferred over incorrectly removing one.

## Out of scope

- Pinning, Bulk Pinning, Manage Items, item removal, and Clear Temporary Items.
- Full manual resize intent and native perimeter hit-testing beyond sizing needed by this slice.
- Tray, single-instance activation, global hotkey, startup registration, and Settings UI.
- Edge Rail, Shake-to-open, shell thumbnails, final theme/accessibility conformance, corruption recovery, and installer lifecycle hardening.

## Release acceptance

1. Drop files and folders, restart DropCove, and observe the same newest-first batches and source-order items.
2. Repeat an exact path set in another order and observe no persistence, ordering, status, scrolling, or sizing change.
3. Drag an item and a batch to a same-integrity destination; accepted `Copy` removes only participating Temporary Items.
4. Cancel or reject item and batch drags and observe retained references.
5. Exercise Available, Missing, Unavailable, mixed, and no-Available batch preparation outcomes.
6. Remove a batch and verify that source files and folders remain untouched.
7. Build, install, launch, and smoke `DropCove-Setup-0.0.2.exe` on Windows 11 x64.

## Demo path

1. Drop `A.png`, `B.png`, and one folder; observe one ordered three-item Shelf Batch.
2. Drop the same paths in another order; observe no change.
3. Restart DropCove; observe the restored batch.
4. Make one path Missing and one path Unavailable, then start whole-batch drag.
5. Confirm dragging only Available Items and complete with `Copy`.
6. Observe Missing cleanup, Unavailable retention, and consumption of only the Available Temporary participants.
