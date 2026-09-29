# 06: Restore the shelf through SQLite

**What to build:** Persist the complete confirmed shelf lifecycle so temporary and pinned Path References survive application and Windows restarts without storing source or thumbnail content.

**Blocked by:** 05: Drag and clean complete Shelf Batches.

**Status:** ready-for-agent

- [ ] Shelf metadata is stored in a per-user SQLite database and includes Shelf Batch order/creation time plus Shelf Item identity, path, type, and pinned state.
- [ ] Every completed drop-in, pin/unpin, manual removal, Missing cleanup, and Successful Drag-Out is committed transactionally.
- [ ] Temporary and pinned Shelf Items restore after normal application restart and hidden auto-start.
- [ ] Source file bytes and thumbnail bytes are never persisted.
- [ ] Restored paths are reclassified as available, Missing, or Unavailable without crashing startup.
- [ ] Database open or corruption failure preserves the failed database as a timestamped backup, informs the user, and starts with a new database rather than overwriting the backup.
- [ ] The at-least-once crash window is preserved: an accepted drag that was not committed may reappear after restart.
- [ ] Application-seam tests use real temporary SQLite databases and cover every persisted transition, restore ordering, corruption handling, and cleanup durability.
- [ ] A packaged smoke scenario confirms restored content after process and Windows-startup activation.