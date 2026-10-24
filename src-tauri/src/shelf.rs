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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
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
}

pub(crate) fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1461 - doe / 142401) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn format_utc_timestamp(time: SystemTime) -> String {
    let dur = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs();
    let days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let (year, month, day) = days_to_ymd(days);
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    format!("{year:04}{month:02}{day:02}T{hour:02}{min:02}{sec:02}Z")
}

fn is_recoverable_error(err: &ShelfError) -> bool {
    match err {
        ShelfError::DatabaseCorrupt(_) => true,
        ShelfError::Database(rusqlite_err) => match rusqlite_err {
            rusqlite::Error::SqliteFailure(err, msg) => {
                let code = err.code;
                if matches!(
                    code,
                    rusqlite::ffi::ErrorCode::DatabaseCorrupt
                        | rusqlite::ffi::ErrorCode::NotADatabase
                ) {
                    return true;
                }
                if matches!(code, rusqlite::ffi::ErrorCode::Unknown) {
                    if let Some(msg) = msg {
                        let msg_lower = msg.to_ascii_lowercase();
                        return msg_lower.contains("no such table")
                            || msg_lower.contains("no such column");
                    }
                }
                false
            }
            rusqlite::Error::SqlInputError { error, msg, .. } => {
                if matches!(error.code, rusqlite::ffi::ErrorCode::Unknown) {
                    let msg_lower = msg.to_ascii_lowercase();
                    msg_lower.contains("no such table") || msg_lower.contains("no such column")
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn stage_shm_file(shm_path: &Path) -> Result<std::path::PathBuf, ShelfError> {
    let mut source_file = fs::File::open(shm_path).map_err(|source| ShelfError::FileSystem {
        path: shm_path.display().to_string(),
        source,
    })?;
    let pid = std::process::id();
    let mut attempt = 0u64;
    let (mut stage_file, stage_path) = loop {
        let mut os = shm_path.as_os_str().to_os_string();
        os.push(format!(".stage-{pid}-{attempt}"));
        let candidate = std::path::PathBuf::from(os);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => break (file, candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt += 1;
            }
            Err(source) => {
                return Err(ShelfError::FileSystem {
                    path: candidate.display().to_string(),
                    source,
                });
            }
        }
    };
    if let Err(source) = std::io::copy(&mut source_file, &mut stage_file)
        .and_then(|_| stage_file.sync_all())
    {
        drop(stage_file);
        let _ = fs::remove_file(&stage_path);
        return Err(ShelfError::FileSystem {
            path: stage_path.display().to_string(),
            source,
        });
    }
    Ok(stage_path)
}

fn restore_staged_shm(shm_path: &Path, stage_path: &Path) -> Result<(), ShelfError> {
    if fs::rename(stage_path, shm_path).is_ok() {
        return Ok(());
    }
    let copy_result = (|| -> std::io::Result<()> {
        let mut src = fs::File::open(stage_path)?;
        let mut dst = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(shm_path)?;
        std::io::copy(&mut src, &mut dst)?;
        Ok(())
    })();
    match copy_result {
        Ok(()) => {
            let _ = fs::remove_file(stage_path);
            Ok(())
        }
        Err(err) => Err(ShelfError::RecoveryRollbackFailed {
            source_dir: shm_path.to_path_buf(),
            backup_dir: stage_path.to_path_buf(),
            details: format!(
                "failed to restore original SHM evidence from staged copy `{}` to `{}`: {err}",
                stage_path.display(),
                shm_path.display()
            ),
        }),
    }
}

fn validate_existing_database(path: &Path, shm_path: &Path) -> Result<(), ShelfError> {
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(|source| ShelfError::FileSystem {
        path: path.display().to_string(),
        source,
    })?;
    let metadata = file.metadata().map_err(|source| ShelfError::FileSystem {
        path: path.display().to_string(),
        source,
    })?;
    if metadata.len() == 0 {
        return Ok(());
    }
    let mut header = [0u8; 16];
    match file.read_exact(&mut header) {
        Ok(()) => {
            if header != *b"SQLite format 3\0" {
                return Err(ShelfError::DatabaseCorrupt("file is not a database".into()));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(ShelfError::DatabaseCorrupt(
                "file is truncated / not a database".into(),
            ));
        }
        Err(err) => {
            return Err(ShelfError::FileSystem {
                path: path.display().to_string(),
                source: err,
            });
        }
    }
    drop(file);

    let had_shm = shm_path.exists();
    let staged_shm = if had_shm {
        Some(stage_shm_file(shm_path)?)
    } else {
        None
    };

    let validation_result = (|| -> Result<(), ShelfError> {
        let ro_conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        ro_conn.busy_timeout(Duration::from_secs(5))?;
        let integrity: String =
            ro_conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(ShelfError::DatabaseCorrupt(integrity));
        }

        ro_conn
            .query_row(
                "SELECT id, created_at, position FROM shelf_batches LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        ro_conn
            .query_row(
                "SELECT id, batch_id, position, path, name, is_folder, is_pinned FROM shelf_items LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        ro_conn
            .query_row(
                "SELECT id, rail_monitor FROM shelf_ui_state LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        ro_conn
            .query_row(
                "SELECT id, preferred_width, manual_height FROM shelf_settings LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        ro_conn
            .query_row(
                "SELECT item_id, relative_path, state, staging_path FROM managed_files LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;
        ro_conn
            .query_row(
                "SELECT id, relative_path FROM pending_managed_file_deletions LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?;

        Ok(())
    })();

    match validation_result {
        Ok(()) => {
            if let Some(stage_path) = staged_shm {
                let _ = fs::remove_file(stage_path);
            }
            Ok(())
        }
        Err(err) => {
            if let Some(stage_path) = staged_shm {
                restore_staged_shm(shm_path, &stage_path)?;
            } else if shm_path.exists() {
                let _ = fs::remove_file(shm_path);
            }
            Err(err)
        }
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

    pub(crate) fn deliver_recovery_warning(
        &self,
        backup_dir: Option<&Path>,
        notify: impl FnOnce(&str) -> Result<(), String>,
    ) {
        let Some(backup_dir) = backup_dir else {
            return;
        };
        let warning_message = format!(
            "Shelf database was corrupted and preserved in:\n{}\n\nStarted with an empty Shelf.",
            backup_dir.display()
        );
        if let Err(error) = notify(&warning_message) {
            eprintln!("recovery warning notification delivery failed: {error}");
            if let Ok(mut lock) = self.pending_warning.lock() {
                *lock = Some(warning_message);
            }
        }
    }

    pub(crate) fn open_or_recover(
        path: impl AsRef<Path>,
    ) -> Result<(Self, Option<std::path::PathBuf>), ShelfError> {
        Self::open_or_recover_at(path.as_ref(), SystemTime::now())
    }

    pub(crate) fn open_or_recover_at(
        path: &Path,
        now: SystemTime,
    ) -> Result<(Self, Option<std::path::PathBuf>), ShelfError> {
        if !path.exists() {
            let state = Self::open(path)?;
            return Ok((state, None));
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let mut wal_os = path.as_os_str().to_os_string();
        wal_os.push("-wal");
        let wal_path = std::path::PathBuf::from(wal_os);

        let mut shm_os = path.as_os_str().to_os_string();
        shm_os.push("-shm");
        let shm_path = std::path::PathBuf::from(shm_os);

        match validate_existing_database(path, &shm_path) {
            Ok(()) => {
                let store = ShelfStore::open_internal(path, false)?;
                Ok((
                    Self {
                        store: Arc::new(Mutex::new(Some(store))),
                        pending_warning: Arc::new(Mutex::new(None)),
                    },
                    None,
                ))
            }
            Err(err) if is_recoverable_error(&err) => {
                let recovery_area = parent.join("recovery");
                fs::create_dir_all(&recovery_area).map_err(|source| ShelfError::FileSystem {
                    path: recovery_area.display().to_string(),
                    source,
                })?;

                let timestamp = format_utc_timestamp(now);
                let base_name = format!("shelf-corrupt-{timestamp}");
                let mut suffix = 0usize;
                let backup_dir = loop {
                    let candidate = if suffix == 0 {
                        recovery_area.join(&base_name)
                    } else {
                        recovery_area.join(format!("{base_name}-{suffix}"))
                    };
                    match fs::create_dir(&candidate) {
                        Ok(()) => break candidate,
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            suffix += 1;
                        }
                        Err(source) => {
                            return Err(ShelfError::FileSystem {
                                path: candidate.display().to_string(),
                                source,
                            });
                        }
                    }
                };
                let file_name = path
                    .file_name()
                    .ok_or(ShelfError::InvalidPath)?;

                let mut to_move = Vec::new();
                to_move.push((path.to_path_buf(), backup_dir.join(file_name)));
                if wal_path.exists() {
                    to_move.push((
                        wal_path.clone(),
                        backup_dir.join(wal_path.file_name().unwrap()),
                    ));
                }
                if shm_path.exists() {
                    to_move.push((
                        shm_path.clone(),
                        backup_dir.join(shm_path.file_name().unwrap()),
                    ));
                }
                let managed_dir = parent.join("managed");
                if managed_dir.exists() {
                    to_move.push((
                        managed_dir.clone(),
                        backup_dir.join("managed"),
                    ));
                }
                let staging_dir = parent.join("staging");
                if staging_dir.exists() {
                    to_move.push((
                        staging_dir.clone(),
                        backup_dir.join("staging"),
                    ));
                }

                let mut moved = Vec::new();
                let mut move_failure = None;
                for (src, dst) in &to_move {
                    if let Err(e) = fs::rename(src, dst) {
                        move_failure = Some((src.clone(), dst.clone(), e));
                        break;
                    }
                    moved.push((src, dst));
                }

                if let Some((failed_src, failed_dst, move_err)) = move_failure {
                    let mut rollback_errs = Vec::new();
                    for (src, dst) in moved.into_iter().rev() {
                        if let Err(e) = fs::rename(dst, src) {
                            rollback_errs.push((dst.clone(), src.clone(), e));
                        }
                    }
                    let _ = fs::remove_dir(&backup_dir);
                    if !rollback_errs.is_empty() {
                        return Err(ShelfError::RecoveryRollbackFailed {
                            source_dir: parent.to_path_buf(),
                            backup_dir,
                            details: format!(
                                "failed to move `{}` to `{}` ({move_err}); rollback failed to restore {} artifact(s)",
                                failed_src.display(),
                                failed_dst.display(),
                                rollback_errs.len()
                            ),
                        });
                    }
                    return Err(ShelfError::RecoveryMoveFailed {
                        path: failed_src.display().to_string(),
                        backup_dir,
                        source: move_err,
                    });
                }

                let clean_store = ShelfStore::open_internal(path, false).map_err(|clean_err| {
                    ShelfError::RecoveryRecreationFailed {
                        path: path.display().to_string(),
                        backup_dir: backup_dir.clone(),
                        details: clean_err.to_string(),
                    }
                })?;
                if let Err(e) = fs::create_dir_all(parent.join("managed")) {
                    return Err(ShelfError::RecoveryRecreationFailed {
                        path: parent.join("managed").display().to_string(),
                        backup_dir: backup_dir.clone(),
                        details: format!("failed to recreate active managed directory: {e}"),
                    });
                }
                if let Err(e) = fs::create_dir_all(parent.join("staging")) {
                    return Err(ShelfError::RecoveryRecreationFailed {
                        path: parent.join("staging").display().to_string(),
                        backup_dir: backup_dir.clone(),
                        details: format!("failed to recreate active staging directory: {e}"),
                    });
                }

                Ok((
                    Self {
                        store: Arc::new(Mutex::new(Some(clean_store))),
                        pending_warning: Arc::new(Mutex::new(None)),
                    },
                    Some(backup_dir),
                ))
            }
            Err(unrecoverable) => Err(unrecoverable),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<ShelfSnapshot, ShelfError> {
        let mut snapshot = self.store()?.snapshot()?;
        if let Some(w) = self.pending_warning.lock().ok().and_then(|w| w.clone()) {
            snapshot.warning = Some(w);
        }
        Ok(snapshot)
    }

    pub(crate) fn rail_monitor(&self) -> Result<Option<String>, ShelfError> {
        Ok(self
            .store()?
            .connection
            .query_row(
                "SELECT rail_monitor FROM shelf_ui_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    pub(crate) fn remember_rail_monitor(&self, monitor: Option<&str>) -> Result<(), ShelfError> {
        self.store()?.connection.execute(
            "INSERT INTO shelf_ui_state (id, rail_monitor) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET rail_monitor = excluded.rail_monitor",
            [monitor],
        )?;
        Ok(())
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
                message: None,
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
                    message: None,
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
            message: None,
        })
    }
    pub(crate) fn accept_text(&self, text: String) -> Result<DropResult, ShelfError> {
        let trimmed_nul = text.strip_suffix('\0').unwrap_or(&text);
        if trimmed_nul.is_empty() {
            return Ok(DropResult {
                snapshot: None,
                skipped: 0,
                message: None,
            });
        }
        let bytes = trimmed_nul.as_bytes();
        if bytes.len() > crate::managed_files::MAX_TEXT_BYTES {
            return Ok(DropResult {
                snapshot: None,
                skipped: 1,
                message: None,
            });
        }
        let now = SystemTime::now();
        let timestamp_str = crate::managed_files::format_local_timestamp_now();
        let created_at = now
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ShelfError::InvalidSystemClock)?
            .as_millis()
            .try_into()
            .map_err(|_| ShelfError::TimestampOverflow)?;

        let mut store = self.store()?;
        let snapshot = store.insert_managed_text(bytes, &timestamp_str, created_at)?;
        Ok(DropResult {
            snapshot: Some(snapshot),
            skipped: 0,
            message: None,
        })
    }

    pub(crate) fn image_staging_store(&self) -> Result<crate::managed_files::ManagedStore, ShelfError> {
        let store = self.store()?;
        Ok(crate::managed_files::ManagedStore::new(&store.profile_root))
    }

    pub(crate) fn accept_image_files(
        &self,
        candidates: Vec<(String, String)>,
        skipped: usize,
    ) -> Result<DropResult, ShelfError> {
        if candidates.is_empty() {
            return Ok(DropResult {
                snapshot: None,
                skipped,
                message: None,
            });
        }
        let now = SystemTime::now();
        let created_at = now
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ShelfError::InvalidSystemClock)?
            .as_millis()
            .try_into()
            .map_err(|_| ShelfError::TimestampOverflow)?;

        let mut store = self.store()?;
        let count = candidates.len();
        let snapshot = store.insert_managed_images(candidates, created_at)?;
        let message = if skipped > 0 {
            Some(format!("Added {count} images; skipped {skipped}."))
        } else {
            None
        };
        Ok(DropResult {
            snapshot: Some(snapshot),
            skipped,
            message,
        })
    }

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

    pub(crate) fn item_visual_target(&self, item_id: &str) -> Result<Option<(String, bool)>, ShelfError> {
        self.store()?.item_visual_target(item_id)
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

    pub(crate) fn read_managed_text_content(&self, item_id: &str) -> Result<Option<String>, ShelfError> {
        let store = self.store()?;
        let relative_path: Option<String> = store.connection.query_row(
            "SELECT mf.relative_path FROM managed_files mf WHERE mf.item_id = ?1 AND mf.state = 'ready'",
            [item_id],
            |row| row.get(0),
        ).optional()?;
        let relative_path = match &relative_path {
            Some(p) if p.ends_with(".txt") => p.clone(),
            _ => return Ok(None),
        };
        let managed_store = crate::managed_files::ManagedStore::new(&store.profile_root);
        let full_path = match managed_store.full_path(&relative_path) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };
        let bytes = match std::fs::read(full_path) {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };
        Ok(String::from_utf8(bytes).ok())
    }

    pub(crate) fn is_item_managed(&self, item_id: &str) -> Result<bool, ShelfError> {
        let store = self.store()?;
        let exists: bool = store.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM managed_files WHERE item_id = ?1)",
            [item_id],
            |row| row.get(0),
        )?;
        Ok(exists)
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

struct ShelfStore {
    connection: Connection,
    width: f64,
    height: f64,
    pending_manual_height: Option<f64>,
    availability: HashMap<String, Availability>,
    restored_availability: bool,
    prepared: Option<PreparedDrag>,
    profile_root: std::path::PathBuf,
    pending_deletions_count: usize,
    pending_reconcile_warning: Option<String>,
}

impl ShelfStore {
    fn open(path: &Path) -> Result<Self, ShelfError> {
        Self::open_internal(path, true)
    }

    fn open_internal(path: &Path, run_integrity: bool) -> Result<Self, ShelfError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ShelfError::FileSystem {
                path: parent.display().to_string(),
                source,
            })?;
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
        if snapshot.warning.is_none() {
            if let Some(ref w) = self.pending_reconcile_warning {
                snapshot.warning = Some(w.clone());
            } else if self.pending_deletions_count > 0 {
                snapshot.warning = Some(if self.pending_deletions_count == 1 {
                    "1 DropCove-created file is pending deletion.".to_string()
                } else {
                    format!("{} DropCove-created files are pending deletion.", self.pending_deletions_count)
                });
            }
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
            "SELECT sb.id, sb.created_at, sb.position
             FROM shelf_batches sb
             WHERE EXISTS (
                 SELECT 1 FROM shelf_items si
                 WHERE si.batch_id = sb.id
                   AND NOT EXISTS (
                       SELECT 1 FROM managed_files mf
                       WHERE mf.item_id = si.id AND mf.state = 'staged'
                   )
             )
             ORDER BY sb.position ASC",
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
            warning: None,
        };
        let mut items_statement = connection.prepare(
            "SELECT si.id, si.batch_id, si.position, si.path, si.name, si.is_folder, si.is_pinned
             FROM shelf_items si
             WHERE si.batch_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM managed_files mf
                   WHERE mf.item_id = si.id AND mf.state = 'staged'
               )
             ORDER BY si.position ASC",
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
        snapshot.height = manual_height.unwrap_or_else(|| {
            if snapshot.batches.is_empty() {
                height
            } else {
                automatic_height(width, snapshot.batches.len())
            }
        });

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
    fn generate_text_snippet(bytes: &[u8]) -> String {
        let text = std::str::from_utf8(bytes).unwrap_or("");
        let collapsed: String = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if collapsed.is_empty() {
            return "Dropped text".to_string();
        }
        let char_count = collapsed.chars().count();
        if char_count <= 28 {
            collapsed
        } else {
            let truncated: String = collapsed.chars().take(25).collect();
            format!("{truncated}...")
        }
    }
    fn insert_managed_text(
        &mut self,
        bytes: &[u8],
        timestamp_str: &str,
        created_at: i64,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let store = crate::managed_files::ManagedStore::new(&self.profile_root);
        store.init_directories().map_err(|source| ShelfError::FileSystem {
            path: self.profile_root.display().to_string(),
            source,
        })?;

        // Generate display snippet from text content
        let display_name = Self::generate_text_snippet(bytes);

        // Determine collision-free filename
        let base_filename = format!("Dropped text {timestamp_str}.txt");
        let mut suffix = 1usize;
        let filename = loop {
            let candidate = if suffix == 1 {
                base_filename.clone()
            } else {
                format!("Dropped text {timestamp_str} ({suffix}).txt")
            };
            let exists: bool = self.connection.query_row(
                "SELECT EXISTS (
                     SELECT 1 FROM shelf_items WHERE name = ?1
                     UNION
                     SELECT 1 FROM managed_files WHERE relative_path LIKE '%' || ?1
                 )",
                [&candidate],
                |row| row.get(0),
            )?;
            if !exists {
                break candidate;
            }
            suffix += 1;
        };

        let item_id = {
            let tx = self.connection.transaction()?;
            let id = random_id(&tx)?;
            tx.rollback()?;
            id
        };
        let staging_rel = crate::managed_files::ManagedStore::staging_rel_path(&item_id);
        let final_rel = crate::managed_files::ManagedStore::final_rel_path(&item_id, &filename);
        let final_path_str = store.full_path(&final_rel).map_err(|source| ShelfError::FileSystem {
            path: final_rel.clone(), source,
        })?.display().to_string();

        // Step 1: Stage bytes to disk with flush
        let stage_full = store.stage_bytes(&staging_rel, bytes).map_err(|source| {
            ShelfError::FileSystem {
                path: staging_rel.clone(),
                source,
            }
        })?;

        // Step 2: Commit batch, item, and managed_files as 'staged'
        let transaction_result = (|| -> Result<(), ShelfError> {
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
            transaction.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, ?2, 0, ?3, ?4, 0, 0)",
                params![item_id, batch_id, final_path_str, display_name],
            )?;
            transaction.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                params![item_id, final_rel, staging_rel],
            )?;
            transaction.commit()?;
            Ok(())
        })();

        if let Err(tx_err) = transaction_result {
            let _ = fs::remove_file(stage_full);
            return Err(tx_err);
        }

        // Step 3: Promote staged file to final path (atomic rename)
        let promote_result = store.promote_staged(&staging_rel, &final_rel);
        if let Err(promote_err) = promote_result {
            // Compensation on runtime failure after staged commit:
            // Enqueue pending deletion for both paths and remove unprojected row atomically,
            // restoring sizing if empty, then attempting immediate deletion.
            let tx = self.connection.transaction()?;
            tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&staging_rel])?;
            tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&final_rel])?;
            tx.execute("DELETE FROM shelf_items WHERE id = ?1", [&item_id])?;
            tx.execute("DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)", [])?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;
            let _ = store.delete_managed_target(&staging_rel);
            let _ = store.delete_managed_target(&final_rel);
            let _ = self.process_pending_managed_deletions();
            return Err(ShelfError::FileSystem {
                path: final_rel,
                source: promote_err,
            });
        }

        // Step 4: Commit 'ready'
        let ready_result = self.connection.execute(
            "UPDATE managed_files SET state = 'ready', staging_path = NULL WHERE item_id = ?1",
            [&item_id],
        );
        if let Err(ready_err) = ready_result {
            let tx = self.connection.transaction()?;
            tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&final_rel])?;
            tx.execute("DELETE FROM shelf_items WHERE id = ?1", [&item_id])?;
            tx.execute("DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)", [])?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;
            let _ = store.delete_managed_target(&final_rel);
            let _ = self.process_pending_managed_deletions();
            return Err(ShelfError::Database(ready_err));
        }

        // Step 5: Read snapshot and publish
        self.availability.insert(item_id, Availability::Available);
        let snapshot = self.snapshot()?;
        self.height = snapshot.height;
        self.pending_manual_height = None;
        self.record_snapshot(&snapshot);
        Ok(snapshot)
    }

    fn insert_managed_images(
        &mut self,
        candidates: Vec<(String, String)>,
        created_at: i64,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let store = crate::managed_files::ManagedStore::new(&self.profile_root);
        store.init_directories().map_err(|source| ShelfError::FileSystem {
            path: self.profile_root.display().to_string(),
            source,
        })?;

        struct PreparedItem {
            item_id: String,
            position: i64,
            display_name: String,
            staging_rel: String,
            final_rel: String,
            final_path_str: String,
        }

        let mut prepared = Vec::with_capacity(candidates.len());
        for (pos, (staging_rel, filename)) in candidates.into_iter().enumerate() {
            let parts: Vec<&str> = staging_rel.split(['/', '\\']).collect();
            let item_id = match parts.as_slice() {
                ["staging", fname] => fname
                    .strip_suffix(".tmp")
                    .filter(|id| id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()))
                    .map(ToString::to_string)
                    .ok_or_else(|| ShelfError::InvalidPath)?,
                _ => return Err(ShelfError::InvalidPath),
            };
            let final_rel = crate::managed_files::ManagedStore::final_rel_path(&item_id, &filename);
            let final_path_str = store.full_path(&final_rel).map_err(|source| ShelfError::FileSystem {
                path: final_rel.clone(),
                source,
            })?.display().to_string();
            prepared.push(PreparedItem {
                item_id,
                position: pos as i64,
                display_name: filename,
                staging_rel,
                final_rel,
                final_path_str,
            });
        }

        // Step 1: Commit batch, all items, and managed_files as 'staged' in one transaction
        let tx_res = (|| -> Result<String, ShelfError> {
            let tx = self.connection.transaction()?;
            tx.execute("UPDATE shelf_batches SET position = position + 1", [])?;
            let batch_id = random_id(&tx)?;
            tx.execute(
                "INSERT OR IGNORE INTO shelf_settings (id, preferred_width, manual_height) VALUES (1, ?1, ?2)",
                params![self.width, self.pending_manual_height],
            )?;
            tx.execute(
                "INSERT INTO shelf_batches (id, created_at, position) VALUES (?1, ?2, 0)",
                params![batch_id, created_at],
            )?;
            for item in &prepared {
                tx.execute(
                    "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, 0)",
                    params![item.item_id, batch_id, item.position, item.final_path_str, item.display_name],
                )?;
                tx.execute(
                    "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                     VALUES (?1, ?2, 'staged', ?3)",
                    params![item.item_id, item.final_rel, item.staging_rel],
                )?;
            }
            tx.commit()?;
            Ok(batch_id)
        })();

        let batch_id = match tx_res {
            Ok(b) => b,
            Err(err) => {
                // Failure before durable commit: delete all uncommitted staging files
                for item in &prepared {
                    let _ = store.delete_managed_target(&item.staging_rel);
                }
                return Err(err);
            }
        };

        let mut first_promote_err = None;
        for item in &prepared {
            if let Err(err) = store.promote_staged(&item.staging_rel, &item.final_rel) {
                first_promote_err = Some(ShelfError::FileSystem {
                    path: item.final_rel.clone(),
                    source: err,
                });
                break;
            }
        }

        if let Some(first_err) = first_promote_err {
            // Group rollback: record cleanup responsibility for all staging and final paths
            let tx = self.connection.transaction()?;
            for item in &prepared {
                tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&item.staging_rel])?;
                tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&item.final_rel])?;
            }
            tx.execute("DELETE FROM shelf_batches WHERE id = ?1", [&batch_id])?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;

            for item in &prepared {
                let _ = store.delete_managed_target(&item.staging_rel);
                let _ = store.delete_managed_target(&item.final_rel);
            }
            let _ = self.process_pending_managed_deletions();
            return Err(first_err);
        }

        // Step 3: Commit all ownership rows to 'ready' in one transaction
        let ready_res = (|| -> Result<(), rusqlite::Error> {
            let tx = self.connection.transaction()?;
            for item in &prepared {
                tx.execute(
                    "UPDATE managed_files SET state = 'ready', staging_path = NULL WHERE item_id = ?1",
                    [&item.item_id],
                )?;
            }
            tx.commit()?;
            Ok(())
        })();

        if let Err(ready_err) = ready_res {
            let tx = self.connection.transaction()?;
            for item in &prepared {
                tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&item.final_rel])?;
            }
            tx.execute("DELETE FROM shelf_batches WHERE id = ?1", [&batch_id])?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;

            for item in &prepared {
                let _ = store.delete_managed_target(&item.final_rel);
            }
            let _ = self.process_pending_managed_deletions();
            return Err(ShelfError::Database(ready_err));
        }

        // Step 4: Publish accepted snapshot
        for item in &prepared {
            self.availability.insert(item.item_id.clone(), Availability::Available);
        }
        let snapshot = self.snapshot()?;
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
        let _ = Self::queue_managed_deletions_for_items(&transaction, missing.iter().copied())?;
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
            if empty { 191.0 } else { self.width },
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
            self.width = 191.0;
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
        self.process_pending_managed_deletions()?;
        let snapshot = self.snapshot()?;
        Ok(BatchPreparation {
            token,
            available,
            unavailable,
            snapshot,
        })
    }
    fn queue_managed_deletions_for_items<'a>(
        transaction: &Transaction<'_>,
        item_ids: impl Iterator<Item = &'a str>,
    ) -> Result<(), ShelfError> {
        let mut stmt = transaction.prepare(
            "SELECT relative_path FROM managed_files WHERE item_id = ?1",
        )?;
        let mut insert_pending = transaction.prepare(
            "INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)",
        )?;
        for item_id in item_ids {
            let mut rows = stmt.query([item_id])?;
            while let Some(row) = rows.next()? {
                let rel_path: String = row.get(0)?;
                insert_pending.execute([&rel_path])?;
            }
        }
        Ok(())
    }

    fn queue_managed_deletions_for_batch(
        transaction: &Transaction<'_>,
        batch_id: &str,
    ) -> Result<(), ShelfError> {
        let mut stmt = transaction.prepare(
            "SELECT mf.relative_path
             FROM managed_files mf
             JOIN shelf_items si ON si.id = mf.item_id
             WHERE si.batch_id = ?1",
        )?;
        let mut insert_pending = transaction.prepare(
            "INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)",
        )?;
        let mut rows = stmt.query([batch_id])?;
        while let Some(row) = rows.next()? {
            let rel_path: String = row.get(0)?;
            insert_pending.execute([&rel_path])?;
        }
        Ok(())
    }

    fn queue_managed_deletions_for_unpinned(
        transaction: &Transaction<'_>,
    ) -> Result<(), ShelfError> {
        let mut stmt = transaction.prepare(
            "SELECT mf.relative_path
             FROM managed_files mf
             JOIN shelf_items si ON si.id = mf.item_id
             WHERE si.is_pinned = 0",
        )?;
        let mut insert_pending = transaction.prepare(
            "INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let rel_path: String = row.get(0)?;
            insert_pending.execute([&rel_path])?;
        }
        Ok(())
    }

    fn process_pending_managed_deletions(&mut self) -> Result<(), ShelfError> {
        let store = crate::managed_files::ManagedStore::new(&self.profile_root);
        let mut stmt = self.connection.prepare(
            "SELECT id, relative_path FROM pending_managed_file_deletions",
        )?;
        let pending: Vec<(i64, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        for (id, rel_path) in pending {
            match store.delete_managed_target(&rel_path) {
                Ok(true) => {
                    let _ = self.connection.execute(
                        "DELETE FROM pending_managed_file_deletions WHERE id = ?1",
                        [id],
                    );
                }
                Ok(false) | Err(_) => {
                    // Remains pending for retry on later mutations or restart
                }
            }
        }
        self.pending_deletions_count = self
            .connection
            .query_row(
                "SELECT count(*) FROM pending_managed_file_deletions",
                [],
                |row| row.get::<_, i64>(0),
            )?
            .max(0) as usize;
        Ok(())
    }
    fn validate_managed_file(path: &Path) -> Result<bool, ShelfError> {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(ShelfError::FileSystem {
                path: path.display().to_string(),
                source: e,
            }),
        };
        if !meta.is_file() {
            return Ok(false);
        }
        let is_txt = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("txt")).unwrap_or(false);
        if is_txt {
            if meta.len() <= crate::managed_files::MAX_TEXT_BYTES as u64 {
                let bytes = match fs::read(path) {
                    Ok(b) => b,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
                    Err(e) => return Err(ShelfError::FileSystem {
                        path: path.display().to_string(),
                        source: e,
                    }),
                };
                return Ok(std::str::from_utf8(&bytes).is_ok());
            }
            return Ok(false);
        } else {
            match crate::website_images::validate_image(path) {
                Ok(_) => Ok(true),
                Err(crate::website_images::ValidationError::Invalid(_)) => Ok(false),
                Err(crate::website_images::ValidationError::Io(e)) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(false)
                    } else {
                        Err(ShelfError::FileSystem {
                            path: path.display().to_string(),
                            source: e,
                        })
                    }
                }
            }
        }
    }

    fn reconcile_managed_files(&mut self) -> Result<Option<String>, ShelfError> {
        let store = crate::managed_files::ManagedStore::new(&self.profile_root);
        let _ = store.init_directories();

        // Query all distinct batch_ids that have staged items
        let mut batch_stmt = self.connection.prepare(
            "SELECT DISTINCT si.batch_id
             FROM shelf_items si
             JOIN managed_files mf ON mf.item_id = si.id
             WHERE mf.state = 'staged'",
        )?;
        let staged_batch_ids: Vec<String> = batch_stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(batch_stmt);

        let mut discarded_count = 0usize;

        for batch_id in staged_batch_ids {
            // Retrieve all items for this batch
            let mut item_stmt = self.connection.prepare(
                "SELECT si.id, mf.relative_path, mf.staging_path
                 FROM shelf_items si
                 JOIN managed_files mf ON mf.item_id = si.id
                 WHERE si.batch_id = ?1",
            )?;
            struct BatchItemRecord {
                item_id: String,
                final_rel: String,
                staging_rel_opt: Option<String>,
                needs_promotion: bool,
            }
            let mut records = Vec::new();
            let mut batch_valid = true;

            let rows = item_stmt.query_map([&batch_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?))
            })?;

            for row in rows {
                let (item_id, final_rel, staging_rel_opt) = row?;
                let staging_path = staging_rel_opt.as_ref().and_then(|r| store.full_path(r).ok());
                let final_path = store.full_path(&final_rel).ok();

                let final_valid = match final_path.as_deref() {
                    Some(p) => Self::validate_managed_file(p)?,
                    None => false,
                };
                let staging_valid = match staging_path.as_deref() {
                    Some(p) => Self::validate_managed_file(p)?,
                    None => false,
                };

                if final_valid {
                    records.push(BatchItemRecord {
                        item_id,
                        final_rel,
                        staging_rel_opt,
                        needs_promotion: false,
                    });
                } else if staging_valid {
                    records.push(BatchItemRecord {
                        item_id,
                        final_rel,
                        staging_rel_opt,
                        needs_promotion: true,
                    });
                } else {
                    batch_valid = false;
                    records.push(BatchItemRecord {
                        item_id,
                        final_rel,
                        staging_rel_opt,
                        needs_promotion: false,
                    });
                }
            }
            drop(item_stmt);

            if batch_valid {
                // Try promoting all items that need promotion
                let mut promotion_ok = true;
                for item in &records {
                    if item.needs_promotion {
                        let st_rel = item.staging_rel_opt.as_ref().unwrap();
                        if store.promote_staged(st_rel, &item.final_rel).is_err() {
                            promotion_ok = false;
                            break;
                        }
                        let final_path = store.full_path(&item.final_rel).map_err(|e| ShelfError::FileSystem {
                            path: item.final_rel.clone(),
                            source: e,
                        })?;
                        if !Self::validate_managed_file(&final_path)? {
                            promotion_ok = false;
                            break;
                        }
                    }
                }

                if promotion_ok {
                    // Mark all ready atomically
                    let tx_res = (|| -> Result<(), rusqlite::Error> {
                        let tx = self.connection.transaction()?;
                        for item in &records {
                            tx.execute(
                                "UPDATE managed_files SET state = 'ready', staging_path = NULL WHERE item_id = ?1",
                                [&item.item_id],
                            )?;
                        }
                        tx.commit()?;
                        Ok(())
                    })();

                    if tx_res.is_ok() {
                        for item in &records {
                            if let Some(st_rel) = &item.staging_rel_opt {
                                let _ = store.delete_managed_target(st_rel);
                            }
                        }
                        continue; // Successfully reconciled this batch
                    }
                }
            }

            // Batch recovery failed or not fully valid: discard entire batch atomically
            discarded_count += records.len();
            let tx = self.connection.transaction()?;
            for item in &records {
                if let Some(st_rel) = &item.staging_rel_opt {
                    tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [st_rel])?;
                }
                tx.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&item.final_rel])?;
            }
            tx.execute("DELETE FROM shelf_batches WHERE id = ?1", [&batch_id])?;
            clear_sizing_when_empty(&tx)?;
            tx.commit()?;

            for item in &records {
                if let Some(st_rel) = &item.staging_rel_opt {
                    let _ = store.delete_managed_target(st_rel);
                }
                let _ = store.delete_managed_target(&item.final_rel);
            }
            let _ = self.process_pending_managed_deletions();
        }

        self.connection.execute(
            "DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)",
            [],
        )?;

        let mut stmt = self.connection.prepare(
            "SELECT staging_path FROM managed_files WHERE staging_path IS NOT NULL",
        )?;
        let active_staging: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        let _ = store.cleanup_unowned_staging(&active_staging);

        let warning = if discarded_count > 0 {
            Some(if discarded_count == 1 {
                "Interrupted drop was discarded.".to_string()
            } else {
                format!("{discarded_count} interrupted drops were discarded.")
            })
        } else {
            None
        };

        Ok(warning)
    }

    fn consume_items<'a>(
        &mut self,
        item_ids: impl Iterator<Item = &'a str>,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let item_ids_vec: Vec<&str> = item_ids.collect();
        let transaction = self.connection.transaction()?;
        let mut unpinned_ids = Vec::new();
        {
            let mut check_pinned = transaction.prepare(
                "SELECT is_pinned FROM shelf_items WHERE id = ?1",
            )?;
            for item_id in &item_ids_vec {
                let pinned: bool = check_pinned
                    .query_row([item_id], |row| row.get(0))
                    .optional()?
                    .unwrap_or(false);
                if !pinned {
                    unpinned_ids.push(*item_id);
                }
            }
        }
        Self::queue_managed_deletions_for_items(
            &transaction,
            unpinned_ids.iter().copied(),
        )?;
        for item_id in item_ids_vec {
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
            if empty { 191.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        if empty {
            self.width = 191.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
        self.record_snapshot(&snapshot);
        self.process_pending_managed_deletions()?;
        let snapshot = self.snapshot()?;
        Ok(snapshot)
    }

    fn remove_batch(&mut self, batch_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let _ = Self::queue_managed_deletions_for_batch(&transaction, batch_id)?;
        transaction.execute("DELETE FROM shelf_batches WHERE id = ?1", [batch_id])?;
        clear_sizing_when_empty(&transaction)?;
        let empty =
            !transaction.query_row(
                "SELECT EXISTS (
                    SELECT 1 FROM shelf_items si
                    WHERE NOT EXISTS (
                        SELECT 1 FROM managed_files mf
                        WHERE mf.item_id = si.id AND mf.state = 'staged'
                    )
                )",
                [],
                |row| row.get::<_, bool>(0),
            )?;
        let snapshot = Self::read_snapshot(
            &transaction,
            if empty { 191.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        if empty {
            self.width = 191.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
        self.record_snapshot(&snapshot);
        self.process_pending_managed_deletions()?;
        let snapshot = self.snapshot()?;
        Ok(snapshot)
    }

    fn remove_item(&mut self, item_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let _ = Self::queue_managed_deletions_for_items(&transaction, std::iter::once(item_id))?;
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
            if empty { 191.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        if empty {
            self.width = 191.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
        self.record_snapshot(&snapshot);
        self.process_pending_managed_deletions()?;
        let snapshot = self.snapshot()?;
        Ok(snapshot)
    }

    fn clear_temporary(&mut self) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let _ = Self::queue_managed_deletions_for_unpinned(&transaction)?;
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
            if empty { 191.0 } else { self.width },
            if empty { 180.0 } else { self.height },
            &self.availability,
        )?;
        transaction.commit()?;
        if empty {
            self.width = 191.0;
            self.height = 180.0;
            self.pending_manual_height = None;
        } else {
            self.width = snapshot.preferred_width;
            self.height = snapshot.height;
        }
        self.record_snapshot(&snapshot);
        self.process_pending_managed_deletions()?;
        let snapshot = self.snapshot()?;
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
            "SELECT si.id, si.batch_id, si.position, si.path, si.name, si.is_folder, si.is_pinned
             FROM shelf_items si
             WHERE si.batch_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM managed_files mf
                   WHERE mf.item_id = si.id AND mf.state = 'staged'
               )
             ORDER BY si.position ASC",
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
        let updated = transaction.execute(
            "UPDATE shelf_items SET is_pinned = ?1
             WHERE id = ?2
               AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = ?2 AND mf.state = 'staged')",
            params![pinned, item_id],
        )?;
        if updated == 0
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
            "UPDATE shelf_items SET is_pinned = ?1
             WHERE batch_id = ?2
               AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = shelf_items.id AND mf.state = 'staged')",
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

    fn item_path(&self, item_id: &str) -> Result<Option<String>, ShelfError> {
        Ok(self
            .connection
            .query_row(
                "SELECT si.path FROM shelf_items si
                 WHERE si.id = ?1
                   AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')",
                [item_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    fn item_visual_target(&self, item_id: &str) -> Result<Option<(String, bool)>, ShelfError> {
        Ok(self
            .connection
            .query_row(
                "SELECT si.path, si.is_folder FROM shelf_items si
                 WHERE si.id = ?1
                   AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')",
                [item_id],
                |row| Ok((row.get(0)?, row.get::<_, bool>(1)?)),
            )
            .optional()?)
    }
}

fn random_id(transaction: &Transaction<'_>) -> Result<String, rusqlite::Error> {
    transaction.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
}

fn clear_sizing_when_empty(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
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

fn automatic_height(width: f64, batches: usize) -> f64 {
    let columns = ((width - 12.0) / 168.0).floor().max(1.0) as usize;
    let rows = batches.div_ceil(columns);
    (52.0 * rows.min(5) as f64 + 74.0).max(180.0)
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
    fn automatic_sizing_fits_rows_and_shrinks_after_removal_and_width_changes() {
        let directory = TestDirectory::new("sizing");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        for (index, expected) in [180.0, 180.0, 230.0, 282.0, 334.0, 334.0].into_iter().enumerate() {
            let snapshot = state.accept_drop(vec![Some(format!("C:\\{index}.txt"))]).unwrap().snapshot.unwrap();
            assert_eq!(snapshot.height, expected, "accepted batch {}", index + 1);
        }
        for (width, expected) in [(348.0, 230.0), (516.0, 180.0), (347.0, 334.0)] {
            assert_eq!(state.save_size(width, None).unwrap().height, expected);
        }
        for expected in [334.0, 282.0, 230.0, 180.0, 180.0] {
            let id = state.snapshot().unwrap().batches[0].id.clone();
            assert_eq!(state.remove_batch(&id).unwrap().height, expected);
        }
        drop(state);
        let restored = ShelfState::open(database).unwrap();
        let snapshot = restored.snapshot().unwrap();
        assert_eq!((snapshot.preferred_width, snapshot.height), (347.0, 180.0));
        let empty = restored.remove_batch(&snapshot.batches[0].id).unwrap();
        assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));
    }

    #[test]
    fn batch_pruning_reclaims_automatic_height_but_preserves_manual_height() {
        for manual in [None, Some(400.0)] {
            for operation in ["remove-batch", "remove-item", "clear", "item-copy", "batch-copy", "missing"] {
                let directory = TestDirectory::new(operation);
                let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
                for index in 0..4 {
                    let snapshot = state.accept_drop(vec![Some(format!("C:\\keep-{index}.txt"))]).unwrap().snapshot.unwrap();
                    state.set_batch_pinned(&snapshot.batches[0].id, true).unwrap();
                }
                let incoming = if operation == "remove-item" {
                    vec![Some("C:\\remove-a.txt".into()), Some("C:\\remove-b.txt".into())]
                } else { vec![Some("C:\\remove-a.txt".into())] };
                let snapshot = state.accept_drop(incoming).unwrap().snapshot.unwrap();
                let batch = &snapshot.batches[0];
                state.save_size(191.0, manual).unwrap();
                let result = match operation {
                    "remove-batch" => state.remove_batch(&batch.id).unwrap(),
                    "remove-item" => {
                        let unchanged = state.remove_item(&batch.items[0].id).unwrap();
                        assert_eq!(unchanged.height, manual.unwrap_or(334.0));
                        state.remove_item(&batch.items[1].id).unwrap()
                    }
                    "clear" => state.clear_temporary().unwrap(),
                    "item-copy" => state.complete_item_drag(&batch.items[0].id, DragOutcome::Copy).unwrap(),
                    "batch-copy" => {
                        let prepared = state.prepare_batch_drag_with(&batch.id, |_| super::Availability::Available).unwrap();
                        state.complete_batch_drag(prepared.token.as_deref().unwrap(), DragOutcome::Copy).unwrap()
                    }
                    "missing" => state.prepare_batch_drag_with(&batch.id, |_| super::Availability::Missing).unwrap().snapshot,
                    _ => unreachable!(),
                };
                assert_eq!(result.batches.len(), 4, "{operation}");
                assert_eq!(result.height, manual.unwrap_or(282.0), "{operation}");
                assert_eq!(state.snapshot().unwrap().height, result.height, "{operation}");
            }
        }
    }

    #[test]
    fn restored_width_respects_minimum() {
        let directory = TestDirectory::new("minimum-width");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        state.accept_drop(vec![Some("C:\\first.txt".into())]).unwrap();
        state.accept_drop(vec![Some("C:\\second.txt".into())]).unwrap();
        state.save_size(191.0, None).unwrap();
        drop(state);
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection.execute("UPDATE shelf_settings SET preferred_width = 180", []).unwrap();
        drop(connection);
        let restored = ShelfState::open(&database).unwrap();
        let snapshot = restored.snapshot().unwrap();
        assert_eq!(snapshot.preferred_width, 191.0);
        assert_eq!(snapshot.batches.len(), 2);
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
        assert_eq!(after.preferred_width, 191.0);
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
        assert_eq!(cleared.preferred_width, 191.0);
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
    fn save_size_validation_rejects_below_minimum_and_non_finite() {
        let directory = TestDirectory::new("size-validation");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();

        assert!(matches!(
            state.save_size(190.9, None),
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
        assert_eq!(grown.height, 282.0);

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
            assert_eq!(state.snapshot().unwrap().height, 282.0);
            // Set preferred width only (None for manual height)
            state.save_size(300.0, None).unwrap();
            // Removing complete batches reclaims unused automatic height immediately.
            let snap = state.snapshot().unwrap();
            for batch in &snap.batches[..3] {
                state.remove_batch(&batch.id).unwrap();
            }
            assert_eq!(state.snapshot().unwrap().height, 180.0);
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
    fn final_item_removal_resets_manual_and_staged_sizing_to_minimum() {
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
        assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));

        // Next intake after final removal defaults to 191x180.
        let reintake = state
            .accept_drop(vec![Some("C:\\batch2.txt".into())])
            .unwrap();
        assert_eq!(
            (
                reintake.snapshot.as_ref().unwrap().preferred_width,
                reintake.snapshot.as_ref().unwrap().height
            ),
            (191.0, 180.0)
        );

        // 2. Remove final item via consume_items (Copy outcome)
        state.save_size(270.0, Some(230.0)).unwrap();
        let snap = state.snapshot().unwrap();
        let empty = state
            .complete_item_drag(&snap.batches[0].items[0].id, DragOutcome::Copy)
            .unwrap();
        assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));

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
            (191.0, 180.0)
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

    #[test]
    fn corrupt_database_with_wal_and_shm_is_preserved_byte_for_byte_and_recovers_empty_shelf() {
        let directory = TestDirectory::new("recovery-corrupt-all");
        let database = directory.path().join("shelf.db");
        let wal = directory.path().join("shelf.db-wal");
        let shm = directory.path().join("shelf.db-shm");

        let corrupt_db_bytes = b"corrupted sqlite database header and content";
        let corrupt_wal_bytes = b"corrupted wal content";
        let corrupt_shm_bytes = b"corrupted shm content";

        fs::write(&database, corrupt_db_bytes).unwrap();
        fs::write(&wal, corrupt_wal_bytes).unwrap();
        fs::write(&shm, corrupt_shm_bytes).unwrap();

        let (state, backup_dir) =
            ShelfState::open_or_recover(&database).expect("recovery should succeed");
        let backup_dir = backup_dir.expect("backup directory should be returned");

        assert!(backup_dir.exists(), "backup directory must exist");
        assert_eq!(
            backup_dir.parent().unwrap(),
            directory.path().join("recovery"),
            "backup directory must live under profile recovery area"
        );
        let dir_name = backup_dir.file_name().unwrap().to_string_lossy();
        assert!(
            dir_name.starts_with("shelf-corrupt-"),
            "backup directory must start with shelf-corrupt-, got {dir_name}"
        );

        assert_eq!(
            fs::read(backup_dir.join("shelf.db")).unwrap(),
            corrupt_db_bytes
        );
        assert_eq!(
            fs::read(backup_dir.join("shelf.db-wal")).unwrap(),
            corrupt_wal_bytes
        );
        assert_eq!(
            fs::read(backup_dir.join("shelf.db-shm")).unwrap(),
            corrupt_shm_bytes
        );

        if wal.exists() {
            assert_ne!(
                fs::read(&wal).unwrap(),
                corrupt_wal_bytes,
                "active wal must not contain original corrupt bytes"
            );
        }
        if shm.exists() {
            assert_ne!(
                fs::read(&shm).unwrap(),
                corrupt_shm_bytes,
                "active shm must not contain original corrupt bytes"
            );
        }

        let snapshot = state
            .snapshot()
            .expect("recovered shelf snapshot should load");
        assert!(snapshot.batches.is_empty(), "recovered shelf must start empty");
    }

    #[test]
    fn recovery_accepts_missing_sidecars_and_preserves_only_existing() {
        // Subcase A: No sidecars
        {
            let directory = TestDirectory::new("recovery-no-sidecars");
            let database = directory.path().join("shelf.db");
            fs::write(&database, b"corrupted db without sidecars").unwrap();

            let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
            let backup = backup_dir.unwrap();
            assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
            assert_eq!(
                fs::read(backup.join("shelf.db")).unwrap(),
                b"corrupted db without sidecars"
            );
            assert!(!backup.join("shelf.db-wal").exists());
            assert!(!backup.join("shelf.db-shm").exists());
            assert!(state.snapshot().unwrap().batches.is_empty());
        }

        // Subcase B: WAL only
        {
            let directory = TestDirectory::new("recovery-wal-only");
            let database = directory.path().join("shelf.db");
            let wal = directory.path().join("shelf.db-wal");
            fs::write(&database, b"corrupted db").unwrap();
            fs::write(&wal, b"wal only").unwrap();

            let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
            let backup = backup_dir.unwrap();
            assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
            assert_eq!(fs::read(backup.join("shelf.db")).unwrap(), b"corrupted db");
            assert_eq!(fs::read(backup.join("shelf.db-wal")).unwrap(), b"wal only");
            assert!(!backup.join("shelf.db-shm").exists());
            assert!(state.snapshot().unwrap().batches.is_empty());
        }

        // Subcase C: SHM only
        {
            let directory = TestDirectory::new("recovery-shm-only");
            let database = directory.path().join("shelf.db");
            let shm = directory.path().join("shelf.db-shm");
            fs::write(&database, b"corrupted db").unwrap();
            fs::write(&shm, b"shm only").unwrap();

            let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
            let backup = backup_dir.unwrap();
            assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
            assert_eq!(fs::read(backup.join("shelf.db")).unwrap(), b"corrupted db");
            assert_eq!(fs::read(backup.join("shelf.db-shm")).unwrap(), b"shm only");
            assert!(!backup.join("shelf.db-wal").exists());
            assert!(state.snapshot().unwrap().batches.is_empty());
        }
    }

    #[test]
    fn recovery_backup_name_collision_resolves_deterministic_numeric_suffix_under_recovery_area() {
        use std::time::{Duration, UNIX_EPOCH};
        let directory = TestDirectory::new("recovery-collision");
        let database = directory.path().join("shelf.db");
        let fixed_time = UNIX_EPOCH + Duration::from_secs(1776182400); // 2026-04-14T16:00:00Z

        // First corrupt episode
        fs::write(&database, b"first corrupt db").unwrap();
        let (state1, backup1) =
            ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
        let backup1 = backup1.unwrap();
        assert_eq!(
            backup1,
            directory
                .path()
                .join("recovery")
                .join("shelf-corrupt-20260414T160000Z")
        );
        assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
        assert!(state1.snapshot().unwrap().batches.is_empty());
        drop(state1);

        // Second corrupt episode at same timestamp -> must produce -1
        fs::write(&database, b"second corrupt db").unwrap();
        let (state2, backup2) =
            ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
        let backup2 = backup2.unwrap();
        assert_eq!(
            backup2,
            directory
                .path()
                .join("recovery")
                .join("shelf-corrupt-20260414T160000Z-1")
        );
        assert_eq!(fs::read(backup2.join("shelf.db")).unwrap(), b"second corrupt db");
        // Prior backup1 must be completely untouched
        assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
        assert!(state2.snapshot().unwrap().batches.is_empty());
        drop(state2);

        // Third corrupt episode at same timestamp -> must produce -2
        fs::write(&database, b"third corrupt db").unwrap();
        let (state3, backup3) =
            ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
        let backup3 = backup3.unwrap();
        assert_eq!(
            backup3,
            directory
                .path()
                .join("recovery")
                .join("shelf-corrupt-20260414T160000Z-2")
        );
        assert_eq!(fs::read(backup3.join("shelf.db")).unwrap(), b"third corrupt db");
        assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
        assert_eq!(fs::read(backup2.join("shelf.db")).unwrap(), b"second corrupt db");
        assert!(state3.snapshot().unwrap().batches.is_empty());
    }

    #[test]
    fn clean_and_missing_database_do_not_create_recovery_backup() {
        let directory = TestDirectory::new("recovery-clean");
        let database = directory.path().join("shelf.db");
        let recovery_area = directory.path().join("recovery");

        // 1. Missing database
        assert!(!database.exists());
        let (state, backup) = ShelfState::open_or_recover(&database).unwrap();
        assert!(backup.is_none(), "missing db must not create recovery backup");
        assert!(!recovery_area.exists(), "recovery directory must not be created");
        assert!(database.exists(), "clean database must be initialized");
        assert!(state.snapshot().unwrap().batches.is_empty());

        // Populate database
        state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap();
        drop(state);

        // 2. Existing clean database
        let (reopened, backup) = ShelfState::open_or_recover(&database).unwrap();
        assert!(backup.is_none(), "clean existing db must not create recovery backup");
        assert!(!recovery_area.exists(), "recovery directory must not be created");
        assert_eq!(reopened.snapshot().unwrap().batches.len(), 1);
    }

    #[test]
    fn schema_invalid_database_is_recovered_and_preserves_evidence() {
        let directory = TestDirectory::new("recovery-schema-invalid");
        let database = directory.path().join("shelf.db");

        // Create a sqlite database with wrong columns in shelf_batches
        {
            let conn = rusqlite::Connection::open(&database).unwrap();
            conn.execute_batch(
                "CREATE TABLE shelf_batches (id TEXT PRIMARY KEY, wrong_column INTEGER);",
            )
            .unwrap();
        }
        let original_bytes = fs::read(&database).unwrap();

        let (state, backup) = ShelfState::open_or_recover(&database).unwrap();
        let backup_dir = backup.expect("schema-invalid db must trigger recovery");
        assert_eq!(backup_dir.parent().unwrap(), directory.path().join("recovery"));
        assert_eq!(fs::read(backup_dir.join("shelf.db")).unwrap(), original_bytes);

        // Clean database now has correct schema and starts empty
        assert!(state.snapshot().unwrap().batches.is_empty());
        // Normal operations work on the clean database
        let dropped = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap();
        assert!(dropped.snapshot.is_some());
    }

    #[test]
    fn unrecoverable_filesystem_failure_aborts_without_clean_shelf_or_data_loss() {
        let directory = TestDirectory::new("recovery-fs-fail");
        let database = directory.path().join("shelf.db");
        let wal = directory.path().join("shelf.db-wal");

        let corrupt_db = b"corrupted db content";
        let corrupt_wal = b"corrupted wal content";
        fs::write(&database, corrupt_db).unwrap();
        fs::write(&wal, corrupt_wal).unwrap();

        // Block the recovery directory by creating a regular file named "recovery"
        fs::write(directory.path().join("recovery"), b"blocking file").unwrap();

        let result = ShelfState::open_or_recover(&database);
        assert!(result.is_err(), "must fail fatally when recovery directory cannot be created");

        // Original evidence must remain intact and untouched
        assert_eq!(fs::read(&database).unwrap(), corrupt_db);
        assert_eq!(fs::read(&wal).unwrap(), corrupt_wal);
    }

    #[test]
    fn recovery_partial_move_failure_restores_moved_first_artifact() {
        use std::time::{Duration, UNIX_EPOCH};
        let directory = TestDirectory::new("recovery-partial-move");
        let database = directory.path().join("shelf.db");
        let wal = directory.path().join("shelf.db-wal");
        let fixed_time = UNIX_EPOCH + Duration::from_secs(1776182400);

        let corrupt_db = b"corrupted db content";
        let corrupt_wal = b"corrupted wal content";
        fs::write(&database, corrupt_db).unwrap();
        fs::write(&wal, corrupt_wal).unwrap();
        #[cfg(windows)]
        use std::os::windows::fs::OpenOptionsExt;
        let _wal_lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(1 | 2) // FILE_SHARE_READ | FILE_SHARE_WRITE (no FILE_SHARE_DELETE)
            .open(&wal)
            .unwrap();

        let result = ShelfState::open_or_recover_at(&database, fixed_time);
        assert!(result.is_err(), "must fail when second artifact move fails");

        drop(_wal_lock);

        // First artifact (database) was moved then restored back to source
        assert_eq!(fs::read(&database).unwrap(), corrupt_db);
        assert_eq!(fs::read(&wal).unwrap(), corrupt_wal);
    }

    #[test]
    fn recovery_warning_delivery_failure_retains_snapshot_warning_until_acknowledged() {
        let directory = TestDirectory::new("recovery-warning-fallback");
        let database = directory.path().join("shelf.db");
        fs::write(&database, b"corrupted db content").unwrap();

        let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
        let backup_path = backup_dir.expect("recovery backup should exist");

        // Simulate notification-area delivery failure at the startup seam
        let mut attempted_message = None;
        state.deliver_recovery_warning(Some(&backup_path), |msg| {
            attempted_message = Some(msg.to_owned());
            Err("notification channel unavailable".into())
        });

        let msg = attempted_message.expect("notifier should be called with warning message");
        assert!(
            msg.contains(&backup_path.display().to_string()),
            "warning must include Recovery Backup path"
        );
        assert!(
            msg.contains("empty Shelf"),
            "warning must identify that Shelf started empty"
        );

        // Warning is exposed on ShelfSnapshot and retained across multiple reads before dismissal
        let first_read = state.snapshot().unwrap();
        assert_eq!(first_read.warning.as_deref(), Some(msg.as_str()));
        let second_read = state.snapshot().unwrap();
        assert_eq!(second_read.warning.as_deref(), Some(msg.as_str()));

        // Acknowledging warning after user dismissal clears it from subsequent snapshots
        state.acknowledge_warning();
        assert_eq!(state.snapshot().unwrap().warning, None);
    }

    #[test]
    fn valid_header_corrupt_btree_with_wal_and_shm_preserves_shm_bytes_without_wal_index_overwrite() {
        let directory = TestDirectory::new("recovery-valid-header-shm");
        let database = directory.path().join("shelf.db");
        let wal = directory.path().join("shelf.db-wal");
        let shm = directory.path().join("shelf.db-shm");

        // Create a real WAL-mode SQLite database and close the connection cleanly
        {
            let state = ShelfState::open(&database).unwrap();
            state
                .accept_drop(vec![Some(
                    directory.source_file().to_string_lossy().into_owned(),
                )])
                .unwrap();
            state.close().unwrap();
        }

        // Corrupt the interior B-tree page data while keeping the 16-byte SQLite format 3 header intact
        let mut corrupt_db_bytes = fs::read(&database).unwrap();
        for byte in &mut corrupt_db_bytes[100..200] {
            *byte = 0xFF;
        }
        let corrupt_wal_bytes = b"original WAL evidence";
        let corrupt_shm_bytes = b"original SHM evidence";

        fs::write(&database, &corrupt_db_bytes).unwrap();
        fs::write(&wal, corrupt_wal_bytes).unwrap();
        fs::write(&shm, corrupt_shm_bytes).unwrap();

        let (state, backup_dir) =
            ShelfState::open_or_recover(&database).expect("recovery should succeed");
        let backup_dir = backup_dir.expect("backup directory should be returned");

        assert_eq!(
            fs::read(backup_dir.join("shelf.db")).unwrap(),
            corrupt_db_bytes,
            "preserved shelf.db must match original corrupt bytes"
        );
        assert_eq!(
            fs::read(backup_dir.join("shelf.db-wal")).unwrap(),
            corrupt_wal_bytes,
            "preserved shelf.db-wal must match original WAL evidence"
        );
        assert_eq!(
            fs::read(backup_dir.join("shelf.db-shm")).unwrap(),
            corrupt_shm_bytes,
            "preserved shelf.db-shm must be the exact 21-byte original SHM evidence, not overwritten by 32KB WAL index"
        );
        assert!(state.snapshot().unwrap().batches.is_empty());
    }

    #[test]
    fn failed_batch_intake_preserves_visible_and_durable_state() {
        let directory = TestDirectory::new("intake-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Populate initial batch
        let initial = state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap()
            .snapshot
            .unwrap();
        let initial_json = serde_json::to_value(&initial).unwrap();

        // Install trigger that fails item inserts
        state.install_test_insert_failure().unwrap();

        // Attempt to accept a new batch
        let fail_path = directory.0.join("new_file.txt");
        fs::write(&fail_path, b"new").unwrap();
        let result = state.accept_drop(vec![Some(fail_path.to_string_lossy().into_owned())]);
        assert!(result.is_err(), "Intake must fail when insert fails");

        // Verify visible snapshot is identical to prior snapshot
        let visible = state.snapshot().unwrap();
        assert_eq!(
            serde_json::to_value(&visible).unwrap(),
            initial_json,
            "Visible snapshot must be unmodified after failed intake"
        );

        // Reopen database and verify durable snapshot matches prior state
        drop(visible);
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        let durable = reopened.snapshot().unwrap();
        assert_eq!(
            serde_json::to_value(&durable).unwrap(),
            initial_json,
            "Durable snapshot after restart must match prior state"
        );
    }

    #[test]
    fn failed_batch_removal_preserves_visible_and_durable_state() {
        let directory = TestDirectory::new("remove-batch-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        state
            .accept_drop(vec![Some(file_a.to_string_lossy().into_owned())])
            .unwrap();
        let snap = state
            .accept_drop(vec![Some(file_b.to_string_lossy().into_owned())])
            .unwrap()
            .snapshot
            .unwrap();
        let initial_json = serde_json::to_value(&snap).unwrap();
        let target_batch_id = snap.batches[0].id.clone();

        // Install trigger that fails deletes on shelf_batches
        state
            .store()
            .unwrap()
            .connection
            .execute_batch(
                "CREATE TEMP TRIGGER fail_batch_delete BEFORE DELETE ON shelf_batches
                 BEGIN SELECT RAISE(FAIL, 'forced batch delete failure'); END;",
            )
            .unwrap();

        // Attempt to remove batch
        assert!(state.remove_batch(&target_batch_id).is_err());

        // Visible snapshot is preserved
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            initial_json
        );

        // Durable snapshot on reopen is preserved
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn failed_batch_drag_copy_consumption_preserves_visible_and_durable_state() {
        let directory = TestDirectory::new("batch-copy-rollback");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let file_a = directory.0.join("a.txt");
        let file_b = directory.0.join("b.txt");
        fs::write(&file_a, b"a").unwrap();
        fs::write(&file_b, b"b").unwrap();

        let snap = state
            .accept_drop(vec![
                Some(file_a.to_string_lossy().into_owned()),
                Some(file_b.to_string_lossy().into_owned()),
            ])
            .unwrap()
            .snapshot
            .unwrap();
        let initial_json = serde_json::to_value(&snap).unwrap();
        let batch_id = snap.batches[0].id.clone();

        let prep = state.prepare_batch_drag(&batch_id).unwrap();
        let token = prep.token.unwrap();

        // Install trigger that fails item deletes
        state.install_test_delete_failure().unwrap();

        // Attempt to complete batch drag as Copy (consumption)
        let result = state.complete_batch_drag(&token, DragOutcome::Copy);
        assert!(result.is_err(), "Copy completion must fail if delete fails");

        // Visible snapshot retains prior state
        assert_eq!(
            serde_json::to_value(state.snapshot().unwrap()).unwrap(),
            initial_json
        );

        // Durable snapshot on reopen is preserved
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(
            serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
            initial_json
        );
    }

    #[test]
    fn failed_rail_monitor_persistence_preserves_prior_durable_value_and_reopen() {
        let directory = TestDirectory::new("rail-mon-fail");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Initially set remembered rail monitor to "display-1"
        state.remember_rail_monitor(Some("display-1")).unwrap();
        assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

        // Verify durable state on reopen
        drop(state);
        let state = ShelfState::open(&database).unwrap();
        assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

        // Install trigger failing shelf_ui_state updates and inserts
        state.install_test_rail_monitor_failure().unwrap();

        // Attempt to update rail monitor to "display-2"
        let res = state.remember_rail_monitor(Some("display-2"));
        assert!(res.is_err(), "Updating rail monitor must fail when trigger aborts");

        // Querying rail_monitor returns the unchanged durable value ("display-1")
        assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

        // Reopen database: restart reflects the actual durable value ("display-1")
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        assert_eq!(reopened.rail_monitor().unwrap().as_deref(), Some("display-1"));
    }

    #[test]
    fn accept_text_round_trip_utf8_restoration_and_copy_consumption() {
        let directory = TestDirectory::new("accept-text-red");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let sample_text = "DropCove 0.0.23 plain-text test\nLine 2: 🚀 Unicode characters\r\nLine 3: https://example.com/path?arg=1";
        let result = state.accept_text(sample_text.to_string()).unwrap();
        let snapshot = result.snapshot.expect("accept_text must produce a snapshot");
        assert_eq!(snapshot.batches.len(), 1, "exactly one batch created");
        let batch = &snapshot.batches[0];
        assert_eq!(batch.items.len(), 1, "batch has exactly one item");
        let item = &batch.items[0];
        assert!(!item.is_folder);
        assert!(!item.is_pinned);
        assert_eq!(item.name, "DropCove 0.0.23 plain-tex...");
        assert!(item.path.ends_with(".txt"));

        let item_path = std::path::Path::new(&item.path);
        assert!(item_path.is_file(), "managed file must exist on disk");
        let disk_bytes = std::fs::read(item_path).expect("must read managed file");
        assert_eq!(disk_bytes, sample_text.as_bytes(), "file bytes must match UTF-8 without BOM");

        // Restart verification
        drop(state);
        let reopened = ShelfState::open(&database).unwrap();
        let reopened_snapshot = reopened.snapshot().unwrap();
        assert_eq!(reopened_snapshot.batches.len(), 1);
        let restored_item = &reopened_snapshot.batches[0].items[0];
        assert_eq!(restored_item.id, item.id);
        assert_eq!(restored_item.path, item.path);

        // Copy drag-out consumption permanent deletion
        let prep = reopened.prepare_batch_drag(&batch.id).unwrap();
        let token = prep.token.expect("token should be generated");
        let consumed_snap = reopened
            .complete_batch_drag(&token, DragOutcome::Copy)
            .unwrap();
        assert!(consumed_snap.batches.is_empty(), "temporary item consumed");
        assert!(!item_path.exists(), "managed file permanently deleted after Copy consumption");
    }

    #[test]
    fn accept_text_boundary_and_rejections() {
        let directory = TestDirectory::new("accept-text-bounds");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Empty text is rejected
        let res = state.accept_text("".to_string()).unwrap();
        assert!(res.snapshot.is_none());
        assert_eq!(res.skipped, 0);

        // Terminal NUL only text is rejected
        let res = state.accept_text("\0".to_string()).unwrap();
        assert!(res.snapshot.is_none());
        assert_eq!(res.skipped, 0);

        // Whitespace-only text is accepted
        let res = state.accept_text("   \n\t  ".to_string()).unwrap();
        assert!(res.snapshot.is_some());
        let snap = res.snapshot.unwrap();
        assert_eq!(snap.batches.len(), 1);

        // 10 MiB text exact boundary is accepted
        let exact_10mb = vec![b'a'; 10 * 1024 * 1024];
        let text_10mb = String::from_utf8(exact_10mb).unwrap();
        let res = state.accept_text(text_10mb).unwrap();
        assert!(res.snapshot.is_some());

        // 10 MiB + 1 byte is rejected
        let over_10mb = vec![b'b'; 10 * 1024 * 1024 + 1];
        let text_over = String::from_utf8(over_10mb).unwrap();
        let res = state.accept_text(text_over).unwrap();
        assert!(res.snapshot.is_none());
        assert_eq!(res.skipped, 1);
    }

    #[test]
    fn managed_file_sharing_violation_retry_and_external_safety() {
        let directory = TestDirectory::new("sharing-retry");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // External file drop
        let external_file = directory.path().join("external_source.txt");
        fs::write(&external_file, b"safe external bytes").unwrap();
        let _drop_res = state
            .accept_drop(vec![Some(external_file.display().to_string())])
            .unwrap();

        // Managed text drop
        let text_res = state.accept_text("managed text content".to_string()).unwrap();
        let text_snap = text_res.snapshot.unwrap();
        let text_batch = &text_snap.batches[0];
        let text_item = &text_batch.items[0];
        let text_path = PathBuf::from(&text_item.path);

        // Hold the managed file open on Windows denying delete
        #[cfg(windows)]
        use std::os::windows::fs::OpenOptionsExt;
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(1 | 2) // FILE_SHARE_READ | FILE_SHARE_WRITE (no FILE_SHARE_DELETE)
            .open(&text_path)
            .unwrap();

        // Remove batch while locked: mutation commits in SQLite, pending deletion queued, file remains
        let snap_after_remove = state.remove_batch(&text_batch.id).unwrap();
        assert!(text_path.exists(), "file remains on disk due to sharing violation");
        assert_eq!(
            snap_after_remove.warning.as_deref(),
            Some("1 DropCove-created file is pending deletion.")
        );

        // External file must remain untouched
        assert_eq!(fs::read(&external_file).unwrap(), b"safe external bytes");

        // Release lock
        drop(lock);

        // Later mutation triggers cleanup retry
        let retry_snap = state.clear_temporary().unwrap();
        assert!(!text_path.exists(), "managed file deleted after lock release and retry");
        assert!(retry_snap.warning.is_none(), "pending warning cleared");
        // External file was temporary so reference removed, but source object remains untouched!
        assert_eq!(fs::read(&external_file).unwrap(), b"safe external bytes");
    }

    #[test]
    fn managed_file_recovery_preserves_managed_and_staging_roots() {
        let directory = TestDirectory::new("recovery-managed-preservation");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        let text_res = state.accept_text("recovery text bytes".to_string()).unwrap();
        let text_snap = text_res.snapshot.unwrap();
        let text_item = &text_snap.batches[0].items[0];
        let text_path = PathBuf::from(&text_item.path);
        assert!(text_path.exists());

        // Also create a dummy file in staging
        let staging_file = directory.path().join("staging").join("0123456789abcdef0123456789abcdef.tmp");
        fs::write(&staging_file, b"staged bytes").unwrap();

        // Corrupt the database
        drop(state);
        fs::write(&database, b"corrupted shelf db").unwrap();

        // Trigger open_or_recover
        let (reopened, backup_dir_opt) = ShelfState::open_or_recover(&database).unwrap();
        let backup_dir = backup_dir_opt.expect("recovery backup must be created");

        // Active managed root is now clean and empty
        let active_snap = reopened.snapshot().unwrap();
        assert!(active_snap.batches.is_empty(), "clean empty shelf on recovery");
        assert!(!text_path.exists(), "old managed file moved from active store");

        // Backup directory must preserve database, managed root, and staging root
        assert!(backup_dir.join("shelf.db").exists());
        let preserved_managed = backup_dir.join("managed");
        assert!(preserved_managed.exists(), "backup must contain managed root");
        let preserved_staging = backup_dir.join("staging");
        assert!(preserved_staging.exists(), "backup must contain staging root");
        assert!(preserved_staging.join("0123456789abcdef0123456789abcdef.tmp").exists());
    }

    #[test]
    fn startup_reconciles_promoted_and_staged_states() {
        let directory = TestDirectory::new("crash-reconciliation");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();

        // Seed state: a staged managed file that was promoted to final path before crash
        let store = crate::managed_files::ManagedStore::new(directory.path());
        store.init_directories().unwrap();
        let item_id = "0123456789abcdef0123456789abcdef";
        let filename = "Dropped text 2026-10-23 12-00-00.txt";
        let staging_rel = crate::managed_files::ManagedStore::staging_rel_path(item_id);
        let final_rel = crate::managed_files::ManagedStore::final_rel_path(item_id, filename);
        let final_full = store.full_path(&final_rel).unwrap();
        if let Some(p) = final_full.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(&final_full, b"promoted bytes").unwrap();

        // Seed SQLite as 'staged'
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b1', 100, 0)", []).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b1', 0, ?2, ?3, 0, 0)",
                [item_id, final_full.to_str().unwrap(), filename],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [item_id, &final_rel, &staging_rel],
            ).unwrap();
        }
        drop(state);

        // Reopen database: startup reconciliation must promote 'staged' to 'ready'
        let reopened = ShelfState::open(&database).unwrap();
        let snap = reopened.snapshot().unwrap();
        assert_eq!(snap.batches.len(), 1, "reconciled item becomes visible");
        assert_eq!(snap.batches[0].items[0].id, item_id);
        assert_eq!(fs::read(&final_full).unwrap(), b"promoted bytes");
    }

    #[test]
    fn missing_managed_file_removal_also_removes_its_empty_directory() {
        let directory = TestDirectory::new("managed-missing-directory");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let snapshot = state.accept_text("discard me".into()).unwrap().snapshot.unwrap();
        let batch = &snapshot.batches[0];
        let file = PathBuf::from(&batch.items[0].path);
        let item_directory = file.parent().unwrap().to_path_buf();
        fs::remove_file(&file).unwrap();
        state.remove_batch(&batch.id).unwrap();
        assert!(!item_directory.exists());
        assert!(state.snapshot().unwrap().warning.is_none());
    }

    #[test]
    fn pinned_managed_copy_retains_file_until_explicit_removal() {
        let directory = TestDirectory::new("managed-pinned-copy");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        let snapshot = state.accept_text("keep these exact bytes".into()).unwrap().snapshot.unwrap();
        let item = &snapshot.batches[0].items[0];
        state.set_item_pinned(&item.id, true).unwrap();
        let retained = state.complete_item_drag(&item.id, DragOutcome::Copy).unwrap();
        assert_eq!(retained.batches[0].items[0].id, item.id);
        assert_eq!(fs::read(&item.path).unwrap(), b"keep these exact bytes");
        state.remove_item(&item.id).unwrap();
        assert!(!Path::new(&item.path).exists());
    }

    #[test]
    fn pending_cleanup_warning_survives_snapshot_and_restart() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = TestDirectory::new("managed-pending-snapshot");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let snapshot = state.accept_text("temporarily locked".into()).unwrap().snapshot.unwrap();
        let batch = &snapshot.batches[0];
        let path = &batch.items[0].path;
        let lock = fs::OpenOptions::new().read(true).share_mode(3).open(path).unwrap();
        state.remove_batch(&batch.id).unwrap();
        assert_eq!(state.snapshot().unwrap().warning.as_deref(), Some("1 DropCove-created file is pending deletion."));
        drop(state);
        let restored = ShelfState::open(&database).unwrap();
        assert_eq!(restored.snapshot().unwrap().warning.as_deref(), Some("1 DropCove-created file is pending deletion."));
        drop(lock);
        restored.clear_temporary().unwrap();
        assert!(!Path::new(path).exists());
        assert!(restored.snapshot().unwrap().warning.is_none());
    }

    #[test]
    fn failed_ready_publication_compensates_without_leaking_managed_bytes() {
        let directory = TestDirectory::new("managed-ready-rollback");
        let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
        state.store().unwrap().connection.execute_batch(
            "CREATE TEMP TRIGGER reject_ready BEFORE UPDATE OF state ON managed_files WHEN NEW.state = 'ready' BEGIN SELECT RAISE(ABORT, 'ready blocked'); END;"
        ).unwrap();
        assert!(state.accept_text("must not appear".into()).is_err());
        assert!(state.snapshot().unwrap().batches.is_empty());
        assert_eq!(fs::read_dir(directory.path().join("managed")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(directory.path().join("staging")).unwrap().count(), 0);
    }

    #[test]
    fn clear_temporary_mixed_ownership_preserves_external_and_cleans_managed() {
        let directory = TestDirectory::new("clear-mixed-ownership");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let external = directory.source_file();
        state.accept_drop(vec![Some(external.to_str().unwrap().to_owned())]).unwrap();
        let managed = state.accept_text("managed text for clear".into()).unwrap();
        let managed_item = &managed.snapshot.unwrap().batches[0].items[0];
        let managed_path = std::path::PathBuf::from(&managed_item.path);
        assert!(managed_path.exists());
        assert!(external.exists());

        // Clear temporary items
        state.clear_temporary().unwrap();
        assert!(!managed_path.exists(), "managed file deleted");
        assert!(external.exists(), "external source object preserved");
        assert_eq!(state.snapshot().unwrap().batches.len(), 0);
    }

    #[test]
    fn remove_item_cleans_only_managed_occurrence_leaving_same_name_external_intact() {
        let directory = TestDirectory::new("remove-mixed-ownership");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let external = directory.source_file();
        state.accept_drop(vec![Some(external.to_str().unwrap().to_owned())]).unwrap();
        let managed = state.accept_text("managed text for remove".into()).unwrap();
        let managed_item = &managed.snapshot.unwrap().batches[0].items[0];
        let managed_path = std::path::PathBuf::from(&managed_item.path);
        assert!(managed_path.exists());
        assert!(external.exists());

        state.remove_item(&managed_item.id).unwrap();
        assert!(!managed_path.exists(), "managed file deleted");
        assert!(external.exists(), "external source object preserved");
        assert_eq!(state.snapshot().unwrap().batches.len(), 1);
    }

    #[test]
    fn image_group_acceptance_ordered_and_copy_lifecycle() {
        let directory = TestDirectory::new("image-group-copy-lifecycle");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "11111111111111111111111111111111";
        let id2 = "22222222222222222222222222222222";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);

        // Minimal 1x1 valid PNG bytes
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        store.stage_bytes(&st1, png_bytes).unwrap();
        store.stage_bytes(&st2, png_bytes).unwrap();

        let candidates = vec![
            (st1.clone(), "first.png".to_string()),
            (st2.clone(), "second.png".to_string()),
        ];
        let res = state.accept_image_files(candidates, 1).unwrap();
        assert_eq!(res.skipped, 1);
        assert_eq!(res.message, Some("Added 2 images; skipped 1.".to_string()));
        let snap = res.snapshot.unwrap();
        assert_eq!(snap.batches.len(), 1);
        let batch = &snap.batches[0];
        assert_eq!(batch.items.len(), 2);
        assert_eq!(batch.items[0].name, "first.png");
        assert_eq!(batch.items[1].name, "second.png");

        let path1 = PathBuf::from(&batch.items[0].path);
        let path2 = PathBuf::from(&batch.items[1].path);
        assert!(path1.exists());
        assert!(path2.exists());

        // Pin first item, leave second temporary
        state.set_item_pinned(&batch.items[0].id, true).unwrap();

        // Complete Copy drag for second item (temporary): consumed & cleaned
        state.complete_item_drag(&batch.items[1].id, DragOutcome::Copy).unwrap();
        assert!(!path2.exists(), "temporary image deleted on Copy completion");

        // Complete Copy drag for first item (pinned): retained
        let retained = state.complete_item_drag(&batch.items[0].id, DragOutcome::Copy).unwrap();
        assert_eq!(retained.batches[0].items.len(), 1);
        assert!(path1.exists(), "pinned image retained after Copy");

        // Explicit removal removes pinned image file
        state.remove_item(&batch.items[0].id).unwrap();
        assert!(!path1.exists(), "explicit removal cleans pinned image file");
    }

    #[test]
    fn image_group_startup_reconciliation_restores_complete_ready_batch() {
        let directory = TestDirectory::new("image-group-reconciliation");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "33333333333333333333333333333333";
        let id2 = "44444444444444444444444444444444";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let fin1 = crate::managed_files::ManagedStore::final_rel_path(id1, "a.png");
        let fin2 = crate::managed_files::ManagedStore::final_rel_path(id2, "b.png");

        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        // Item 1 was promoted before crash
        let full_fin1 = store.full_path(&fin1).unwrap();
        fs::create_dir_all(full_fin1.parent().unwrap()).unwrap();
        fs::write(&full_fin1, png_bytes).unwrap();

        // Item 2 was still in staging before crash
        store.stage_bytes(&st2, png_bytes).unwrap();

        // Seed SQLite as 'staged' for both items in one batch
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_img', 100, 0)", []).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_img', 0, ?2, 'a.png', 0, 0)",
                [id1, full_fin1.to_str().unwrap()],
            ).unwrap();
            let full_fin2 = store.full_path(&fin2).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_img', 1, ?2, 'b.png', 0, 0)",
                [id2, full_fin2.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id1, &fin1, &st1],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id2, &fin2, &st2],
            ).unwrap();
        }
        drop(state);

        // Reopen database: startup reconciliation must promote both and mark Ready atomically
        let reopened = ShelfState::open(&database).unwrap();
        let snap = reopened.snapshot().unwrap();
        assert_eq!(snap.batches.len(), 1, "reconciled image batch becomes visible");
        assert_eq!(snap.batches[0].items.len(), 2);
        assert_eq!(snap.batches[0].items[0].id, id1);
        assert_eq!(snap.batches[0].items[1].id, id2);
    }

    #[test]
    fn image_group_startup_reconciliation_rolls_back_if_any_sibling_corrupt() {
        let directory = TestDirectory::new("image-group-sibling-fail");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "55555555555555555555555555555555";
        let id2 = "66666666666666666666666666666666";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let fin1 = crate::managed_files::ManagedStore::final_rel_path(id1, "ok.png");
        let fin2 = crate::managed_files::ManagedStore::final_rel_path(id2, "bad.png");

        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        // Item 1 has valid PNG in staging
        store.stage_bytes(&st1, png_bytes).unwrap();
        // Item 2 has corrupt bytes in staging
        store.stage_bytes(&st2, b"not a png").unwrap();

        // Seed SQLite as 'staged' for both items in one batch
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_fail', 100, 0)", []).unwrap();
            let full_fin1 = store.full_path(&fin1).unwrap();
            let full_fin2 = store.full_path(&fin2).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_fail', 0, ?2, 'ok.png', 0, 0)",
                [id1, full_fin1.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_fail', 1, ?2, 'bad.png', 0, 0)",
                [id2, full_fin2.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id1, &fin1, &st1],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id2, &fin2, &st2],
            ).unwrap();
        }
        drop(state);

        // Reopen database: sibling failure means NO batch is published, all are discarded
        let reopened = ShelfState::open(&database).unwrap();
        let snap = reopened.snapshot().unwrap();
        assert_eq!(snap.batches.len(), 0, "corrupt sibling discards entire batch");
    }

    #[test]
    fn image_group_insert_staged_transaction_failure_cleans_up_staging_files() {
        let directory = TestDirectory::new("image-group-tx-fail");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "11111111111111111111111111111111";
        let id2 = "22222222222222222222222222222222";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let path1 = store.stage_bytes(&st1, png_bytes).unwrap();
        let path2 = store.stage_bytes(&st2, png_bytes).unwrap();
        assert!(path1.exists());
        assert!(path2.exists());

        // Inject trigger to fail the staged transaction on insert into shelf_batches
        state.store().unwrap().connection.execute_batch(
            "CREATE TEMP TRIGGER abort_batch_insert BEFORE INSERT ON shelf_batches BEGIN SELECT RAISE(ABORT, 'staged tx blocked'); END;"
        ).unwrap();

        let candidates = vec![
            (st1.clone(), "img1.png".to_string()),
            (st2.clone(), "img2.png".to_string()),
        ];
        let res = state.accept_image_files(candidates, 0);
        assert!(res.is_err());

        // Staging files must be cleaned up immediately, and no shelf batches exist
        assert!(!path1.exists(), "staging file 1 deleted on tx failure");
        assert!(!path2.exists(), "staging file 2 deleted on tx failure");
        let snap = state.snapshot().unwrap();
        assert!(snap.batches.is_empty());
    }

    #[test]
    fn image_group_insert_sibling_promotion_failure_compensates_whole_batch() {
        let directory = TestDirectory::new("image-group-promo-fail");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "33333333333333333333333333333333";
        let id2 = "44444444444444444444444444444444";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let fin1 = crate::managed_files::ManagedStore::final_rel_path(id1, "first.png");
        let fin2 = crate::managed_files::ManagedStore::final_rel_path(id2, "second.png");
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let path1 = store.stage_bytes(&st1, png_bytes).unwrap();
        let _path2 = store.stage_bytes(&st2, png_bytes).unwrap();

        // Pre-create destination directory of item 2 as a file to force fs::create_dir_all failure on second promote
        let full_fin2 = store.full_path(&fin2).unwrap();
        let conflict_parent = full_fin2.parent().unwrap();
        fs::write(conflict_parent, b"blocking file").unwrap();

        let candidates = vec![
            (st1.clone(), "first.png".to_string()),
            (st2.clone(), "second.png".to_string()),
        ];
        let res = state.accept_image_files(candidates, 0);
        assert!(res.is_err());

        // Sibling failure must compensate entire batch: no shelf items/batches, item1 final cleaned up
        let full_fin1 = store.full_path(&fin1).unwrap();
        assert!(!full_fin1.exists(), "item 1 promoted file must be compensated");
        assert!(!path1.exists(), "item 1 staging cleaned up");
        let snap = state.snapshot().unwrap();
        assert!(snap.batches.is_empty(), "no partial batch visible");
    }

    #[test]
    fn image_group_insert_ready_transaction_failure_cleans_up_and_rolls_back() {
        let directory = TestDirectory::new("image-group-ready-fail");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "77777777777777777777777777777777";
        let id2 = "88888888888888888888888888888888";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let fin1 = crate::managed_files::ManagedStore::final_rel_path(id1, "r1.png");
        let fin2 = crate::managed_files::ManagedStore::final_rel_path(id2, "r2.png");
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        store.stage_bytes(&st1, png_bytes).unwrap();
        store.stage_bytes(&st2, png_bytes).unwrap();

        // Block transition to 'ready'
        state.store().unwrap().connection.execute_batch(
            "CREATE TEMP TRIGGER block_ready BEFORE UPDATE OF state ON managed_files WHEN NEW.state = 'ready' BEGIN SELECT RAISE(ABORT, 'ready blocked'); END;"
        ).unwrap();

        let candidates = vec![
            (st1.clone(), "r1.png".to_string()),
            (st2.clone(), "r2.png".to_string()),
        ];
        let res = state.accept_image_files(candidates, 0);
        assert!(res.is_err());

        // No items or batches visible, promoted files deleted
        let full_fin1 = store.full_path(&fin1).unwrap();
        let full_fin2 = store.full_path(&fin2).unwrap();
        assert!(!full_fin1.exists(), "item 1 promoted file deleted");
        assert!(!full_fin2.exists(), "item 2 promoted file deleted");
        let snap = state.snapshot().unwrap();
        assert!(snap.batches.is_empty(), "no partial ready batch visible");
    }

    #[test]
    fn startup_reconciliation_mixed_staged_promoted_preserves_batch() {
        let directory = TestDirectory::new("reconcile-mixed-staged-promoted");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let id2 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let st1 = crate::managed_files::ManagedStore::staging_rel_path(id1);
        let st2 = crate::managed_files::ManagedStore::staging_rel_path(id2);
        let fin1 = crate::managed_files::ManagedStore::final_rel_path(id1, "mix1.png");
        let fin2 = crate::managed_files::ManagedStore::final_rel_path(id2, "mix2.png");
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];

        // Sibling 1 already promoted to final path
        let full_fin1 = store.full_path(&fin1).unwrap();
        if let Some(p) = full_fin1.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(&full_fin1, png_bytes).unwrap();

        // Sibling 2 still in staging
        store.stage_bytes(&st2, png_bytes).unwrap();

        // Seed both as 'staged' in DB
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_mix', 100, 0)", []).unwrap();
            let full_fin2 = store.full_path(&fin2).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_mix', 0, ?2, 'mix1.png', 0, 0)",
                [id1, full_fin1.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_mix', 1, ?2, 'mix2.png', 0, 0)",
                [id2, full_fin2.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id1, &fin1, &st1],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id2, &fin2, &st2],
            ).unwrap();
        }
        drop(state);

        // Startup reconciliation reconciles both into a complete ready batch
        let reopened = ShelfState::open(&database).unwrap();
        let snap = reopened.snapshot().unwrap();
        assert_eq!(snap.batches.len(), 1, "reconciled mixed batch is visible");
        assert_eq!(snap.batches[0].items.len(), 2);
        assert_eq!(snap.batches[0].items[0].id, id1);
        assert_eq!(snap.batches[0].items[1].id, id2);
        let full_fin2 = store.full_path(&fin2).unwrap();
        assert!(full_fin2.exists(), "sibling 2 promoted to final path");
    }

    #[test]
    fn unavailable_staging_file_io_error_fails_startup_safely_without_discard() {
        let directory = TestDirectory::new("reconcile-unavailable-io");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id = "cccccccccccccccccccccccccccccccc";
        let st = crate::managed_files::ManagedStore::staging_rel_path(id);
        let fin = crate::managed_files::ManagedStore::final_rel_path(id, "locked.png");
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let full_st = store.stage_bytes(&st, png_bytes).unwrap();

        // Seed staged item
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_lock', 100, 0)", []).unwrap();
            let full_fin = store.full_path(&fin).unwrap();
            conn.execute(
                "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
                 VALUES (?1, 'b_lock', 0, ?2, 'locked.png', 0, 0)",
                [id, full_fin.to_str().unwrap()],
            ).unwrap();
            conn.execute(
                "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
                 VALUES (?1, ?2, 'staged', ?3)",
                [id, &fin, &st],
            ).unwrap();
        }
        drop(state);

        // Lock staging file exclusively with Windows sharing mode 0 (no sharing)
        use std::os::windows::fs::OpenOptionsExt;
        let _lock_handle = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&full_st)
            .unwrap();

        // Reopen database: reading staging file should fail with I/O sharing violation, NOT treated as invalid content!
        // Must return Err rather than silently discarding/deleting the file.
        let reopen_res = ShelfState::open(&database);
        assert!(reopen_res.is_err(), "startup must fail on inaccessible I/O error");

        // File must NOT be deleted or discarded
        drop(_lock_handle);
        assert!(full_st.exists(), "staged file preserved across temporary I/O lock");
    }

    #[test]
    fn image_recovery_backup_preserves_bytes_and_ads() {
        let directory = TestDirectory::new("recovery-image-ads");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        let id = "dddddddddddddddddddddddddddddddd";
        let st = crate::managed_files::ManagedStore::staging_rel_path(id);
        let fin = crate::managed_files::ManagedStore::final_rel_path(id, "zone.png");
        let png_bytes = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
            0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let _path = store.stage_bytes(&st, png_bytes).unwrap();
        let candidates = vec![(st, "zone.png".to_string())];
        let drop_res = state.accept_image_files(candidates, 0).unwrap();
        let snap = drop_res.snapshot.unwrap();
        assert_eq!(snap.batches.len(), 1);

        let full_fin = store.full_path(&fin).unwrap();
        assert!(full_fin.exists());

        // Write Zone.Identifier ADS to the managed file
        let ads_path = format!("{}:Zone.Identifier", full_fin.display());
        fs::write(&ads_path, b"[ZoneTransfer]\r\nZoneId=3\r\n").unwrap();
        assert_eq!(fs::read(&ads_path).unwrap(), b"[ZoneTransfer]\r\nZoneId=3\r\n");

        // Corrupt the database to force Recovery Backup
        drop(state);
        fs::write(&database, b"corrupt db forcing recovery").unwrap();

        // Reopen via open_or_recover
        let (reopened, backup_dir_opt) = ShelfState::open_or_recover(&database).unwrap();
        let backup_dir = backup_dir_opt.expect("recovery backup created");

        // Active shelf is clean and empty
        assert!(reopened.snapshot().unwrap().batches.is_empty());
        assert!(!full_fin.exists(), "moved out of active managed store");

        // Check preserved file and ADS in backup directory
        let preserved_file = backup_dir.join("managed").join(id).join("zone.png");
        assert!(preserved_file.exists(), "preserved managed file in backup");
        assert_eq!(fs::read(&preserved_file).unwrap(), png_bytes);

        let preserved_ads = format!("{}:Zone.Identifier", preserved_file.display());
        assert_eq!(fs::read(&preserved_ads).unwrap(), b"[ZoneTransfer]\r\nZoneId=3\r\n", "Zone.Identifier ADS preserved in Recovery Backup");
    }

    #[test]
    fn pending_cleanup_queue_persists_across_restart_and_unowned_prefix_is_not_deleted() {
        let directory = TestDirectory::new("cleanup-queue-persistence");
        let database = directory.path().join("shelf.db");
        let state = ShelfState::open(&database).unwrap();
        let store = state.image_staging_store().unwrap();
        store.init_directories().unwrap();

        // Create a managed file and an external file in the same directory structure or profile
        let id = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
        let fin = crate::managed_files::ManagedStore::final_rel_path(id, "test.png");
        let full_fin = store.full_path(&fin).unwrap();
        if let Some(p) = full_fin.parent() {
            fs::create_dir_all(p).unwrap();
        }
        fs::write(&full_fin, b"test").unwrap();

        // Insert into pending_managed_file_deletions manually
        {
            let store_guard = state.store().unwrap();
            let conn = &store_guard.connection;
            conn.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&fin]).unwrap();
        }

        // Also create an external unowned file that shares the directory prefix or name
        let external_file = directory.path().join("managed").join("external.txt");
        fs::write(&external_file, b"unowned external file").unwrap();

        // Lock the managed file so deletion cannot succeed immediately
        use std::os::windows::fs::OpenOptionsExt;
        let lock_handle = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&full_fin)
            .unwrap();

        drop(state);

        // Reopen database: pending deletion was attempted and failed due to lock, so it must remain in pending queue!
        let reopened = ShelfState::open(&database).unwrap();
        let pending_count: i64 = reopened.store().unwrap().connection.query_row(
            "SELECT count(*) FROM pending_managed_file_deletions",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(pending_count, 1, "pending deletion must survive failed deletion on startup");
        assert!(external_file.exists(), "unowned file never deleted by prefix or location");

        // Release lock
        drop(lock_handle);
        drop(reopened);

        // Reopen database again: pending deletion should now succeed and row removed
        let reopened2 = ShelfState::open(&database).unwrap();
        let pending_count2: i64 = reopened2.store().unwrap().connection.query_row(
            "SELECT count(*) FROM pending_managed_file_deletions",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(pending_count2, 0, "pending deletion completed successfully once lock released");
        assert!(!full_fin.exists(), "managed file deleted");
        assert!(external_file.exists(), "unowned external file still untouched");
    }
}
