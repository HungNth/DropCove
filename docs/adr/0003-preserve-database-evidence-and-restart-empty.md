# Preserve database evidence and restart with an empty Shelf

When the Shelf database cannot be opened or fails integrity validation, DropCove preserves the database and every existing WAL/SHM sidecar together as one timestamped Recovery Backup before creating a clean database and starting with an empty Shelf. DropCove does not attempt automatic salvage; if complete preservation or clean recreation fails, startup remains fatal. This favors intact recovery evidence and deterministic state over partial or misleading reconstruction.

ADR-0006 extends the Recovery Backup set with complete Managed File storage once app-owned content exists; the same all-or-fatal preservation rule applies.
