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
    #[error("shelf height must be a finite value of at least 180 logical pixels")]
    InvalidHeight,
    #[error("database integrity check failed: {0}")]
    DatabaseCorrupt(String),
    #[error("shelf state lock is poisoned")]
    StatePoisoned,
    #[error("shelf persistence is closed")]
    StateClosed,
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
    store: Arc<Mutex<Option<ShelfStore>>>,
}

// A checked guard keeps all existing Shelf operations behind the same lifetime boundary.
// Its private Option cannot be cleared while this guard holds the store mutex.
struct ShelfStoreGuard<'a>(MutexGuard<'a, Option<ShelfStore>>);

impl std::ops::Deref for ShelfStoreGuard<'_> {
    type Target = ShelfStore;
    fn deref(&self) -> &ShelfStore {
        match self.0.as_ref() {
            Some(store) => store,
            None => unreachable!("ShelfStoreGuard is constructed only for an open store"),
        }
    }
}

impl std::ops::DerefMut for ShelfStoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut ShelfStore {
        match self.0.as_mut() {
            Some(store) => store,
            None => unreachable!("ShelfStoreGuard is constructed only for an open store"),
        }
    }
}

impl ShelfState {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, ShelfError> {
        Ok(Self {
            store: Arc::new(Mutex::new(Some(ShelfStore::open(path.as_ref())?))),
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

    pub(crate) fn save_size(
        &self,
        width: f64,
        manual_height: Option<f64>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        if !width.is_finite() || width < 180.0 {
            return Err(ShelfError::InvalidWidth);
        }
        if let Some(height) = manual_height {
            if !height.is_finite() || height < 180.0 {
                return Err(ShelfError::InvalidHeight);
            }
        }
        self.store()?.save_size(width, manual_height)
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
    pub(crate) fn clear_temporary(&self) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.clear_temporary()
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
    pub(crate) fn set_batch_pinned(
        &self,
        batch_id: &str,
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.set_batch_pinned(batch_id, pinned)
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

    fn store(&self) -> Result<ShelfStoreGuard<'_>, ShelfError> {
        let guard = self.store.lock().map_err(|_| ShelfError::StatePoisoned)?;
        if guard.is_none() {
            return Err(ShelfError::StateClosed);
        }
        Ok(ShelfStoreGuard(guard))
    }

    pub(crate) fn has_pending_operations(&self) -> bool {
        Arc::strong_count(&self.store) > 1
    }

    pub(crate) fn close(&self) -> Result<(), ShelfError> {
        let mut guard = self.store.lock().map_err(|_| ShelfError::StatePoisoned)?;
        // Wait for any current transaction, then close the real connection even when
        // background callers still retain ShelfState clones. Later operations are rejected.
        drop(guard.take());
        Ok(())
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
    #[cfg(test)]
    fn install_test_settings_failure(&self) -> Result<(), ShelfError> {
        self.store()?.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_settings_update
             BEFORE UPDATE ON shelf_settings
             BEGIN
                 SELECT RAISE(FAIL, 'forced settings update failure');
             END;",
        )?;
        Ok(())
    }

    #[cfg(test)]
    fn install_test_insert_failure(&self) -> Result<(), ShelfError> {
        self.store()?.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_item_insert
             BEFORE INSERT ON shelf_items
             BEGIN
                 SELECT RAISE(FAIL, 'forced insert failure');
             END;",
        )?;
        Ok(())
    }
}

struct ShelfStore {
    connection: Connection,
    width: f64,
    height: f64,
    pending_manual_height: Option<f64>,
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
            pending_manual_height: None,
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
            "INSERT OR IGNORE INTO shelf_settings (id, preferred_width, manual_height) VALUES (1, ?1, ?2)",
            params![self.width, self.pending_manual_height],
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
        self.pending_manual_height = None;
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
        if empty {
            self.width = 180.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
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
        if empty {
            self.width = 180.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
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
        if empty {
            self.width = 180.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
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
        if empty {
            self.width = 180.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }
    fn clear_temporary(&mut self) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM shelf_items WHERE is_pinned = 0", [])?;
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
        if empty {
            self.width = 180.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
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
    fn set_batch_pinned(
        &mut self,
        batch_id: &str,
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let updated = transaction.execute(
            "UPDATE shelf_items SET is_pinned = ?1 WHERE batch_id = ?2",
            params![pinned, batch_id],
        )?;
        if updated == 0 {
            return Err(ShelfError::BatchNotFound(batch_id.to_owned()));
        }
        let snapshot =
            Self::read_snapshot(&transaction, self.width, self.height, &self.availability)?;
        transaction.commit()?;
        Ok(snapshot)
    }

    fn save_size(
        &mut self,
        width: f64,
        manual_height: Option<f64>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let has_content: bool =
            self.connection
                .query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| {
                    row.get(0)
                })?;

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
    fn close_releases_database_and_rejects_retained_state_clones() {
        let directory = TestDirectory::new("resident-close");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap();
        state.save_size(300.0, Some(220.0)).unwrap();
        let retained = serde_json::to_value(state.snapshot().unwrap()).unwrap();
        let clone = state.clone();
        state.close().unwrap();
        assert!(matches!(
            clone.snapshot(),
            Err(super::ShelfError::StateClosed)
        ));
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            retained
        );
        reopened.close().unwrap();
        assert!(!database.with_extension("db-wal").exists());
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
        state.save_size(348.0, None).unwrap();
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
        state.save_size(300.0, None).unwrap();
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
    #[test]
    fn set_batch_pinned_transitions_and_preserves_availability_classifications() {
        use super::Availability;
        let directory = TestDirectory::new("batch-pin-transitions");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let avail_file = directory.0.join("available.txt");
        fs::write(&avail_file, b"avail").unwrap();
        let missing_file = directory.0.join("missing.txt");
        fs::write(&missing_file, b"will_delete").unwrap();
        let unavail_path = PathBuf::from("Z:\\unavailable_volume\\file.txt");

        let initial = state
            .accept_drop(vec![
                Some(avail_file.to_string_lossy().into_owned()),
                Some(missing_file.to_string_lossy().into_owned()),
                Some(unavail_path.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch_id = initial.batches[0].id.clone();
        let item_avail_id = initial.batches[0].items[0].id.clone();
        let item_missing_id = initial.batches[0].items[1].id.clone();
        let item_unavail_id = initial.batches[0].items[2].id.clone();

        // Delete missing file so it classifies as Missing
        fs::remove_file(&missing_file).unwrap();

        // Update availability in store cache via management_batch_with
        state
            .management_batch_with(&batch_id, |p| {
                if p == avail_file.to_string_lossy() {
                    Availability::Available
                } else if p == missing_file.to_string_lossy() {
                    Availability::Missing
                } else {
                    Availability::Unavailable
                }
            })
            .unwrap();

        let snap_before = state.snapshot().unwrap();
        assert_eq!(
            snap_before.batches[0].items[0].availability,
            Availability::Available
        );
        assert_eq!(
            snap_before.batches[0].items[1].availability,
            Availability::Missing
        );
        assert_eq!(
            snap_before.batches[0].items[2].availability,
            Availability::Unavailable
        );
        // Initial bulk state: Off (all items is_pinned == false)
        assert!(snap_before.batches[0].items.iter().all(|i| !i.is_pinned));

        // 1. Off -> On
        let snap_on = state.set_batch_pinned(&batch_id, true).unwrap();
        assert!(snap_on.batches[0].items.iter().all(|i| i.is_pinned));
        // Retains all 3 items with their classifications intact
        assert_eq!(snap_on.batches[0].items.len(), 3);
        assert_eq!(snap_on.batches[0].items[0].id, item_avail_id);
        assert_eq!(
            snap_on.batches[0].items[0].availability,
            Availability::Available
        );
        assert_eq!(snap_on.batches[0].items[1].id, item_missing_id);
        assert_eq!(
            snap_on.batches[0].items[1].availability,
            Availability::Missing
        );
        assert_eq!(snap_on.batches[0].items[2].id, item_unavail_id);
        assert_eq!(
            snap_on.batches[0].items[2].availability,
            Availability::Unavailable
        );

        // 2. On -> mixed by unpinning one item
        let snap_mixed = state.set_item_pinned(&item_missing_id, false).unwrap();
        assert!(snap_mixed.batches[0].items[0].is_pinned);
        assert!(!snap_mixed.batches[0].items[1].is_pinned);
        assert!(snap_mixed.batches[0].items[2].is_pinned);

        // 3. mixed -> On
        let snap_on_again = state.set_batch_pinned(&batch_id, true).unwrap();
        assert!(snap_on_again.batches[0].items.iter().all(|i| i.is_pinned));
        assert_eq!(
            snap_on_again.batches[0].items[0].availability,
            Availability::Available
        );
        assert_eq!(
            snap_on_again.batches[0].items[1].availability,
            Availability::Missing
        );
        assert_eq!(
            snap_on_again.batches[0].items[2].availability,
            Availability::Unavailable
        );

        // 4. On -> Off
        let snap_off = state.set_batch_pinned(&batch_id, false).unwrap();
        assert!(snap_off.batches[0].items.iter().all(|i| !i.is_pinned));
        assert_eq!(snap_off.batches[0].items.len(), 3);
        assert_eq!(
            snap_off.batches[0].items[0].availability,
            Availability::Available
        );
        assert_eq!(
            snap_off.batches[0].items[1].availability,
            Availability::Missing
        );
        assert_eq!(
            snap_off.batches[0].items[2].availability,
            Availability::Unavailable
        );

        // Unknown batch rejected with BatchNotFound
        assert!(matches!(
            state.set_batch_pinned("nonexistent", true),
            Err(super::ShelfError::BatchNotFound(_))
        ));
    }

    #[test]
    fn set_batch_pinned_rollback_preserves_visible_and_durable_state() {
        let directory = TestDirectory::new("batch-pin-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        let initial = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch_id = initial.batches[0].id.clone();
        let item_b_id = initial.batches[0].items[1].id.clone();

        // Set mixed state: item 0 unpinned, item 1 pinned
        state.set_item_pinned(&item_b_id, true).unwrap();
        let before = state.snapshot().unwrap();
        assert!(!before.batches[0].items[0].is_pinned);
        assert!(before.batches[0].items[1].is_pinned);

        // Install a failing trigger on update of is_pinned for item 1 specifically,
        // simulating failure halfway through the bulk update
        state
            .store()
            .unwrap()
            .connection
            .execute_batch(&format!(
                "CREATE TEMP TRIGGER fail_halfway BEFORE UPDATE OF is_pinned ON shelf_items
                 WHEN NEW.id = '{item_b_id}'
                 BEGIN SELECT RAISE(ABORT, 'forced bulk pin halfway failure'); END;"
            ))
            .unwrap();

        // Attempting to set all pinned to true (which would update item 0, then fail on item 1)
        // or false should fail and rollback completely
        assert!(state.set_batch_pinned(&batch_id, false).is_err());

        // Visible snapshot is preserved unchanged
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );

        // Reopening database verifies durable state was not partially mutated
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
    }

    #[test]
    fn pin_changes_between_prepared_batch_drag_and_completion_current_pin_state_wins() {
        let directory = TestDirectory::new("batch-pin-race");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        let initial = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch_id = initial.batches[0].id.clone();
        let item_a_id = initial.batches[0].items[0].id.clone();

        // Prepare batch drag while items are temporary (is_pinned = false)
        let prep = state.prepare_batch_drag(&batch_id).unwrap();
        let token = prep.token.expect("batch drag token should exist");
        assert_eq!(prep.available, 2);

        // While drag is active/prepared, user pins item_a via set_item_pinned
        // (or pins all via set_batch_pinned)
        state.set_item_pinned(&item_a_id, true).unwrap();

        // Complete batch drag with Copy outcome
        // consume_items executes "DELETE FROM shelf_items WHERE id = ?1 AND is_pinned = 0"
        // Since item_a is now pinned, it MUST be retained; item_b is temporary, so it is consumed
        let snap = state
            .complete_batch_drag(&token, DragOutcome::Copy)
            .unwrap();

        assert_eq!(snap.batches.len(), 1);
        assert_eq!(snap.batches[0].id, batch_id);
        assert_eq!(snap.batches[0].items.len(), 1);
        assert_eq!(snap.batches[0].items[0].id, item_a_id);
        assert!(snap.batches[0].items[0].is_pinned);

        // Clean up retained item explicitly to verify batch prunes
        let empty_snap = state.remove_item(&item_a_id).unwrap();
        assert!(empty_snap.batches.is_empty());
    }

    #[test]
    fn clear_temporary_mixed_batches_prunes_empty_and_preserves_identities_and_sources() {
        let directory = TestDirectory::new("clear-temp-mixed");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        state.save_size(320.0, None).unwrap();

        // Setup 3 files: a, b, c. Overlap a in multiple drops
        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        let file_c = directory.0.join("c.txt");
        let missing_path = directory.0.join("deleted_later.txt");

        fs::write(&file_a, b"content-a").unwrap();
        fs::write(&file_b, b"content-b").unwrap();
        fs::write(&file_c, b"content-c").unwrap();
        fs::write(&missing_path, b"content-missing").unwrap();

        // Batch 1 (older, pos 1): file_a and missing_path (both temporary initially)
        let snap1 = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(missing_path.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch1_id = snap1.batches[0].id.clone();
        let item_a1_id = snap1.batches[0].items[0].id.clone();
        let _item_missing_id = snap1.batches[0].items[1].id.clone();

        // Delete missing_path from disk so its availability becomes Missing/Unavailable
        fs::remove_file(&missing_path).unwrap();

        // Batch 2 (newer, pos 0): file_b (will be pinned) and file_c (temporary), plus overlapping file_a (temporary)
        let snap2 = state
            .accept_drop(vec![
                Some(file_b.to_string_lossy().into_owned()),
                Some(file_c.to_string_lossy().into_owned()),
                Some(file_a.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let batch2_id = snap2.batches[0].id.clone();
        let item_b2_id = snap2.batches[0].items[0].id.clone();
        let _item_c2_id = snap2.batches[0].items[1].id.clone();
        let _item_a2_id = snap2.batches[0].items[2].id.clone();

        // Pin item_a in Batch 1; Pin item_b in Batch 2
        state.set_item_pinned(&item_a1_id, true).unwrap();
        state.set_item_pinned(&item_b2_id, true).unwrap();

        // Clear temporary items across shelf
        let cleared = state.clear_temporary().unwrap();

        // Both batches survive because each has 1 pinned item
        assert_eq!(cleared.batches.len(), 2);
        // Order preserved: Batch 2 is position 0, Batch 1 is position 1
        assert_eq!(cleared.batches[0].id, batch2_id);
        assert_eq!(cleared.batches[1].id, batch1_id);

        // Surviving items retain identities and pin flag
        assert_eq!(cleared.batches[0].items.len(), 1);
        assert_eq!(cleared.batches[0].items[0].id, item_b2_id);
        assert!(cleared.batches[0].items[0].is_pinned);
        assert_eq!(cleared.batches[0].items[0].path, file_b.to_string_lossy());

        assert_eq!(cleared.batches[1].items.len(), 1);
        assert_eq!(cleared.batches[1].items[0].id, item_a1_id);
        assert!(cleared.batches[1].items[0].is_pinned);
        assert_eq!(cleared.batches[1].items[0].path, file_a.to_string_lossy());

        // Preferred width is preserved while any items remain
        assert_eq!(cleared.preferred_width, 320.0);

        // Source filesystem files are untouched
        assert_eq!(fs::read(&file_a).unwrap(), b"content-a");
        assert_eq!(fs::read(&file_b).unwrap(), b"content-b");
        assert_eq!(fs::read(&file_c).unwrap(), b"content-c");

        // Now unpin item_b2 and clear_temporary again: Batch 2 becomes empty and must be pruned
        state.set_item_pinned(&item_b2_id, false).unwrap();
        let pruned = state.clear_temporary().unwrap();
        assert_eq!(pruned.batches.len(), 1);
        assert_eq!(pruned.batches[0].id, batch1_id);
        assert_eq!(pruned.batches[0].items.len(), 1);
        assert_eq!(pruned.batches[0].items[0].id, item_a1_id);
        assert_eq!(pruned.preferred_width, 320.0);
    }

    #[test]
    fn clear_temporary_all_pinned_is_noop() {
        let directory = TestDirectory::new("clear-temp-noop");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let file_a = directory.0.join("a.txt");
        fs::write(&file_a, b"a").unwrap();

        let snap = state
            .accept_drop(vec![Some(file_a.to_string_lossy().into_owned())])
            .unwrap()
            .snapshot
            .unwrap();
        let item_id = snap.batches[0].items[0].id.clone();
        let pinned_snap = state.set_item_pinned(&item_id, true).unwrap();

        let cleared = state.clear_temporary().unwrap();
        assert_eq!(
            serde_json::to_value(&cleared).unwrap(),
            serde_json::to_value(&pinned_snap).unwrap()
        );
    }

    #[test]
    fn clear_temporary_all_temporary_clears_sizing_and_resets() {
        let directory = TestDirectory::new("clear-temp-final-reset");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        state.save_size(360.0, None).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
            ])
            .unwrap();

        let cleared = state.clear_temporary().unwrap();
        assert!(cleared.batches.is_empty());
        assert_eq!(cleared.preferred_width, 180.0);
        assert_eq!(cleared.height, 180.0);

        // Sources are still on disk
        assert_eq!(fs::read(&file_a).unwrap(), b"a");
        assert_eq!(fs::read(&file_b).unwrap(), b"b");
    }

    #[test]
    fn clear_temporary_rollback_preserves_visible_and_durable_state() {
        let directory = TestDirectory::new("clear-temp-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        state.save_size(340.0, None).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        let initial = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let before = initial;
        state.store().unwrap().connection.execute_batch(
            "CREATE TEMP TRIGGER fail_partial_clear BEFORE DELETE ON shelf_items
             WHEN NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = OLD.batch_id AND position = 0)
             BEGIN SELECT RAISE(FAIL, 'forced failure after first deletion'); END;",
        ).expect("partial-clear failure trigger should install");

        // clear_temporary must fail
        assert!(state.clear_temporary().is_err());

        // Visible snapshot is preserved unchanged
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );

        // Durable state on reopen is preserved unchanged
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
    }
    #[test]
    fn save_size_validation_rejects_sub_180_and_non_finite() {
        let directory = TestDirectory::new("size-validation");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();

        assert!(matches!(
            state.save_size(179.9, None),
            Err(super::ShelfError::InvalidWidth)
        ));
        assert!(matches!(
            state.save_size(f64::NAN, None),
            Err(super::ShelfError::InvalidWidth)
        ));
        assert!(matches!(
            state.save_size(f64::INFINITY, None),
            Err(super::ShelfError::InvalidWidth)
        ));
        assert!(matches!(
            state.save_size(200.0, Some(179.9)),
            Err(super::ShelfError::InvalidHeight)
        ));
        assert!(matches!(
            state.save_size(200.0, Some(f64::NAN)),
            Err(super::ShelfError::InvalidHeight)
        ));
        assert!(matches!(
            state.save_size(200.0, Some(f64::NEG_INFINITY)),
            Err(super::ShelfError::InvalidHeight)
        ));
    }

    #[test]
    fn width_only_eligibility_and_existing_override_preservation() {
        let directory = TestDirectory::new("width-eligibility");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Intake 1 batch
        state
            .accept_drop(vec![Some("C:\\file1.txt".into())])
            .unwrap();
        assert_eq!(state.snapshot().unwrap().height, 180.0);

        // Width-only resize: height is None, auto-growth eligibility preserved
        let snap = state.save_size(250.0, None).unwrap();
        assert_eq!(snap.preferred_width, 250.0);
        assert_eq!(snap.height, 180.0);

        // Add more batches: auto-growth kicks in because no manual override exists
        for i in 2..=4 {
            state
                .accept_drop(vec![Some(format!("C:\\file{i}.txt"))])
                .unwrap();
        }
        let grown = state.snapshot().unwrap();
        assert_eq!(grown.height, 348.0);

        // Now set manual override
        let manual_snap = state.save_size(250.0, Some(220.0)).unwrap();
        assert_eq!(manual_snap.height, 220.0);

        // Another width-only resize with None must PRESERVE the existing manual override
        let width_only = state.save_size(300.0, None).unwrap();
        assert_eq!(width_only.preferred_width, 300.0);
        assert_eq!(width_only.height, 220.0);
    }

    #[test]
    fn empty_pending_intent_and_first_valid_intake_atomicity() {
        let directory = TestDirectory::new("empty-pending");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Empty shelf: resize width + manual height
        let empty_snap = state.save_size(320.0, Some(260.0)).unwrap();
        assert_eq!(empty_snap.preferred_width, 320.0);
        assert_eq!(empty_snap.height, 260.0);
        assert!(empty_snap.batches.is_empty());

        // Check reading empty snapshot preserves pending intent
        let read_snap = state.snapshot().unwrap();
        assert_eq!(read_snap.preferred_width, 320.0);
        assert_eq!(read_snap.height, 260.0);

        // No settings row in SQLite while empty
        {
            let conn = rusqlite::Connection::open(&database).unwrap();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM shelf_settings", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0);
        }

        // Unsupported/empty drop must be no-op: does not create settings row or alter intent
        let drop_empty = state.accept_drop(vec![None, Some("".into())]).unwrap();
        assert!(drop_empty.snapshot.is_none());
        {
            let conn = rusqlite::Connection::open(&database).unwrap();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM shelf_settings", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 0);
        }
        let check_snap = state.snapshot().unwrap();
        assert_eq!(check_snap.preferred_width, 320.0);
        assert_eq!(check_snap.height, 260.0);

        // First valid accepted batch commits pending intent into shelf_settings atomically
        let accepted = state
            .accept_drop(vec![Some("C:\\first.txt".into())])
            .unwrap();
        let snap = accepted.snapshot.unwrap();
        assert_eq!(snap.preferred_width, 320.0);
        assert_eq!(snap.height, 260.0);

        // Verify settings row exists with persisted manual height
        {
            let conn = rusqlite::Connection::open(&database).unwrap();
            let (pw, mh): (f64, Option<f64>) = conn
                .query_row(
                    "SELECT preferred_width, manual_height FROM shelf_settings WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(pw, 320.0);
            assert_eq!(mh, Some(260.0));
        }

        // Duplicate drop is a no-op and does not alter persistence
        let dup = state
            .accept_drop(vec![Some("C:\\first.txt".into())])
            .unwrap();
        assert!(dup.snapshot.is_none());
    }

    #[test]
    fn restart_manual_versus_automatic_behavior() {
        let directory = TestDirectory::new("restart-manual-vs-auto");
        let database = directory.path().join("shelf.db");

        // Case 1: Manual override persists across restart
        {
            let state = ShelfState::open(&database).unwrap();
            state
                .accept_drop(vec![Some("C:\\item.txt".into())])
                .unwrap();
            state.save_size(280.0, Some(250.0)).unwrap();
        }
        {
            let restored = ShelfState::open(&database).unwrap();
            let snap = restored.snapshot().unwrap();
            assert_eq!(snap.preferred_width, 280.0);
            assert_eq!(snap.height, 250.0);
        }

        // Clear shelf
        {
            let state = ShelfState::open(&database).unwrap();
            state.clear_temporary().unwrap();
        }

        // Case 2: Automatic growth does NOT persist manual intent; recalculates on restart
        {
            let state = ShelfState::open(&database).unwrap();
            for i in 0..4 {
                state
                    .accept_drop(vec![Some(format!("C:\\file{i}.txt"))])
                    .unwrap();
            }
            assert_eq!(state.snapshot().unwrap().height, 348.0);
            // Set preferred width only (None for manual height)
            state.save_size(300.0, None).unwrap();
            // Remove 3 batches: live height does NOT shrink
            let snap = state.snapshot().unwrap();
            for batch in &snap.batches[..3] {
                state.remove_batch(&batch.id).unwrap();
            }
            assert_eq!(state.snapshot().unwrap().height, 348.0);
        }
        {
            // On restart, automatic height recalculates to 180.0 for 1 batch
            let restored = ShelfState::open(&database).unwrap();
            let snap = restored.snapshot().unwrap();
            assert_eq!(snap.preferred_width, 300.0);
            assert_eq!(snap.height, 180.0);
        }
    }

    #[test]
    fn final_item_removal_resets_manual_and_staged_sizing_to_180() {
        let directory = TestDirectory::new("final-item-reset");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // 1. Remove final item via remove_batch
        state
            .accept_drop(vec![Some("C:\\batch1.txt".into())])
            .unwrap();
        state.save_size(240.0, Some(220.0)).unwrap();
        let snap = state.snapshot().unwrap();
        let empty = state.remove_batch(&snap.batches[0].id).unwrap();
        assert_eq!((empty.preferred_width, empty.height), (180.0, 180.0));

        // Next intake after final removal defaults to 180x180
        let reintake = state
            .accept_drop(vec![Some("C:\\batch2.txt".into())])
            .unwrap();
        assert_eq!(
            (
                reintake.snapshot.as_ref().unwrap().preferred_width,
                reintake.snapshot.as_ref().unwrap().height
            ),
            (180.0, 180.0)
        );

        // 2. Remove final item via consume_items (Copy outcome)
        state.save_size(270.0, Some(230.0)).unwrap();
        let snap = state.snapshot().unwrap();
        let empty = state
            .complete_item_drag(&snap.batches[0].items[0].id, DragOutcome::Copy)
            .unwrap();
        assert_eq!((empty.preferred_width, empty.height), (180.0, 180.0));

        // 3. Remove final item via preparation pruning (missing path)
        state
            .accept_drop(vec![Some("C:\\missing_item.txt".into())])
            .unwrap();
        state.save_size(290.0, Some(240.0)).unwrap();
        let snap = state.snapshot().unwrap();
        let prep = state
            .prepare_batch_drag_with(&snap.batches[0].id, |_| super::Availability::Missing)
            .unwrap();
        assert_eq!(
            (prep.snapshot.preferred_width, prep.snapshot.height),
            (180.0, 180.0)
        );
    }

    #[test]
    fn failed_sizing_transaction_and_staged_commit_rollback() {
        let directory = TestDirectory::new("sizing-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Case 1: Failed save_size on populated shelf preserves previous retained intent
        state
            .accept_drop(vec![Some("C:\\item.txt".into())])
            .unwrap();
        state.save_size(250.0, Some(210.0)).unwrap();

        // Install trigger that fails updates on shelf_settings
        state.install_test_settings_failure().unwrap();

        let fail_res = state.save_size(300.0, Some(250.0));
        assert!(fail_res.is_err());

        // Previous cached intent retained
        let cached = state.snapshot().unwrap();
        assert_eq!(cached.preferred_width, 250.0);
        assert_eq!(cached.height, 210.0);

        // Drop and reopen to verify SQLite state was also rolled back
        drop(cached);
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        let reopened_snap = reopened.snapshot().unwrap();
        assert_eq!(reopened_snap.preferred_width, 250.0);
        assert_eq!(reopened_snap.height, 210.0);

        // Case 2: Failed batch insert rolls back staged commit and keeps staged intent pending
        reopened.clear_temporary().unwrap();
        reopened.save_size(320.0, Some(240.0)).unwrap();

        // Install trigger that fails item inserts
        reopened.install_test_insert_failure().unwrap();

        let insert_res = reopened.accept_drop(vec![Some("C:\\fail.txt".into())]);
        assert!(insert_res.is_err());

        // Cached state is still pending staged intent
        let pending_snap = reopened.snapshot().unwrap();
        assert_eq!(pending_snap.preferred_width, 320.0);
        assert_eq!(pending_snap.height, 240.0);
    }
}
