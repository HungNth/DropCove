# 03: Managed File recovery and profile preservation

**What to build:** Make Managed File ownership survive resident restart and database failure without weakening DropCove's evidence-preserving recovery model. A recoverable Shelf failure preserves the database, sidecars, retained Managed Files, staging state, and pending cleanup together; DropCove then starts from clean active stores with an empty Shelf and an actionable warning. Incomplete preservation remains fatal rather than partially moving or deleting user-owned evidence.

**Blocked by:** 01: Managed-text Drop Shelf round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.23 — Managed File Lifecycle and Plain-Text Intake](../spec.md)

**Testing seam:** Core Shelf State startup/recovery with real temporary SQLite and filesystem artifacts, supported by an actual Windows Resident launched against owned profiles to observe warning, empty-Shelf, restart, and pending-cleanup behavior.

**Demo path:** Create retained and pending Managed Files, corrupt the Shelf database, restart DropCove, inspect the timestamped Recovery Backup for exact database/sidecar/managed bytes, observe the empty active Shelf warning, and prove a separate intact profile still restores normally.

- [x] Normal startup reconciles `Staged`, `Ready`, unowned staging, and Pending Managed File Deletion state before the first Shelf projection.
- [x] Ready Temporary and Pinned Managed Files restore with exact identity, bytes, filename, order, Pin state, sizing, and availability.
- [x] Pending deletion survives process restart and retries without restoring a removed Shelf Item.
- [x] A recoverable database open, format, integrity, or schema failure creates one timestamped Recovery Backup containing the failed database, every existing WAL/SHM sidecar, and the complete managed-storage root.
- [x] Backup filename collision handling retains deterministic numeric suffixes and never modifies an earlier Recovery Backup.
- [x] Missing WAL or SHM sidecars are accepted; only artifacts that existed are represented in the backup.
- [x] Managed files, staging bytes, directory names, security metadata where the filesystem preserves it, and pending owned paths remain associated with the preserved database evidence.
- [x] After complete preservation, DropCove creates a clean database and empty active managed-storage root, starts with an empty Shelf, and reports the Recovery Backup path.
- [x] DropCove does not automatically salvage, import, delete, or partially reuse preserved Managed Files.
- [x] Partial move failure restores every already-moved database, sidecar, staging, and managed artifact to its original source location before returning an error.
- [x] Failure to create the Recovery Backup, preserve every required artifact, restore a failed partial move, create the clean database, or create the empty active managed root is fatal.
- [x] Fatal recovery failure leaves original database and Managed File evidence intact and does not start with a misleading clean Shelf.
- [x] Recovery-warning notification failure retains the warning in the authoritative Shelf snapshot until it can be shown and acknowledged.
- [x] A clean or missing database creates no Recovery Backup and does not move healthy Managed Files.
- [x] An intact profile remains byte-for-byte and behaviorally unaffected by recovery tests performed against another owned profile.
- [x] Cooperative Shutdown closes SQLite and releases Managed File handles so restart and later update/uninstall workflows are not blocked.
- [x] Profile-scoped instance handling remains unchanged; explicit test profiles own independent database, managed, staging, recovery, and cleanup state.
- [x] Permanent tests extend existing recovery collision, missing-sidecar, partial-move rollback, fatal-failure, warning-fallback, and valid-header corruption prior art with Managed File artifacts.
- [x] Actual Windows startup smoke proves empty-Shelf recovery warning, retained backup path, healthy-profile isolation, Ready-item restore, and pending-cleanup retry.
- [x] Frontend/native regressions and cargo-xwin Windows build pass while release metadata remains `0.0.22`.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- Started after resolution of Tickets 01 and 02. Recovery backup preservation of both managed and staging roots, partial-move rollback, fatal recreation errors, warning fallback, and isolated profile verification are implemented and test-covered. No Git commit is authorized.

## Answer

Delivered and verified. When a Shelf database is corrupted, `open_or_recover_at` moves `shelf.db`, WAL/SHM sidecars, `managed/`, and `staging/` into one timestamped `recovery/shelf-corrupt-<timestamp>` backup folder, creates fresh empty active roots, and restores an empty Shelf with the backup warning. Partial move failures roll back completely. Reinstall, restart, and Cooperative Shutdown tests pass cleanly across independent profiles. All recovery regression suites passed. Release metadata remains `0.0.22`. No commit created.