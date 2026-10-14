# 01: Recovery Backup and Startup Warning

**What to build:** When an existing Shelf database has a recoverable open, format, schema, or integrity failure, preserve it and every existing WAL/SHM sidecar as one Recovery Backup, create a clean database, start with an empty Shelf, and tell the user where the backup lives. Preserve normal versus Hidden Start behavior and fail clearly instead of continuing when evidence or clean persistence cannot be guaranteed.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.8 — Recovery and Distribution Hardening](../spec.md)

## Acceptance criteria

- [ ] A recoverable failed database creates one `shelf-corrupt-<UTC timestamp>[-N]` Recovery Backup directory containing the original database and every existing WAL/SHM sidecar under their original names.
- [ ] Missing WAL or SHM sidecars are accepted; every artifact that exists must be preserved before a clean database is created.
- [ ] A backup-name collision produces a deterministic numeric suffix without overwriting any prior Recovery Backup.
- [ ] If an artifact move fails, already moved artifacts are restored and startup aborts; if restoration is incomplete, the fatal error reports both source and Recovery Backup locations.
- [ ] Permission, lock, exhausted-storage, backup, rollback, or clean-database creation failure never falls back to an in-memory or apparently durable Shelf.
- [ ] Successful recovery creates a valid empty Shelf and performs no automatic salvage, import, schema guessing, or partial record reconstruction.
- [ ] Normal launch remains visible and Hidden Start remains hidden after recovery.
- [ ] A notification-area warning identifies that the Shelf started empty and includes the Recovery Backup path.
- [ ] If notification delivery fails, the same warning remains available on the Shelf status surface until shown.
- [ ] Clean databases and missing databases retain their existing startup behavior and do not create a Recovery Backup.
- [ ] Tests cover corrupt/non-database/schema-invalid input with no sidecars, WAL only, SHM only, and both sidecars, including byte-for-byte evidence preservation.

## Testing seam

Open the real Shelf persistence layer against an owned temporary profile, then continue through resident startup far enough to observe empty state, launch visibility, and warning delivery. Inject filesystem failures at the profile boundary; assert consumer-visible recovery or fatal behavior rather than private helper order.

## Demo path

Start DropCove with a deliberately corrupt Shelf database plus WAL/SHM fixtures. Observe the timestamped Recovery Backup, empty Shelf, and warning containing its directory. Repeat with a locked artifact and observe fatal startup with the original evidence intact.

## Non-goals

Automatic salvage, a Recovery Backup browser/import UI, settings-file backup, cloud backup, or changing ordinary Shelf persistence semantics.

## Answer

Implemented end-to-end Recovery Backup and Startup Warning for Ticket 01:

1. **Database Detection & Recovery Area Layout**:
   - Database format validation in `src-tauri/src/shelf.rs`: 16-byte magic check (`SQLite format 3\0`) without heap allocation; zero-length files preserved as clean newly created DBs; bounded-memory `io::copy` SHM staging (`create_new(true)`) before read-only (`SQLITE_OPEN_READ_ONLY`) validation prevents SQLite WAL index initialization from overwriting original `-shm` bytes; restores staged SHM on any validation failure and removes stage file on healthy validation.
   - Pure allowlist classification in `is_recoverable_error`: `DatabaseCorrupt`, `NotADatabase`, and `Unknown` (`SQLITE_ERROR`) with exact "no such table"/"no such column" messages; all lock, busy, readonly, perm, and I/O errors fail fatally.
   - Recovery backups created strictly under `<profile>/recovery/shelf-corrupt-<UTC timestamp>[-N]` via exclusive `fs::create_dir` (`AlreadyExists` increments numeric suffix) preserving original database, `-wal`, and `-shm` sidecars byte-for-byte using atomic filesystem moves.
   - Suffix collision resolution (`-1`, `-2`, ...) preserves prior recovery evidence without overwriting.
   - Atomic rollback on move failure: restores previously moved files to source and aborts; reports both source and recovery/stage locations if restoration is incomplete.
   - Clean database created fresh with an empty Shelf; no automatic salvage or reconstruction attempted.

2. **Startup Warning & Fallback Surface**:
   - In `src-tauri/src/lib.rs`: `ShelfState::open_or_recover` runs before `tauri::Builder::default()` (after `windows_instance::acquire`) so fatal startup persistence errors route directly through `fatal_startup_error` (exit code 1) rather than Tauri `.setup` panic.
   - Inside `.setup(...)` after `resident_tray::register`: `ShelfState::deliver_recovery_warning` publishes the notification-area warning via `resident::publish_warning`.
   - If notification delivery fails, the warning is retained on `ShelfState` and exposed consistently via `ShelfState::snapshot()` (`snapshot.warning`).
   - `get_shelf` returns `state.snapshot()` directly without a separate wrapper.
   - In `src/composables/useShelf.ts`: `pendingRecoveryWarning` retains the fallback warning across refreshes and Hidden Start until the user explicitly dismisses the matching recovery status (`acknowledge_shelf_warning`).
   - Preserves normal launch versus Hidden Start (`!launch.autostart` controls `show_shelf`).
   - Clean and missing databases create no recovery backup and emit no recovery warning.

3. **Native Behavioral Tests Status**:
   - Verified passing by parent native execution:
     - `corrupt_database_with_wal_and_shm_is_preserved_byte_for_byte_and_recovers_empty_shelf`
     - `recovery_accepts_missing_sidecars_and_preserves_only_existing` (no sidecars, WAL only, SHM only)
     - `recovery_backup_name_collision_resolves_deterministic_numeric_suffix_under_recovery_area`
     - `recovery_partial_move_failure_restores_moved_first_artifact` (Windows `share_mode` seam test)
     - `schema_invalid_database_is_recovered_and_preserves_evidence`
     - `unrecoverable_filesystem_failure_aborts_without_clean_shelf_or_data_loss`
   - Also verified: clean/missing database behavior; failed notification retains warning across repeated snapshots until dismissal; valid-header corrupt B-tree preserves original SHM bytes instead of SQLite's reconstructed WAL index.
   - Actual embedded native app: corrupt and valid-header corrupt databases recover to an empty Shelf with DB/WAL/SHM byte-for-byte preserved. Normal launch remained visible; recovered Hidden Start left Shelf and Rail hidden; cooperative shutdown exited cleanly.
   - Actual locked-database startup displayed the source/backup/error dialog and exited 1, rather than Tauri setup panic 101. Original evidence stayed unchanged.
   - Release-level notification-area and fallback-banner visual inspection remains part of ticket 05; this evidence does not claim an observed toast or forced-notifier-failure screenshot.
