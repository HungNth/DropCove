//! Recovery Backup: database validation, SHM staging, and open-or-recover.

use rusqlite::OptionalExtension;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use super::store::ShelfStore;
use super::{ShelfError, ShelfState};

fn days_to_ymd(days: i64) -> (i64, u32, u32) {
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
    let mut source_file = fs::File::open(shm_path).map_err(|source| ShelfError::fs(shm_path.display(), source))?;
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
                return Err(ShelfError::fs(candidate.display(), source));
            }
        }
    };
    if let Err(source) = std::io::copy(&mut source_file, &mut stage_file)
        .and_then(|_| stage_file.sync_all())
    {
        drop(stage_file);
        let _ = fs::remove_file(&stage_path);
        return Err(ShelfError::fs(stage_path.display(), source));
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
    let mut file = fs::File::open(path).map_err(|source| ShelfError::fs(path.display(), source))?;
    let metadata = file.metadata().map_err(|source| ShelfError::fs(path.display(), source))?;
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
            return Err(ShelfError::fs(path.display(), err));
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

impl ShelfState {
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
                fs::create_dir_all(&recovery_area).map_err(|source| ShelfError::fs(recovery_area.display(), source))?;

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
                            return Err(ShelfError::fs(candidate.display(), source));
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
}
