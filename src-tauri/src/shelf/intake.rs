//! Intake of dropped paths, text, and images into a new Shelf Batch.

use rusqlite::params;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use super::paths::{self, ordinal_equal};
use super::store::ShelfStore;
use super::{Availability, DropResult, ShelfError, ShelfSnapshot, ShelfState};
use super::store::{clear_sizing_when_empty, random_id};

struct IncomingItem {
    path: String,
    ordinal_path: Vec<u16>,
    availability: Availability,
}

impl ShelfState {
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
        if bytes.len() > crate::shelf::managed_files::MAX_TEXT_BYTES {
            return Ok(DropResult {
                snapshot: None,
                skipped: 1,
                message: None,
            });
        }
        let now = SystemTime::now();
        let timestamp_str = crate::shelf::managed_files::format_local_timestamp_now();
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

    pub(crate) fn image_staging_store(&self) -> Result<crate::shelf::managed_files::ManagedStore, ShelfError> {
        let store = self.store()?;
        Ok(crate::shelf::managed_files::ManagedStore::new(&store.profile_root))
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
}

impl ShelfStore {
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
            let (availability, is_folder) = paths::inspect(&item.path);
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
        let store = crate::shelf::managed_files::ManagedStore::new(&self.profile_root);
        store.init_directories().map_err(|source| ShelfError::fs(self.profile_root.display(), source))?;

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
        let staging_rel = crate::shelf::managed_files::ManagedStore::staging_rel_path(&item_id);
        let final_rel = crate::shelf::managed_files::ManagedStore::final_rel_path(&item_id, &filename);
        let final_path_str = store.full_path(&final_rel).map_err(|source| ShelfError::fs(&final_rel, source))?.display().to_string();

        // Step 1: Stage bytes to disk with flush
        let stage_full = store.stage_bytes(&staging_rel, bytes).map_err(|source| {
            ShelfError::fs(&staging_rel, source)
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
            return Err(ShelfError::fs(final_rel, promote_err));
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
        let store = crate::shelf::managed_files::ManagedStore::new(&self.profile_root);
        store.init_directories().map_err(|source| ShelfError::fs(self.profile_root.display(), source))?;

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
            let final_rel = crate::shelf::managed_files::ManagedStore::final_rel_path(&item_id, &filename);
            let final_path_str = store.full_path(&final_rel).map_err(|source| ShelfError::fs(&final_rel, source))?.display().to_string();
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
                first_promote_err = Some(ShelfError::fs(&item.final_rel, err));
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
}
