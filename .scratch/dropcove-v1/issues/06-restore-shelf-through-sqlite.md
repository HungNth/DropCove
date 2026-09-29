# 06: Restore the shelf through SQLite

**What to build:** Persist the complete confirmed shelf lifecycle so temporary and pinned Path References survive application and Windows restarts without storing source or thumbnail content.

**Blocked by:** 05: Drag and clean complete Shelf Batches.

**Status:** ready-for-agent

- [x] Shelf metadata is stored in a per-user SQLite database and includes Shelf Batch order/creation time plus Shelf Item identity, path, type, and pinned state.
- [x] Every completed drop-in, pin/unpin, manual removal, Missing cleanup, and Successful Drag-Out is committed transactionally.
- [x] Temporary and pinned Shelf Items restore after normal application restart and hidden auto-start.
- [x] Source file bytes and thumbnail bytes are never persisted.
- [x] Restored paths are reclassified as available, Missing, or Unavailable without crashing startup.
- [x] Database open or corruption failure preserves the failed database as a timestamped backup, informs the user, and starts with a new database rather than overwriting the backup.
- [x] The at-least-once crash window is preserved: an accepted drag that was not committed may reappear after restart.
- [x] Application-seam tests use real temporary SQLite databases and cover every persisted transition, restore ordering, corruption handling, and cleanup durability.
- [x] An installed-EXE smoke scenario confirms restored content after process and Windows-startup activation.

## Comments

- 2026-09-29: Ticket 06 requires an embedded transactional SQLite database. `Microsoft.Data.Sqlite` 10.0.12 is the official lightweight ADO.NET provider and avoids a custom database layer; its NuGet metadata declares MIT. Its bundled `SQLitePCLRaw.bundle_e_sqlite3` and native `SQLitePCLRaw.lib.e_sqlite3` 2.1.12 dependencies declare Apache-2.0. Both licenses permit commercial redistribution.
- 2026-09-29: Implemented `%LOCALAPPDATA%\DropCove\shelf.db` with transactional batch/item mutations, restore-time availability classification, serialized off-UI-thread SQLite work, and timestamped corruption backup/reset. The schema stores only shelf metadata; it has no source-content or thumbnail-content columns.
- 2026-09-29: Release build succeeded with zero warnings and all 29 tests passed. Real temporary SQLite tests cover restore order/identity/type/pinned state, pin/unpin, item/batch/bulk removal, Missing cleanup, accepted/canceled/rejected/failed drag durability, partial-availability retention, at-least-once restart, classification, and corruption recovery.
- 2026-09-29: Installed self-contained EXE smoke passed: a persisted item restored after process termination; `--autostart` kept the shelf hidden while restoring state; a second launch activated the resident process and exposed the restored item. The fixture and test batch were removed afterward.
- 2026-09-29: User approved aligning criterion 17 with the approved unpackaged deployment model. The verified installed self-contained EXE smoke therefore closes the restart and Windows-startup activation criterion.
- 2026-09-29: A focused working-tree standards/spec re-review found the initial async DragStarting deferral, access classification, partial confirmation, and partial-drag cleanup defects. Those findings were fixed; focused re-review returned no remaining P1 findings. User accepted this focused review as the final review because the repository has no pre-Ticket-06 committed fixed point.