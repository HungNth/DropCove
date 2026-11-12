//! Persistent Shelf store: database open, schema, sizing, and shared helpers.

use rusqlite::OptionalExtension;
use rusqlite::{Connection, Transaction, params};
use std::fs;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;
use super::{Availability, PreparedDrag, ShelfError, ShelfSnapshot, ShelfState};

pub(super) struct ShelfStore {
    pub(super) connection: Connection,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) pending_manual_height: Option<f64>,
    pub(super) availability: HashMap<String, Availability>,
    pub(super) restored_availability: bool,
    pub(super) prepared: Option<PreparedDrag>,
    pub(super) profile_root: std::path::PathBuf,
    pub(super) pending_deletions_count: usize,
    pub(super) pending_reconcile_warning: Option<String>,
}

pub(super) fn random_id(transaction: &Transaction<'_>) -> Result<String, rusqlite::Error> {
    transaction.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
}

pub(super) fn validate_unique_batches(batch_ids: &[String]) -> Result<(), ShelfError> {
    if batch_ids.is_empty() {
        return Err(ShelfError::EmptyBatchSelection);
    }
    let mut seen = HashSet::with_capacity(batch_ids.len());
    for id in batch_ids {
        if !seen.insert(id.as_str()) {
            return Err(ShelfError::DuplicateBatchSelection(id.clone()));
        }
    }
    Ok(())
}

pub(super) fn validate_unique_items(item_ids: &[String]) -> Result<(), ShelfError> {
    if item_ids.is_empty() {
        return Err(ShelfError::EmptyItemSelection);
    }
    let mut seen = HashSet::with_capacity(item_ids.len());
    for id in item_ids {
        if !seen.insert(id.as_str()) {
            return Err(ShelfError::DuplicateItemSelection(id.clone()));
        }
    }
    Ok(())
}

// Length-delimited so no identity, whatever characters it contains, can collide with another.
pub(super) fn push_token_field(token: &mut String, field: &str) {
    token.push_str(&field.len().to_string());
    token.push(':');
    token.push_str(field);
}

pub(super) fn clear_sizing_when_empty(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute(
        "DELETE FROM shelf_settings
         WHERE NOT EXISTS (
             SELECT 1 FROM shelf_items si
             WHERE NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')
         )",
        [],
    )?;
    Ok(())
}

pub(super) fn automatic_height(width: f64, batches: usize) -> f64 {
    let columns = ((width - 12.0) / 168.0).floor().max(1.0) as usize;
    let rows = batches.div_ceil(columns);
    (52.0 * rows.min(5) as f64 + 74.0).max(180.0)
}

impl ShelfState {
    pub(crate) fn save_size(
        &self,
        width: f64,
        manual_height: Option<f64>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        if !width.is_finite() || width < 191.0 {
            return Err(ShelfError::InvalidWidth);
        }
        if let Some(height) = manual_height {
            if !height.is_finite() || height < 180.0 {
                return Err(ShelfError::InvalidHeight);
            }
        }
        self.store()?.save_size(width, manual_height)
    }
}

impl ShelfStore {
    pub(super) fn open(path: &Path) -> Result<Self, ShelfError> {
        Self::open_internal(path, true)
    }

    pub(super) fn open_internal(path: &Path, run_integrity: bool) -> Result<Self, ShelfError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ShelfError::fs(parent.display(), source))?;
        }

        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;

        if run_integrity {
            let integrity: String =
                connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
            if integrity != "ok" {
                return Err(ShelfError::DatabaseCorrupt(integrity));
            }
        }
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS shelf_batches (
                 id TEXT PRIMARY KEY,
                 created_at INTEGER NOT NULL,
                 position INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS shelf_items (
                 id TEXT PRIMARY KEY,
                 batch_id TEXT NOT NULL REFERENCES shelf_batches(id) ON DELETE CASCADE,
                 position INTEGER NOT NULL,
                 path TEXT NOT NULL,
                 name TEXT NOT NULL,
                 is_folder INTEGER NOT NULL CHECK (is_folder IN (0, 1)),
                 is_pinned INTEGER NOT NULL CHECK (is_pinned IN (0, 1)) DEFAULT 0,
                 UNIQUE (batch_id, position)
             );
             CREATE TABLE IF NOT EXISTS shelf_ui_state (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 rail_monitor TEXT
             );
             CREATE TABLE IF NOT EXISTS shelf_settings (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 preferred_width REAL NOT NULL,
                 manual_height REAL
             );
             CREATE TABLE IF NOT EXISTS managed_files (
                 item_id TEXT PRIMARY KEY REFERENCES shelf_items(id) ON DELETE CASCADE,
                 relative_path TEXT NOT NULL,
                 state TEXT NOT NULL CHECK (state IN ('staged', 'ready')),
                 staging_path TEXT
             );
             CREATE TABLE IF NOT EXISTS pending_managed_file_deletions (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 relative_path TEXT NOT NULL
             );",
        )?;

        let width = connection
            .query_row(
                "SELECT preferred_width FROM shelf_settings WHERE id = 1",
                [],
                |row| row.get::<_, f64>(0),
            )
            .optional()?
            .unwrap_or(191.0)
            .max(191.0);
        connection
            .query_row(
                "SELECT rail_monitor FROM shelf_ui_state WHERE id = 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        let profile_root = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let pending_deletions_count: usize = connection
            .query_row(
                "SELECT count(*) FROM pending_managed_file_deletions",
                [],
                |row| row.get::<_, i64>(0),
            )?
            .max(0) as usize;
        let mut store = Self {
            connection,
            width,
            height: 180.0,
            pending_manual_height: None,
            availability: HashMap::new(),
            restored_availability: false,
            prepared: None,
            profile_root,
            pending_deletions_count,
            pending_reconcile_warning: None,
        };
        let initial_warning = store.reconcile_managed_files()?;
        let _ = store.process_pending_managed_deletions();
        let empty: bool = !store.connection.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM shelf_items si
                JOIN shelf_batches sb ON sb.id = si.batch_id
                WHERE NOT EXISTS (
                    SELECT 1 FROM managed_files mf
                    WHERE mf.item_id = si.id AND mf.state = 'staged'
                )
            )",
            [],
            |row| row.get::<_, bool>(0),
        )?;
        if empty {
            store.width = 191.0;
            store.height = 180.0;
            store.pending_manual_height = None;
            let tx = store.connection.transaction()?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;
        }
        let snapshot = Self::read_snapshot(&store.connection, width, 180.0, &store.availability)?;
        store.height = snapshot.height;
        store.pending_reconcile_warning = initial_warning;
        Ok(store)
    }

    pub(super) fn save_size(
        &mut self,
        width: f64,
        manual_height: Option<f64>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let has_content: bool =
            self.connection
                .query_row(
                    "SELECT EXISTS (
                         SELECT 1 FROM shelf_items si
                         WHERE NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')
                     )",
                    [],
                    |row| row.get(0),
                )?;
        if !has_content {
            self.width = width;
            if let Some(height) = manual_height {
                self.height = height;
                self.pending_manual_height = Some(height);
            }
            return Ok(ShelfSnapshot {
                batches: Vec::new(),
                preferred_width: self.width,
                height: self.height,
                warning: if self.pending_deletions_count > 0 {
                    Some(if self.pending_deletions_count == 1 {
                        "1 DropCove-created file is pending deletion.".to_string()
                    } else {
                        format!("{} DropCove-created files are pending deletion.", self.pending_deletions_count)
                    })
                } else {
                    None
                },
            });
        }

        let transaction = self.connection.transaction()?;
        if transaction.execute(
            "UPDATE shelf_settings SET preferred_width = ?1, manual_height = COALESCE(?2, manual_height) WHERE id = 1",
            params![width, manual_height],
        )? != 1 {
            return Err(ShelfError::DatabaseCorrupt("retained items have no sizing settings".to_owned()));
        }
        let snapshot = Self::read_snapshot(&transaction, width, self.height, &self.availability)?;
        transaction.commit()?;
        self.width = snapshot.preferred_width;
        self.height = snapshot.height;
        self.pending_manual_height = None;
        Ok(snapshot)
    }
}
