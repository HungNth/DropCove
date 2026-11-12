pub(crate) mod commands;
pub(crate) mod managed_files;
pub(crate) mod paths;
pub(crate) mod website_images;

mod drag;
mod intake;
mod managed_lifecycle;
mod pinning;
mod recovery;
mod removal;
mod snapshot;
mod store;
#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use paths::ordinal_equal;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use store::ShelfStore;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DropResult {
    pub(crate) snapshot: Option<ShelfSnapshot>,
    pub(crate) skipped: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
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
}

pub(crate) struct PreparedTransfer {
    pub(crate) participants: Vec<DragParticipant>,
    pub(crate) paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragDomain {
    Batch,
    Item,
}

struct PreparedDrag {
    token: String,
    domain: DragDomain,
    batch_id: Option<String>,
    transfer: Arc<PreparedTransfer>,
    changes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchPreparation {
    pub(crate) token: Option<String>,
    pub(crate) available: usize,
    pub(crate) unavailable: usize,
    pub(crate) snapshot: ShelfSnapshot,
}

pub(crate) type ItemPreparation = BatchPreparation;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShelfSnapshot {
    pub(crate) batches: Vec<ShelfBatch>,
    pub(crate) preferred_width: f64,
    pub(crate) height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) warning: Option<String>,
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
    pub(crate) ownership: ItemOwnership,
    pub(crate) managed_content_kind: Option<ManagedContentKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ItemOwnership {
    External,
    Managed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ManagedContentKind {
    Text,
    Image,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub(crate) enum ItemActionKind {
    ShowInExplorer,
    CopyPath,
    CopyText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResolvedItemAction {
    ShowInExplorer { path: String, is_folder: bool },
    CopyPath(String),
    CopyText(String),
}

/// Authoritative scope of a selection removal. Counts and the membership token come from
/// persisted state, never from the caller. The token is opaque to the frontend, which echoes it
/// back on confirm; `remove_batches` recomputes it from persisted state and rejects any change.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemovalPreview {
    pub(crate) batches: usize,
    pub(crate) items: usize,
    pub(crate) managed_files: usize,
    pub(crate) token: String,
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
    #[error("shelf width must be a finite value of at least 191 logical pixels")]
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
    #[error("batch selection must contain at least one batch")]
    EmptyBatchSelection,
    #[error("batch selection contains a duplicate batch: {0}")]
    DuplicateBatchSelection(String),
    #[error("item selection must contain at least one item")]
    EmptyItemSelection,
    #[error("item selection contains a duplicate item: {0}")]
    DuplicateItemSelection(String),
    #[error("the selection changed since removal was previewed; nothing was removed")]
    StaleRemovalTarget,
    #[error("prepared drag is no longer active")]
    InvalidPreparedDrag,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemClock,
    #[error("timestamp does not fit in SQLite INTEGER")]
    TimestampOverflow,
    #[error("database recovery failed to move `{path}` to `{backup_dir}`: {source}")]
    RecoveryMoveFailed {
        path: String,
        backup_dir: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("database recovery move failed and rollback could not fully restore artifacts; source: `{source_dir}`, backup: `{backup_dir}`: {details}")]
    RecoveryRollbackFailed {
        source_dir: std::path::PathBuf,
        backup_dir: std::path::PathBuf,
        details: String,
    },
    #[error("failed to recreate clean database at `{path}` after preserving recovery backup at `{backup_dir}`: {details}")]
    RecoveryRecreationFailed {
        path: String,
        backup_dir: std::path::PathBuf,
        details: String,
    },
    #[error("action `{0:?}` is not valid for item `{1}`")]
    InvalidItemAction(ItemActionKind, String),
}

impl ShelfError {
    pub(super) fn fs(path: impl std::fmt::Display, source: std::io::Error) -> Self {
        Self::FileSystem { path: path.to_string(), source }
    }
}

#[derive(Clone)]
pub(crate) struct ShelfState {
    store: Arc<Mutex<Option<ShelfStore>>>,
    pending_warning: Arc<Mutex<Option<String>>>,
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
            pending_warning: Arc::new(Mutex::new(None)),
        })
    }

    pub(crate) fn acknowledge_warning(&self) {
        if let Ok(mut lock) = self.pending_warning.lock() {
            *lock = None;
        }
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

    #[cfg(test)]
    pub(crate) fn install_test_rail_monitor_failure(&self) -> Result<(), ShelfError> {
        self.store()?.connection.execute_batch(
            "CREATE TEMP TRIGGER fail_rail_monitor_update
             BEFORE UPDATE ON shelf_ui_state
             BEGIN
                 SELECT RAISE(FAIL, 'forced rail monitor update failure');
             END;
             CREATE TEMP TRIGGER fail_rail_monitor_insert
             BEFORE INSERT ON shelf_ui_state
             BEGIN
                 SELECT RAISE(FAIL, 'forced rail monitor insert failure');
             END;",
        )?;
        Ok(())
    }
}
