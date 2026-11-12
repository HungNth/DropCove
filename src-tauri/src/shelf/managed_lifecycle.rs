//! Pending Managed File Deletion, cleanup, and startup reconciliation of Managed Files.

use super::*;
use super::store::{clear_sizing_when_empty};

impl ShelfStore {
    pub(super) fn queue_managed_deletions_for_items<'a>(
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

    pub(super) fn queue_managed_deletions_for_batch(
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

    pub(super) fn queue_managed_deletions_for_unpinned(
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

    pub(super) fn process_pending_managed_deletions(&mut self) -> Result<(), ShelfError> {
        let store = crate::shelf::managed_files::ManagedStore::new(&self.profile_root);
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

    pub(super) fn validate_managed_file(path: &Path) -> Result<bool, ShelfError> {
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(ShelfError::fs(path.display(), e)),
        };
        if !meta.is_file() {
            return Ok(false);
        }
        let is_txt = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("txt")).unwrap_or(false);
        if is_txt {
            if meta.len() <= crate::shelf::managed_files::MAX_TEXT_BYTES as u64 {
                let bytes = match fs::read(path) {
                    Ok(b) => b,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
                    Err(e) => return Err(ShelfError::fs(path.display(), e)),
                };
                return Ok(std::str::from_utf8(&bytes).is_ok());
            }
            return Ok(false);
        } else {
            match crate::shelf::website_images::validate_image(path) {
                Ok(_) => Ok(true),
                Err(crate::shelf::website_images::ValidationError::Invalid(_)) => Ok(false),
                Err(crate::shelf::website_images::ValidationError::Io(e)) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(false)
                    } else {
                        Err(ShelfError::fs(path.display(), e))
                    }
                }
            }
        }
    }

    pub(super) fn reconcile_managed_files(&mut self) -> Result<Option<String>, ShelfError> {
        let store = crate::shelf::managed_files::ManagedStore::new(&self.profile_root);
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
                        let final_path = store.full_path(&item.final_rel).map_err(|e| ShelfError::fs(&item.final_rel, e))?;
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
}
