use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::windows_paths::{self, ordinal_equal};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DropResult {
    pub(crate) snapshot: Option<ShelfSnapshot>,
    pub(crate) skipped: usize,
}

struct IncomingItem {
    path: String,
    ordinal_path: Vec<u16>,
    availability: Availability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DragOutcome {
    Copy,
    Rejected,
    Canceled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum Availability {
    Available,
    Missing,
    Unavailable,
}

pub(crate) struct DragParticipant {
    pub(crate) id: String,
    pub(crate) path: String,
}

struct PreparedDrag {
    token: String,
    participants: Arc<Vec<DragParticipant>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchPreparation {
    pub(crate) token: Option<String>,
    pub(crate) available: usize,
    pub(crate) unavailable: usize,
    pub(crate) snapshot: ShelfSnapshot,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShelfSnapshot {
    pub(crate) batches: Vec<ShelfBatch>,
    pub(crate) preferred_width: f64,
    pub(crate) height: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShelfBatch {
    pub(crate) id: String,
    pub(crate) created_at: i64,
    pub(crate) position: i64,
    pub(crate) items: Vec<ShelfItem>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShelfItem {
    pub(crate) id: String,
    pub(crate) batch_id: String,
    pub(crate) position: i64,
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) is_folder: bool,
    pub(crate) is_pinned: bool,
    pub(crate) availability: Availability,
}

#[derive(Debug, Error)]
pub(crate) enum ShelfError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("filesystem error for `{path}`: {source}")]
    FileSystem {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("path contains a NUL character or exceeds Windows string limits")]
    InvalidPath,
    #[error("shelf width must be a finite value of at least 180 logical pixels")]
    InvalidWidth,
    #[error("database integrity check failed: {0}")]
    DatabaseCorrupt(String),
    #[error("shelf state lock is poisoned")]
    StatePoisoned,
    #[error("shelf item not found: {0}")]
    ItemNotFound(String),
    #[error("shelf batch not found: {0}")]
    BatchNotFound(String),
    #[error("prepared drag is no longer active")]
    InvalidPreparedDrag,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemClock,
    #[error("timestamp does not fit in SQLite INTEGER")]
    TimestampOverflow,
}

#[derive(Clone)]
pub(crate) struct ShelfState {
    store: Arc<Mutex<ShelfStore>>,
}

impl ShelfState {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, ShelfError> {
        Ok(Self {
            store: Arc::new(Mutex::new(ShelfStore::open(path.as_ref())?)),
        })
    }

    pub(crate) fn snapshot(&self) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.snapshot()
    }

    pub(crate) fn accept_drop(&self, paths: Vec<Option<String>>) -> Result<DropResult, ShelfError> {
        let mut items: Vec<IncomingItem> = Vec::with_capacity(paths.len());
        let mut skipped = 0;
        for path in paths {
            let Some(path) = path.filter(|path| !path.is_empty()) else {
                skipped += 1;
                continue;
            };
            if path.contains('\0') || path.len() > i32::MAX as usize {
                return Err(ShelfError::InvalidPath);
            }
            let ordinal_path = path.encode_utf16().collect::<Vec<_>>();
            if items
                .iter()
                .any(|item| ordinal_equal(&item.ordinal_path, &ordinal_path))
            {
                continue;
            }
            items.push(IncomingItem {
                path,
                ordinal_path,
                availability: Availability::Unavailable,
            });
        }
        if items.is_empty() {
            return Ok(DropResult {
                snapshot: None,
                skipped: 0,
            });
        }
        let mut store = self.store()?;
        let current = store.snapshot()?;
        // ponytail: linear retained-batch scan; index path sets only if measured shelf size needs it.
        for batch in &current.batches {
            if batch.items.len() == items.len()
                && batch.items.iter().all(|existing| {
                    let key = existing.path.encode_utf16().collect::<Vec<_>>();
                    items
                        .iter()
                        .any(|item| ordinal_equal(&key, &item.ordinal_path))
                })
            {
                return Ok(DropResult {
                    snapshot: None,
                    skipped: 0,
                });
            }
        }
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ShelfError::InvalidSystemClock)?
            .as_millis()
            .try_into()
            .map_err(|_| ShelfError::TimestampOverflow)?;
        let snapshot = store.insert_batch(items, created_at)?;
        Ok(DropResult {
            snapshot: Some(snapshot),
            skipped,
        })
    }

    pub(crate) fn save_width(&self, width: f64) -> Result<ShelfSnapshot, ShelfError> {
        if !width.is_finite() || width < 180.0 {
            return Err(ShelfError::InvalidWidth);
        }
        self.store()?.save_width(width)
    }

    pub(crate) fn complete_item_drag(
        &self,
        item_id: &str,
        outcome: DragOutcome,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let mut store = self.store()?;
        if outcome == DragOutcome::Copy {
            store.consume_items(std::iter::once(item_id))
        } else {
            store.snapshot()
        }
    }

    pub(crate) fn remove_batch(&self, batch_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.remove_batch(batch_id)
    }
    pub(crate) fn remove_item(&self, item_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.remove_item(item_id)
    }

    pub(crate) fn management_batch(&self, batch_id: &str) -> Result<ShelfBatch, ShelfError> {
        self.management_batch_with(batch_id, windows_paths::classify)
    }

    fn management_batch_with(
        &self,
        batch_id: &str,
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<ShelfBatch, ShelfError> {
        self.store()?.management_batch(batch_id, classifier)
    }

    pub(crate) fn set_item_pinned(
        &self,
        item_id: &str,
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.set_item_pinned(item_id, pinned)
    }

    pub(crate) fn item_path(&self, item_id: &str) -> Result<String, ShelfError> {
        self.store()?
            .item_path(item_id)?
            .ok_or_else(|| ShelfError::ItemNotFound(item_id.to_owned()))
    }

    pub(crate) fn prepare_batch_drag(
        &self,
        batch_id: &str,
    ) -> Result<BatchPreparation, ShelfError> {
        self.prepare_batch_drag_with(batch_id, windows_paths::classify)
    }

    fn prepare_batch_drag_with(
        &self,
        batch_id: &str,
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<BatchPreparation, ShelfError> {
        self.store()?.prepare_batch(batch_id, classifier)
    }

    pub(crate) fn prepared_participants(
        &self,
        token: &str,
    ) -> Result<Arc<Vec<DragParticipant>>, ShelfError> {
        self.store()?
            .prepared
            .as_ref()
            .filter(|prepared| prepared.token == token)
            .map(|prepared| Arc::clone(&prepared.participants))
            .ok_or(ShelfError::InvalidPreparedDrag)
    }

    pub(crate) fn complete_batch_drag(
        &self,
        token: &str,
        outcome: DragOutcome,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let mut store = self.store()?;
        let participants = store
            .prepared
            .as_ref()
            .filter(|prepared| prepared.token == token)
            .map(|prepared| Arc::clone(&prepared.participants))
            .ok_or(ShelfError::InvalidPreparedDrag)?;
        let result = if outcome == DragOutcome::Copy {
            store.consume_items(participants.iter().map(|item| item.id.as_str()))
        } else {
            store.snapshot()
        };
        store.prepared = None;
        result
    }

    fn store(&self) -> Result<MutexGuard<'_, ShelfStore>, ShelfError> {
        self.store.lock().map_err(|_| ShelfError::StatePoisoned)
    }

    #[cfg(test)]
    fn install_test_delete_failure(&self) -> Result<(), ShelfError> {
        self.store()?.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_item_delete
             BEFORE DELETE ON shelf_items
             BEGIN
                 SELECT RAISE(FAIL, 'forced delete failure');
             END;",
        )?;
        Ok(())
    }
}

struct ShelfStore {
    connection: Connection,
    width: f64,
    height: f64,
    availability: HashMap<String, Availability>,
    restored_availability: bool,
    prepared: Option<PreparedDrag>,
}

impl ShelfStore {
    fn open(path: &Path) -> Result<Self, ShelfError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ShelfError::FileSystem {
                path: parent.display().to_string(),
                source,
            })?;
        }

        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;

        let integrity: String =
            connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(ShelfError::DatabaseCorrupt(integrity));
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
             );",
        )?;

        let width = connection
            .query_row(
                "SELECT preferred_width FROM shelf_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(180.0);
        let mut store = Self {
            connection,
            width,
            height: 180.0,
            availability: HashMap::new(),
            restored_availability: false,
            prepared: None,
        };
        let snapshot = Self::read_snapshot(&store.connection, width, 180.0, &store.availability)?;
        store.height = snapshot.height;
        Ok(store)
    }

    fn snapshot(&mut self) -> Result<ShelfSnapshot, ShelfError> {
        let mut snapshot = Self::read_snapshot(
            &self.connection,
            self.width,
            self.height,
            &self.availability,
        )?;
        if !self.restored_availability {
            for item in snapshot
                .batches
                .iter_mut()
                .flat_map(|batch| batch.items.iter_mut())
            {
                item.availability = windows_paths::classify(&item.path);
            }
            self.record_snapshot(&snapshot);
            self.restored_availability = true;
        }
        Ok(snapshot)
    }

    fn record_snapshot(&mut self, snapshot: &ShelfSnapshot) {
        let count = snapshot.batches.iter().map(|batch| batch.items.len()).sum();
        let mut retained = HashSet::with_capacity(count);
        for item in snapshot.batches.iter().flat_map(|batch| &batch.items) {
            retained.insert(item.id.as_str());
            if let Some(value) = self.availability.get_mut(&item.id) {
                *value = item.availability;
            } else {
                self.availability.insert(item.id.clone(), item.availability);
            }
        }
        self.availability
            .retain(|id, _| retained.contains(id.as_str()));
    }

    fn read_snapshot(
        connection: &Connection,
        width: f64,
        height: f64,
        availability: &HashMap<String, Availability>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let mut batches_statement = connection.prepare(
            "SELECT id, created_at, position
             FROM shelf_batches
             ORDER BY position ASC",
        )?;
        let batches = batches_statement
            .query_map([], |row| {
                Ok(ShelfBatch {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    position: row.get(2)?,
                    items: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut snapshot = ShelfSnapshot {
            batches,
            preferred_width: width,
            height: 180.0,
        };
        let mut items_statement = connection.prepare(
            "SELECT id, batch_id, position, path, name, is_folder, is_pinned
             FROM shelf_items
             WHERE batch_id = ?1
             ORDER BY position ASC",
        )?;
        for batch in &mut snapshot.batches {
            batch.items = items_statement
                .query_map([&batch.id], |row| {
                    let id: String = row.get(0)?;
                    let availability = availability
                        .get(&id)
                        .copied()
                        .unwrap_or(Availability::Unavailable);
                    Ok(ShelfItem {
                        id,
                        batch_id: row.get(1)?,
                        position: row.get(2)?,
                        path: row.get(3)?,
                        name: row.get(4)?,
                        is_folder: row.get(5)?,
                        is_pinned: row.get(6)?,
                        availability,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
        }
        let manual_height: Option<f64> = connection
            .query_row(
                "SELECT manual_height FROM shelf_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        snapshot.height = manual_height
            .unwrap_or_else(|| height.max(automatic_height(width, snapshot.batches.len())));

        Ok(snapshot)
    }

    fn insert_batch(
        &mut self,
        mut items: Vec<IncomingItem>,
        created_at: i64,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("UPDATE shelf_batches SET position = position + 1", [])?;
        let batch_id = random_id(&transaction)?;
        transaction.execute(
            "INSERT OR IGNORE INTO shelf_settings (id, preferred_width, manual_height) VALUES (1, ?1, NULL)",
            [self.width],
        )?;
        transaction.execute(
            "INSERT INTO shelf_batches (id, created_at, position) VALUES (?1, ?2, 0)",
            params![batch_id, created_at],
        )?;
        for (position, item) in items.iter_mut().enumerate() {
            let (availability, is_folder) = windows_paths::inspect(&item.path);
            item.availability = availability;
            let name = Path::new(&item.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&item.path);
            let item_id = random_id(&transaction)?;
            transaction.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                params![
                    item_id,
                    batch_id,
                    position as i64,
                    item.path,
                    name,
                    is_folder
                ],
            )?;
        }
        let mut snapshot =
            Self::read_snapshot(&transaction, self.width, self.height, &self.availability)?;
        if let Some(batch) = snapshot
            .batches
            .iter_mut()
            .find(|batch| batch.id == batch_id)
        {
            for (item, incoming) in batch.items.iter_mut().zip(&items) {
                item.availability = incoming.availability;
            }
        }
        transaction.commit()?;
        self.height = snapshot.height;
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }

    fn prepare_batch(
        &mut self,
        batch_id: &str,
        mut classifier: impl FnMut(&str) -> Availability,
    ) -> Result<BatchPreparation, ShelfError> {
        let current = self.snapshot()?;
        let batch = current
            .batches
            .iter()
            .find(|batch| batch.id == batch_id)
            .ok_or_else(|| ShelfError::BatchNotFound(batch_id.to_owned()))?;
        let mut participants = Vec::with_capacity(batch.items.len());
        let mut missing = Vec::new();
        let mut unavailable = 0;
        for item in &batch.items {
            let availability = classifier(&item.path);
            if let Some(value) = self.availability.get_mut(&item.id) {
                *value = availability;
            }
            match availability {
                Availability::Available => participants.push(DragParticipant {
                    id: item.id.clone(),
                    path: item.path.clone(),
                }),
                Availability::Missing => missing.push(item.id.as_str()),
                Availability::Unavailable => unavailable += 1,
            }
        }
        let transaction = self.connection.transaction()?;
        for item_id in missing {
            transaction.execute("DELETE FROM shelf_items WHERE id = ?1", [item_id])?;
        }
        transaction.execute("DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)", [])?;
        clear_sizing_when_empty(&transaction)?;
        let empty =
            !transaction.query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| {
                row.get::<_, bool>(0)
            })?;
        let snapshot = Self::read_snapshot(
            &transaction,
            if empty { 180.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        let available = participants.len();
        let token = if available > 0 {
            Some(random_id(&transaction)?)
        } else {
            None
        };
        transaction.commit()?;
        self.width = snapshot.preferred_width;
        self.height = snapshot.height;
        self.record_snapshot(&snapshot);
        self.prepared = token.as_ref().map(|token| PreparedDrag {
            token: token.clone(),
            participants: Arc::new(participants),
        });
        Ok(BatchPreparation {
            token,
            available,
            unavailable,
            snapshot,
        })
    }

    fn consume_items<'a>(
        &mut self,
        item_ids: impl Iterator<Item = &'a str>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        for item_id in item_ids {
            transaction.execute(
                "DELETE FROM shelf_items WHERE id = ?1 AND is_pinned = 0",
                [item_id],
            )?;
        }
        transaction.execute("DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)", [])?;
        clear_sizing_when_empty(&transaction)?;
        let empty =
            !transaction.query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| {
                row.get::<_, bool>(0)
            })?;
        let snapshot = Self::read_snapshot(
            &transaction,
            if empty { 180.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        self.width = snapshot.preferred_width;
        self.height = snapshot.height;
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }

    fn remove_batch(&mut self, batch_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM shelf_batches WHERE id = ?1", [batch_id])?;
        clear_sizing_when_empty(&transaction)?;
        let empty =
            !transaction.query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| {
                row.get::<_, bool>(0)
            })?;
        let snapshot = Self::read_snapshot(
            &transaction,
            if empty { 180.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        self.width = snapshot.preferred_width;
        self.height = snapshot.height;
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }

    fn remove_item(&mut self, item_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let deleted = transaction.execute("DELETE FROM shelf_items WHERE id = ?1", [item_id])?;
        if deleted == 0 {
            return Err(ShelfError::ItemNotFound(item_id.to_owned()));
        }
        transaction.execute(
            "DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)",
            [],
        )?;
        clear_sizing_when_empty(&transaction)?;
        let empty =
            !transaction.query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| {
                row.get::<_, bool>(0)
            })?;
        let snapshot = Self::read_snapshot(
            &transaction,
            if empty { 180.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        self.width = snapshot.preferred_width;
        self.height = snapshot.height;
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }

    fn management_batch(
        &mut self,
        batch_id: &str,
        mut classifier: impl FnMut(&str) -> Availability,
    ) -> Result<ShelfBatch, ShelfError> {
        let mut batch: ShelfBatch = self
            .connection
            .query_row(
                "SELECT id, created_at, position FROM shelf_batches WHERE id = ?1",
                [batch_id],
                |row| {
                    Ok(ShelfBatch {
                        id: row.get(0)?,
                        created_at: row.get(1)?,
                        position: row.get(2)?,
                        items: Vec::new(),
                    })
                },
            )
            .optional()?
            .ok_or_else(|| ShelfError::BatchNotFound(batch_id.to_owned()))?;

        let mut items_statement = self.connection.prepare(
            "SELECT id, batch_id, position, path, name, is_folder, is_pinned
             FROM shelf_items
             WHERE batch_id = ?1
             ORDER BY position ASC",
        )?;
        batch.items = items_statement
            .query_map([batch_id], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(3)?;
                let availability = classifier(&path);
                Ok(ShelfItem {
                    id,
                    batch_id: row.get(1)?,
                    position: row.get(2)?,
                    path,
                    name: row.get(4)?,
                    is_folder: row.get(5)?,
                    is_pinned: row.get(6)?,
                    availability,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for item in &batch.items {
            if let Some(availability) = self.availability.get_mut(&item.id) {
                *availability = item.availability;
            } else {
                self.availability.insert(item.id.clone(), item.availability);
            }
        }

        Ok(batch)
    }

    fn set_item_pinned(
        &mut self,
        item_id: &str,
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        if transaction.execute(
            "UPDATE shelf_items SET is_pinned = ?1 WHERE id = ?2",
            params![pinned, item_id],
        )? == 0
        {
            return Err(ShelfError::ItemNotFound(item_id.to_owned()));
        }
        let snapshot =
            Self::read_snapshot(&transaction, self.width, self.height, &self.availability)?;
        transaction.commit()?;
        Ok(snapshot)
    }

    fn save_width(&mut self, width: f64) -> Result<ShelfSnapshot, ShelfError> {
        if width == self.width {
            return self.snapshot();
        }
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "UPDATE shelf_settings SET preferred_width = ?1 WHERE id = 1",
            [width],
        )?;
        let snapshot = Self::read_snapshot(&transaction, width, self.height, &self.availability)?;
        transaction.commit()?;
        self.width = width;
        self.height = snapshot.height;
        Ok(snapshot)
    }

    fn item_path(&self, item_id: &str) -> Result<Option<String>, ShelfError> {
        Ok(self
            .connection
            .query_row(
                "SELECT path FROM shelf_items WHERE id = ?1",
                [item_id],
                |row| row.get(0),
            )
            .optional()?)
    }
}

fn random_id(transaction: &Transaction<'_>) -> Result<String, rusqlite::Error> {
    transaction.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
}

fn clear_sizing_when_empty(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    transaction.execute(
        "DELETE FROM shelf_settings WHERE NOT EXISTS (SELECT 1 FROM shelf_items)",
        [],
    )?;
    Ok(())
}

fn automatic_height(width: f64, batches: usize) -> f64 {
    let columns = ((width - 12.0) / 168.0).floor().max(1.0) as usize;
    let rows = batches.div_ceil(columns);
    180.0 + rows.saturating_sub(1).min(3) as f64 * 56.0
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{DragOutcome, ShelfState};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("dropcove-{name}-{}-{sequence}", std::process::id()));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn source_file(&self) -> PathBuf {
            let path = self.0.join("source.txt");
            fs::write(&path, b"dropcove").expect("test source file should be written");
            path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn accepted_path_is_restored_after_reopen() {
        let directory = TestDirectory::new("restore");
        let database = directory.path().join("shelf.db");
        let source = directory.source_file();
        let expected_path = source.to_string_lossy().into_owned();

        let state = ShelfState::open(&database).expect("shelf should open");
        state
            .accept_drop(vec![Some(expected_path.clone())])
            .expect("path should be accepted");
        drop(state);

        let restored = ShelfState::open(&database)
            .expect("shelf should reopen")
            .snapshot()
            .expect("snapshot should load");

        assert_eq!(restored.batches[0].items[0].path, expected_path);
    }

    #[test]
    fn accepted_copy_removes_the_temporary_item() {
        let directory = TestDirectory::new("copy");
        let state = ShelfState::open(directory.path().join("shelf.db")).expect("shelf should open");
        let snapshot = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .expect("path should be accepted")
            .snapshot
            .unwrap();
        let item_id = &snapshot.batches[0].items[0].id;

        let completed = state
            .complete_item_drag(item_id, DragOutcome::Copy)
            .expect("copy completion should persist");

        assert!(completed.batches.is_empty());
    }

    #[test]
    fn canceled_drag_retains_the_temporary_item() {
        let directory = TestDirectory::new("cancel");
        let state = ShelfState::open(directory.path().join("shelf.db")).expect("shelf should open");
        let snapshot = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .expect("path should be accepted")
            .snapshot
            .unwrap();
        let item_id = &snapshot.batches[0].items[0].id;

        let completed = state
            .complete_item_drag(item_id, DragOutcome::Canceled)
            .expect("cancellation should succeed");

        assert_eq!(completed.batches[0].items[0].id, *item_id);
    }

    #[test]
    fn failed_copy_persistence_keeps_the_prior_item() {
        let directory = TestDirectory::new("rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).expect("shelf should open");
        let snapshot = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .expect("path should be accepted")
            .snapshot
            .unwrap();
        let item_id = snapshot.batches[0].items[0].id.clone();

        state
            .install_test_delete_failure()
            .expect("failure trigger should install");
        assert!(state
            .complete_item_drag(&item_id, DragOutcome::Copy)
            .is_err());
        drop(state);

        let restored = ShelfState::open(database)
            .expect("shelf should reopen")
            .snapshot()
            .expect("snapshot should load");

        assert_eq!(restored.batches[0].items[0].id, item_id);
    }

    #[test]
    fn invalid_database_is_refused_without_modification() {
        let directory = TestDirectory::new("integrity");
        let database = directory.path().join("shelf.db");
        let invalid_bytes = b"not a sqlite database";
        fs::write(&database, invalid_bytes).expect("invalid database should be written");

        assert!(ShelfState::open(&database).is_err());
        assert_eq!(
            fs::read(database).expect("database should remain readable"),
            invalid_bytes
        );
    }

    #[test]
    fn batch_preparation_requires_available_items_and_cancel_retains_all_eligible() {
        use super::Availability;
        let directory = TestDirectory::new("prepare-empty");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let initial = state
            .accept_drop(vec![Some("Z:\\unavailable.txt".into())])
            .unwrap()
            .snapshot
            .unwrap();
        let empty_prepared = state
            .prepare_batch_drag_with(&initial.batches[0].id, |_| Availability::Unavailable)
            .unwrap();
        assert!(empty_prepared.token.is_none());
        assert_eq!(
            (empty_prepared.available, empty_prepared.unavailable),
            (0, 1)
        );
    }

    #[test]
    fn copy_consumes_only_prepared_available_participants() {
        use super::Availability;
        let directory = TestDirectory::new("prepare-copy");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let initial = state
            .accept_drop(vec![
                Some("C:\\available.txt".into()),
                Some("Z:\\unavailable.txt".into()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let prepared = state
            .prepare_batch_drag_with(&initial.batches[0].id, |path| {
                if path.starts_with("Z:") {
                    Availability::Unavailable
                } else {
                    Availability::Available
                }
            })
            .unwrap();
        let token = prepared.token.unwrap();
        let added = state
            .accept_drop(vec![Some("C:\\unrelated.txt".into())])
            .unwrap()
            .snapshot
            .unwrap();
        assert_eq!(added.batches.len(), 2);
        let completed = state
            .complete_batch_drag(&token, DragOutcome::Copy)
            .unwrap();
        let original_batch = completed
            .batches
            .iter()
            .find(|batch| batch.id == initial.batches[0].id)
            .unwrap();
        assert_eq!(original_batch.items.len(), 1);
        assert_eq!(original_batch.items[0].path, "Z:\\unavailable.txt");
        assert!(completed
            .batches
            .iter()
            .any(|batch| batch.items[0].path == "C:\\unrelated.txt"));
    }
    #[test]
    fn batch_intake_keeps_first_occurrence_and_source_order() {
        let directory = TestDirectory::new("intake-order");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let result = state
            .accept_drop(vec![
                Some("C:\\B.txt".into()),
                Some("c:\\b.TXT".into()),
                None,
                Some("C:\\A.txt".into()),
            ])
            .unwrap();
        let snapshot = result.snapshot.unwrap();
        assert_eq!(
            snapshot.batches[0]
                .items
                .iter()
                .map(|item| item.path.as_str())
                .collect::<Vec<_>>(),
            vec!["C:\\B.txt", "C:\\A.txt"]
        );
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn exact_path_set_is_noop_but_partial_overlap_is_new_batch() {
        let directory = TestDirectory::new("path-sets");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let initial = state
            .accept_drop(vec![Some("C:\\A.txt".into()), Some("C:\\B.txt".into())])
            .unwrap()
            .snapshot
            .unwrap();
        let duplicate = state
            .accept_drop(vec![
                Some("c:\\b.TXT".into()),
                None,
                Some("c:\\a.txt".into()),
            ])
            .unwrap();
        assert!(duplicate.snapshot.is_none());
        assert_eq!(duplicate.skipped, 0);
        let overlapping = state
            .accept_drop(vec![Some("C:\\A.txt".into())])
            .unwrap()
            .snapshot
            .unwrap();
        assert_eq!(overlapping.batches[1].id, initial.batches[0].id);
        assert_eq!(overlapping.batches[0].items[0].path, "C:\\A.txt");
    }

    #[test]
    fn sizing_grows_live_then_recalculates_on_restore_and_resets_when_empty() {
        let directory = TestDirectory::new("sizing");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        for index in 0..4 {
            state
                .accept_drop(vec![Some(format!("C:\\{index}.txt"))])
                .unwrap();
        }
        let grown = state.snapshot().unwrap();
        assert_eq!(grown.height, 348.0);
        for batch in &grown.batches[..3] {
            state.remove_batch(&batch.id).unwrap();
        }
        assert_eq!(state.snapshot().unwrap().height, 348.0);
        state.save_width(348.0).unwrap();
        drop(state);
        let restored = ShelfState::open(database).unwrap();
        let snapshot = restored.snapshot().unwrap();
        assert_eq!((snapshot.preferred_width, snapshot.height), (348.0, 180.0));
        let empty = restored.remove_batch(&snapshot.batches[0].id).unwrap();
        assert_eq!((empty.preferred_width, empty.height), (180.0, 180.0));
    }

    #[test]
    fn batch_preparation_removes_only_missing_and_cancel_retains_available() {
        use super::Availability;
        let directory = TestDirectory::new("prepare-missing");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let initial = state
            .accept_drop(vec![
                Some("C:\\available.txt".into()),
                Some("C:\\missing.txt".into()),
                Some("Z:\\unavailable.txt".into()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let prepared = state
            .prepare_batch_drag_with(&initial.batches[0].id, |path| {
                if path.contains("missing") {
                    Availability::Missing
                } else if path.starts_with("Z:") {
                    Availability::Unavailable
                } else {
                    Availability::Available
                }
            })
            .unwrap();
        assert_eq!((prepared.available, prepared.unavailable), (1, 1));
        let canceled = state
            .complete_batch_drag(prepared.token.as_deref().unwrap(), DragOutcome::Canceled)
            .unwrap();
        assert_eq!(
            canceled.batches[0]
                .items
                .iter()
                .map(|item| item.path.as_str())
                .collect::<Vec<_>>(),
            vec!["C:\\available.txt", "Z:\\unavailable.txt"]
        );
    }

    #[test]
    fn pinned_item_survives_reopen_and_copy_until_unpinned() {
        let directory = TestDirectory::new("pin-lifecycle");
        let database = directory.path().join("shelf.db");
        let source = directory.source_file();
        let state = ShelfState::open(&database).unwrap();
        let initial = state
            .accept_drop(vec![Some(source.to_string_lossy().into_owned())])
            .unwrap()
            .snapshot
            .unwrap();
        let item_id = &initial.batches[0].items[0].id;
        assert!(!initial.batches[0].items[0].is_pinned);
        let pinned = state.set_item_pinned(item_id, true).unwrap();
        let mut expected = serde_json::to_value(&initial).unwrap();
        expected["batches"][0]["items"][0]["isPinned"] = true.into();
        assert_eq!(serde_json::to_value(&pinned).unwrap(), expected);
        drop(state);

        let restored = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(restored.snapshot().unwrap()).unwrap(),
            expected
        );
        for outcome in [
            DragOutcome::Canceled,
            DragOutcome::Rejected,
            DragOutcome::Copy,
        ] {
            let retained = restored.complete_item_drag(item_id, outcome).unwrap();
            assert_eq!(serde_json::to_value(retained).unwrap(), expected);
        }
        restored.set_item_pinned(item_id, false).unwrap();
        drop(restored);
        let temporary = ShelfState::open(&database).unwrap();
        assert!(!temporary.snapshot().unwrap().batches[0].items[0].is_pinned);
        for outcome in [DragOutcome::Canceled, DragOutcome::Rejected] {
            assert_eq!(
                temporary
                    .complete_item_drag(item_id, outcome)
                    .unwrap()
                    .batches[0]
                    .items[0]
                    .id,
                *item_id
            );
        }
        assert!(temporary
            .complete_item_drag(item_id, DragOutcome::Copy)
            .unwrap()
            .batches
            .is_empty());
        assert_eq!(fs::read(source).unwrap(), b"dropcove");
    }

    #[test]
    fn failed_pin_or_unpin_preserves_visible_and_restored_state() {
        let directory = TestDirectory::new("pin-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let initial = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap()
            .snapshot
            .unwrap();
        let item_id = &initial.batches[0].items[0].id;
        drop(state);

        for prior in [false, true] {
            let state = ShelfState::open(&database).unwrap();
            state.snapshot().unwrap();
            let before = state.set_item_pinned(item_id, prior).unwrap();
            state
                .store()
                .unwrap()
                .connection
                .execute_batch(
                    "CREATE TEMP TRIGGER fail_pin BEFORE UPDATE OF is_pinned ON shelf_items
                 BEGIN SELECT RAISE(ABORT, 'forced pin failure'); END;",
                )
                .unwrap();
            assert!(state.set_item_pinned(item_id, !prior).is_err());
            assert_eq!(
                serde_json::to_value(state.snapshot().unwrap()).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
            drop(state);
            let reopened = ShelfState::open(&database).unwrap();
            assert_eq!(
                serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
                serde_json::to_value(&before).unwrap()
            );
        }
    }

    #[test]
    fn pinning_unknown_item_is_rejected_and_pinned_batch_removal_is_explicit() {
        let directory = TestDirectory::new("pin-removal");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let source = directory.source_file();
        let initial = state
            .accept_drop(vec![Some(source.to_string_lossy().into_owned())])
            .unwrap()
            .snapshot
            .unwrap();
        assert!(matches!(
            state.set_item_pinned("absent", true),
            Err(super::ShelfError::ItemNotFound(_))
        ));
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&initial).unwrap()
        );
        state
            .set_item_pinned(&initial.batches[0].items[0].id, true)
            .unwrap();
        assert!(state
            .remove_batch(&initial.batches[0].id)
            .unwrap()
            .batches
            .is_empty());
        assert_eq!(fs::read(source).unwrap(), b"dropcove");
    }

    #[test]
    fn remove_item_deletes_exact_id_regardless_of_pin_and_preserves_same_path_in_other_batch() {
        let directory = TestDirectory::new("remove-id");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let source_a = directory.0.join("shared.txt");
        fs::write(&source_a, b"shared").unwrap();
        let shared_path = source_a.to_string_lossy().into_owned();

        let batch1 = state
            .accept_drop(vec![Some(shared_path.clone())])
            .unwrap()
            .snapshot
            .unwrap();
        let other = directory.0.join("other.txt");
        fs::write(&other, b"other").unwrap();
        let batch2 = state
            .accept_drop(vec![
                Some(shared_path.clone()),
                Some(other.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();

        let item1_id = &batch1.batches[0].items[0].id;
        let batch2_target = batch2
            .batches
            .iter()
            .find(|b| b.id != batch1.batches[0].id)
            .unwrap();
        let item2_shared = batch2_target
            .items
            .iter()
            .find(|it| it.path == shared_path)
            .unwrap();
        let item2_id = item2_shared.id.clone();
        assert_ne!(item1_id, &item2_id);

        // Pin item1_id and remove it directly; pin should not prevent explicit remove_item
        state.set_item_pinned(item1_id, true).unwrap();
        let after_removal = state.remove_item(item1_id).unwrap();

        // batch1 was single-item, so removing item1_id prunes batch1
        assert!(after_removal
            .batches
            .iter()
            .all(|b| b.id != batch1.batches[0].id));
        // batch2 and item2_id remain intact
        let retained_batch2 = after_removal
            .batches
            .iter()
            .find(|b| b.id == batch2_target.id)
            .unwrap();
        assert!(retained_batch2.items.iter().any(|it| it.id == item2_id));
        assert_eq!(retained_batch2.items.len(), 2);
        // Source file is preserved
        assert_eq!(fs::read(&source_a).unwrap(), b"shared");
    }

    #[test]
    fn remove_item_unknown_is_rejected_and_final_pruning_clears_sizing() {
        let directory = TestDirectory::new("remove-prune");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        state.save_width(300.0).unwrap();
        let initial = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap()
            .snapshot
            .unwrap();
        let item_id = &initial.batches[0].items[0].id;

        assert!(matches!(
            state.remove_item("nonexistent-item"),
            Err(super::ShelfError::ItemNotFound(_))
        ));

        let after = state.remove_item(item_id).unwrap();
        assert!(after.batches.is_empty());
        assert_eq!(after.preferred_width, 180.0);
        assert_eq!(after.height, 180.0);
    }

    #[test]
    fn failed_remove_item_leaves_visible_and_reopen_state_intact() {
        let directory = TestDirectory::new("remove-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let initial = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap()
            .snapshot
            .unwrap();
        let item_id = initial.batches[0].items[0].id.clone();

        state
            .install_test_delete_failure()
            .expect("failure trigger should install");
        assert!(state.remove_item(&item_id).is_err());
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&initial).unwrap()
        );
        drop(state);

        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&initial).unwrap()
        );
    }

    #[test]
    fn management_batch_refreshes_availability_without_cleanup_or_mutating_order() {
        use super::Availability;
        let directory = TestDirectory::new("mgmt-batch");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        let file_c = directory.0.join("c.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();
        fs::write(&file_c, b"c").unwrap();

        let initial = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
                Some(file_c.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch_id = &initial.batches[0].id;

        let managed = state
            .management_batch_with(batch_id, |path| {
                if path.ends_with("a.txt") {
                    Availability::Available
                } else if path.ends_with("b.txt") {
                    Availability::Missing
                } else {
                    Availability::Unavailable
                }
            })
            .unwrap();

        assert_eq!(managed.items.len(), 3);
        assert_eq!(managed.items[0].availability, Availability::Available);
        assert_eq!(managed.items[1].availability, Availability::Missing);
        assert_eq!(managed.items[2].availability, Availability::Unavailable);
        assert_eq!(managed.items[0].position, 0);
        assert_eq!(managed.items[1].position, 1);
        assert_eq!(managed.items[2].position, 2);

        // Verifying Missing items were NOT purged from the shelf
        let snapshot = state.snapshot().unwrap();
        assert_eq!(snapshot.batches[0].items.len(), 3);
        assert_eq!(
            snapshot.batches[0].items[1].availability,
            Availability::Missing
        );
    }

    #[test]
    fn pin_retention_in_multi_item_batch_during_item_completion() {
        let directory = TestDirectory::new("mgmt-pin");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let file1 = directory.0.join("one.txt");
        let file2 = directory.0.join("two.txt");
        fs::write(&file1, b"one").unwrap();
        fs::write(&file2, b"two").unwrap();

        let initial = state
            .accept_drop(vec![
                Some(file1.to_string_lossy().into_owned()),
                Some(file2.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch_id = &initial.batches[0].id;
        let item1_id = &initial.batches[0].items[0].id;
        let item2_id = &initial.batches[0].items[1].id;

        // Pin item1
        state.set_item_pinned(item1_id, true).unwrap();

        // Complete item drag for pinned item1 as Copy - should retain it
        let snap_after_pinned_drag = state
            .complete_item_drag(item1_id, DragOutcome::Copy)
            .unwrap();
        assert_eq!(snap_after_pinned_drag.batches[0].items.len(), 2);

        // Complete item drag for temporary item2 as Copy - should consume only item2
        let snap_after_temp_drag = state
            .complete_item_drag(item2_id, DragOutcome::Copy)
            .unwrap();
        let remaining_batch = &snap_after_temp_drag.batches[0];
        assert_eq!(remaining_batch.id, *batch_id);
        assert_eq!(remaining_batch.items.len(), 1);
        assert_eq!(remaining_batch.items[0].id, *item1_id);
        assert!(remaining_batch.items[0].is_pinned);

        // Removing item1 explicitly via remove_item prunes the now-empty batch
        let final_snap = state.remove_item(item1_id).unwrap();
        assert!(final_snap.batches.is_empty());
    }
}
