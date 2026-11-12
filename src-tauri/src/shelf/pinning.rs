//! Pinning of Shelf Items and Shelf Batches.

use super::*;
use super::store::{validate_unique_batches, validate_unique_items};

impl ShelfState {
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

    pub(crate) fn set_batches_pinned(
        &self,
        batch_ids: &[String],
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        validate_unique_batches(batch_ids)?;
        self.store()?.set_batches_pinned(batch_ids, pinned)
    }

    pub(crate) fn set_items_pinned(
        &self,
        batch_id: &str,
        item_ids: &[String],
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        validate_unique_items(item_ids)?;
        self.store()?.set_items_pinned(batch_id, item_ids, pinned)
    }
}

impl ShelfStore {
    pub(super) fn set_item_pinned(
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

    pub(super) fn set_batch_pinned(
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

    pub(super) fn set_batches_pinned(
        &mut self,
        batch_ids: &[String],
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        for batch_id in batch_ids {
            let exists: bool = transaction.query_row(
                "SELECT EXISTS (SELECT 1 FROM shelf_batches WHERE id = ?1)",
                [batch_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(ShelfError::BatchNotFound(batch_id.clone()));
            }
        }
        for batch_id in batch_ids {
            transaction.execute(
                "UPDATE shelf_items SET is_pinned = ?1
                 WHERE batch_id = ?2
                   AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = shelf_items.id AND mf.state = 'staged')",
                params![pinned, batch_id],
            )?;
        }
        let snapshot =
            Self::read_snapshot(&transaction, self.width, self.height, &self.availability)?;
        transaction.commit()?;
        Ok(snapshot)
    }

    pub(super) fn set_items_pinned(
        &mut self,
        batch_id: &str,
        item_ids: &[String],
        pinned: bool,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let transaction = self.connection.transaction()?;
        let batch_exists: bool = transaction.query_row(
            "SELECT EXISTS (SELECT 1 FROM shelf_batches WHERE id = ?1)",
            [batch_id],
            |row| row.get(0),
        )?;
        if !batch_exists {
            return Err(ShelfError::BatchNotFound(batch_id.to_owned()));
        }
        // Validate every identity against the declared batch before any write.
        for item_id in item_ids {
            let member: bool = transaction.query_row(
                "SELECT EXISTS (
                     SELECT 1 FROM shelf_items si
                     WHERE si.id = ?1 AND si.batch_id = ?2
                       AND NOT EXISTS (SELECT 1 FROM managed_files mf WHERE mf.item_id = si.id AND mf.state = 'staged')
                 )",
                params![item_id, batch_id],
                |row| row.get(0),
            )?;
            if !member {
                return Err(ShelfError::ItemNotFound(item_id.clone()));
            }
        }
        for item_id in item_ids {
            transaction.execute("UPDATE shelf_items SET is_pinned = ?1 WHERE id = ?2", params![pinned, item_id])?;
        }
        let snapshot =
            Self::read_snapshot(&transaction, self.width, self.height, &self.availability)?;
        transaction.commit()?;
        Ok(snapshot)
    }
}
