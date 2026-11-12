//! Removal of Shelf Items and Shelf Batches, Clear Temporary, and removal previews.

use super::*;
use super::store::{clear_sizing_when_empty, push_token_field, validate_unique_batches, validate_unique_items};

impl ShelfState {
    pub(crate) fn remove_batch(&self, batch_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.remove_batch(batch_id)
    }

    pub(crate) fn remove_item(&self, item_id: &str) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.remove_item(item_id)
    }

    pub(crate) fn clear_temporary(&self) -> Result<ShelfSnapshot, ShelfError> {
        self.store()?.clear_temporary()
    }

    pub(crate) fn prepare_batch_removal(&self, batch_ids: &[String]) -> Result<RemovalPreview, ShelfError> {
        validate_unique_batches(batch_ids)?;
        self.store()?.prepare_batch_removal(batch_ids)
    }

    /// Confirms a previewed removal from only identities plus the opaque token. Counts are recomputed
    /// from persisted state; a forged or outdated token is stale. Only a vanished target is translated
    /// to stale, so database and validation failures keep their real error.
    pub(crate) fn confirm_batch_removal(
        &self,
        batch_ids: &[String],
        token: &str,
    ) -> Result<ShelfSnapshot, ShelfError> {
        validate_unique_batches(batch_ids)?;
        let current = match self.store()?.prepare_batch_removal(batch_ids) {
            Ok(current) => current,
            Err(ShelfError::BatchNotFound(_)) => return Err(ShelfError::StaleRemovalTarget),
            Err(other) => return Err(other),
        };
        let preview = RemovalPreview { token: token.to_owned(), ..current };
        self.store()?.remove_batches(batch_ids, &preview)
    }

    pub(crate) fn prepare_item_removal(
        &self,
        batch_id: &str,
        item_ids: &[String],
    ) -> Result<RemovalPreview, ShelfError> {
        validate_unique_items(item_ids)?;
        self.store()?.prepare_item_removal(batch_id, item_ids)
    }

    pub(crate) fn confirm_item_removal(
        &self,
        batch_id: &str,
        item_ids: &[String],
        token: &str,
    ) -> Result<ShelfSnapshot, ShelfError> {
        validate_unique_items(item_ids)?;
        let current = match self.store()?.prepare_item_removal(batch_id, item_ids) {
            Ok(current) => current,
            Err(ShelfError::BatchNotFound(_) | ShelfError::ItemNotFound(_)) => return Err(ShelfError::StaleRemovalTarget),
            Err(other) => return Err(other),
        };
        let preview = RemovalPreview { token: token.to_owned(), ..current };
        self.store()?.remove_items(batch_id, item_ids, &preview)
    }
}

impl ShelfStore {
    pub(super) fn remove_batch(&mut self, batch_id: &str) -> Result<ShelfSnapshot, ShelfError> {
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

    pub(super) fn remove_item(&mut self, item_id: &str) -> Result<ShelfSnapshot, ShelfError> {
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

    pub(super) fn clear_temporary(&mut self) -> Result<ShelfSnapshot, ShelfError> {
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

    /// Counts and membership token for a batch removal, read from persisted rows in one pass.
    pub(super) fn removal_scope(
        connection: &Connection,
        batch_ids: &[String],
    ) -> Result<RemovalPreview, ShelfError> {
        let mut items = 0usize;
        let mut managed_files = 0usize;
        let mut token = String::new();
        for batch_id in batch_ids {
            let exists: bool = connection.query_row(
                "SELECT EXISTS (SELECT 1 FROM shelf_batches WHERE id = ?1)",
                [batch_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(ShelfError::BatchNotFound(batch_id.clone()));
            }
            push_token_field(&mut token, batch_id);
            let mut stmt = connection.prepare(
                "SELECT si.id, EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id)
                 FROM shelf_items si
                 WHERE si.batch_id = ?1
                   AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')
                 ORDER BY si.position, si.id",
            )?;
            let mut rows = stmt.query([batch_id])?;
            while let Some(row) = rows.next()? {
                let item_id: String = row.get(0)?;
                let managed: bool = row.get(1)?;
                items += 1;
                managed_files += usize::from(managed);
                push_token_field(&mut token, &item_id);
                token.push(if managed { 'm' } else { 'e' });
            }
            token.push(';');
        }
        Ok(RemovalPreview { batches: batch_ids.len(), items, managed_files, token })
    }

    pub(super) fn prepare_batch_removal(&mut self, batch_ids: &[String]) -> Result<RemovalPreview, ShelfError> {
        Self::removal_scope(&self.connection, batch_ids)
    }

    pub(super) fn item_removal_scope(
        connection: &Connection,
        batch_id: &str,
        item_ids: &[String],
    ) -> Result<RemovalPreview, ShelfError> {
        let batch_exists: bool = connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM shelf_batches WHERE id = ?1)",
            [batch_id],
            |row| row.get(0),
        )?;
        if !batch_exists {
            return Err(ShelfError::BatchNotFound(batch_id.to_owned()));
        }
        let mut managed_files = 0usize;
        let mut token = String::new();
        push_token_field(&mut token, batch_id);
        for item_id in item_ids {
            let managed: Option<bool> = connection
                .query_row(
                    "SELECT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id)
                     FROM shelf_items si
                     WHERE si.id = ?1 AND si.batch_id = ?2
                       AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')",
                    params![item_id, batch_id],
                    |row| row.get(0),
                )
                .optional()?;
            let managed = managed.ok_or_else(|| ShelfError::ItemNotFound(item_id.clone()))?;
            managed_files += usize::from(managed);
            push_token_field(&mut token, item_id);
            token.push(if managed { 'm' } else { 'e' });
        }
        Ok(RemovalPreview { batches: 1, items: item_ids.len(), managed_files, token })
    }

    pub(super) fn prepare_item_removal(&mut self, batch_id: &str, item_ids: &[String]) -> Result<RemovalPreview, ShelfError> {
        Self::item_removal_scope(&self.connection, batch_id, item_ids)
    }

    pub(super) fn remove_items(
        &mut self,
        batch_id: &str,
        item_ids: &[String],
        preview: &RemovalPreview,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        match Self::item_removal_scope(&transaction, batch_id, item_ids) {
            Ok(current) if current == *preview => {}
            Ok(_) | Err(ShelfError::BatchNotFound(_) | ShelfError::ItemNotFound(_)) => return Err(ShelfError::StaleRemovalTarget),
            Err(other) => return Err(other),
        }
        Self::queue_managed_deletions_for_items(&transaction, item_ids.iter().map(String::as_str))?;
        for item_id in item_ids {
            transaction.execute("DELETE FROM shelf_items WHERE id = ?1", [item_id])?;
        }
        transaction.execute(
            "DELETE FROM shelf_batches WHERE NOT EXISTS (SELECT 1 FROM shelf_items WHERE batch_id = shelf_batches.id)",
            [],
        )?;
        clear_sizing_when_empty(&transaction)?;
        let empty = !transaction.query_row("SELECT EXISTS (SELECT 1 FROM shelf_items)", [], |row| row.get::<_, bool>(0))?;
        let snapshot = Self::read_snapshot(&transaction, if empty { 191.0 } else { self.width }, if empty { 180.0 } else { self.height }, &self.availability)?;
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
        self.snapshot()
    }

    pub(super) fn remove_batches(
        &mut self,
        batch_ids: &[String],
        preview: &RemovalPreview,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        // Revalidate inside the transaction so a competing mutation cannot slip between check and delete.
        // A target that disappeared since the preview is stale, not an unknown identity.
        match Self::removal_scope(&transaction, batch_ids) {
            Ok(current) if current == *preview => {}
            Ok(_) | Err(ShelfError::BatchNotFound(_)) => return Err(ShelfError::StaleRemovalTarget),
            Err(other) => return Err(other),
        }
        for batch_id in batch_ids {
            Self::queue_managed_deletions_for_batch(&transaction, batch_id)?;
            transaction.execute("DELETE FROM shelf_batches WHERE id = ?1", [batch_id])?;
        }
        clear_sizing_when_empty(&transaction)?;
        let empty = !transaction.query_row(
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
        self.snapshot()
    }
}
