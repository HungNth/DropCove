//! Batch and Item drag-out: preparation, start, completion, and occurrence consumption.

use rusqlite::OptionalExtension;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use super::paths::{self, ordinal_equal};
use super::store::ShelfStore;
use super::{Availability, BatchPreparation, DragDomain, DragOutcome, DragParticipant, ItemPreparation, PreparedDrag, PreparedTransfer, ShelfError, ShelfItem, ShelfSnapshot, ShelfState};
use super::store::{clear_sizing_when_empty, random_id};

struct PreparedCandidateItems<'a> {
    participants: Vec<DragParticipant>,
    paths: Vec<String>,
    missing: Vec<&'a str>,
    unavailable: usize,
    classifications: Vec<(&'a str, Availability)>,
}

fn classify_and_dedupe_candidates<'a>(
    items: impl IntoIterator<Item = &'a ShelfItem>,
    mut classifier: impl FnMut(&str) -> Availability,
) -> PreparedCandidateItems<'a> {
    let mut participants = Vec::new();
    let mut paths = Vec::new();
    let mut ordinal_paths: HashMap<usize, Vec<Vec<u16>>> = HashMap::new();
    let mut missing = Vec::new();
    let mut unavailable = 0;
    let mut classifications = Vec::new();
    for item in items {
        let availability = classifier(&item.path);
        classifications.push((item.id.as_str(), availability));
        match availability {
            Availability::Available => {
                participants.push(DragParticipant {
                    id: item.id.clone(),
                });
                let ordinal_path = item.path.encode_utf16().collect::<Vec<_>>();
                let same_length = ordinal_paths.entry(ordinal_path.len()).or_default();
                if !same_length
                    .iter()
                    .any(|existing| ordinal_equal(existing, &ordinal_path))
                {
                    same_length.push(ordinal_path);
                    paths.push(item.path.clone());
                }
            }
            Availability::Missing => missing.push(item.id.as_str()),
            Availability::Unavailable => unavailable += 1,
        }
    }
    PreparedCandidateItems {
        participants,
        paths,
        missing,
        unavailable,
        classifications,
    }
}

impl ShelfState {
    pub(crate) fn prepare_batch_drag(
        &self,
        batch_ids: &[String],
    ) -> Result<BatchPreparation, ShelfError> {
        self.prepare_batch_drag_with(batch_ids, paths::classify)
    }

    pub(super) fn prepare_batch_drag_with(
        &self,
        batch_ids: &[String],
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<BatchPreparation, ShelfError> {
        self.store()?.prepare_batches(batch_ids, classifier)
    }

    pub(crate) fn begin_batch_drag(
        &self,
        token: &str,
    ) -> Result<Arc<PreparedTransfer>, ShelfError> {
        let mut store = self.store()?;
        let prepared = store
            .prepared
            .as_ref()
            .filter(|prepared| prepared.token == token && prepared.domain == DragDomain::Batch)
            .ok_or(ShelfError::InvalidPreparedDrag)?;
        if prepared.changes != store.connection.total_changes() {
            store.prepared = None;
            return Err(ShelfError::InvalidPreparedDrag);
        }
        Ok(store.prepared.take().ok_or(ShelfError::InvalidPreparedDrag)?.transfer)
    }

    pub(crate) fn cancel_batch_drag(&self, token: &str) -> Result<ShelfSnapshot, ShelfError> {
        let mut store = self.store()?;
        store
            .prepared
            .as_ref()
            .filter(|prepared| prepared.token == token && prepared.domain == DragDomain::Batch)
            .ok_or(ShelfError::InvalidPreparedDrag)?;
        store.prepared = None;
        store.snapshot()
    }

    pub(crate) fn prepare_item_drag(
        &self,
        batch_id: &str,
        item_ids: &[String],
    ) -> Result<ItemPreparation, ShelfError> {
        self.prepare_item_drag_with(batch_id, item_ids, paths::classify)
    }

    pub(crate) fn prepare_item_drag_with(
        &self,
        batch_id: &str,
        item_ids: &[String],
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<ItemPreparation, ShelfError> {
        self.store()?.prepare_items(batch_id, item_ids, classifier)
    }

    pub(crate) fn begin_item_drag(
        &self,
        token: &str,
        expected_batch_id: Option<&str>,
    ) -> Result<Arc<PreparedTransfer>, ShelfError> {
        let mut store = self.store()?;
        let prepared = store
            .prepared
            .as_ref()
            .filter(|prepared| {
                prepared.token == token
                    && prepared.domain == DragDomain::Item
                    && (expected_batch_id.is_none()
                        || prepared.batch_id.as_deref() == expected_batch_id)
            })
            .ok_or(ShelfError::InvalidPreparedDrag)?;
        if prepared.changes != store.connection.total_changes() {
            store.prepared = None;
            return Err(ShelfError::InvalidPreparedDrag);
        }
        Ok(store.prepared.take().ok_or(ShelfError::InvalidPreparedDrag)?.transfer)
    }

    pub(crate) fn cancel_item_drag(&self, token: &str) -> Result<ShelfSnapshot, ShelfError> {
        let mut store = self.store()?;
        store
            .prepared
            .as_ref()
            .filter(|prepared| prepared.token == token && prepared.domain == DragDomain::Item)
            .ok_or(ShelfError::InvalidPreparedDrag)?;
        store.prepared = None;
        store.snapshot()
    }

    pub(crate) fn cancel_prepared_item_drag(&self) -> Result<(), ShelfError> {
        let mut store = self.store()?;
        if store.prepared.as_ref().is_some_and(|prepared| prepared.domain == DragDomain::Item) {
            store.prepared = None;
        }
        Ok(())
    }

    pub(crate) fn complete_item_drag(
        &self,
        transfer: &PreparedTransfer,
        outcome: DragOutcome,
    ) -> Result<ShelfSnapshot, ShelfError> {
        self.complete_batch_drag(transfer, outcome)
    }

    pub(crate) fn complete_batch_drag(
        &self,
        transfer: &PreparedTransfer,
        outcome: DragOutcome,
    ) -> Result<ShelfSnapshot, ShelfError> {
        let mut store = self.store()?;
        if outcome == DragOutcome::Copy {
            store.consume_items(transfer.participants.iter().map(|item| item.id.as_str()))
        } else {
            store.snapshot()
        }
    }
}

impl ShelfStore {
    fn commit_drag_preparation(
        &mut self,
        domain: DragDomain,
        batch_id: Option<String>,
        candidates: PreparedCandidateItems<'_>,
    ) -> Result<(Option<String>, usize, usize, ShelfSnapshot), ShelfError> {
        let transaction = self.connection.transaction()?;
        Self::queue_managed_deletions_for_items(&transaction, candidates.missing.iter().copied())?;
        for item_id in &candidates.missing {
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
        let available = candidates.participants.len();
        let token = if available > 0 {
            let id = random_id(&transaction)?;
            match domain {
                DragDomain::Batch => Some(format!("batch_{id}")),
                DragDomain::Item => Some(format!("item_{id}")),
            }
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
        for (id, availability) in candidates.classifications {
            if let Some(value) = self.availability.get_mut(id) {
                *value = availability;
            }
        }
        self.process_pending_managed_deletions()?;
        self.prepared = token.as_ref().map(|token| PreparedDrag {
            token: token.clone(),
            domain,
            batch_id,
            changes: self.connection.total_changes(),
            transfer: Arc::new(PreparedTransfer {
                participants: candidates.participants,
                paths: candidates.paths,
            }),
        });
        let snapshot = self.snapshot()?;
        Ok((token, available, candidates.unavailable, snapshot))
    }

    fn prepare_batches(
        &mut self,
        batch_ids: &[String],
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<BatchPreparation, ShelfError> {
        if batch_ids.is_empty() {
            return Err(ShelfError::EmptyBatchSelection);
        }
        let mut selected = HashSet::with_capacity(batch_ids.len());
        for id in batch_ids {
            if !selected.insert(id.as_str()) {
                return Err(ShelfError::DuplicateBatchSelection(id.clone()));
            }
        }
        let current = self.snapshot()?;
        for id in batch_ids {
            if !current.batches.iter().any(|batch| &batch.id == id) {
                return Err(ShelfError::BatchNotFound(id.clone()));
            }
        }
        let candidate_items = current
            .batches
            .iter()
            .filter(|batch| selected.contains(batch.id.as_str()))
            .flat_map(|batch| &batch.items);
        let candidates = classify_and_dedupe_candidates(candidate_items, classifier);
        let (token, available, unavailable, snapshot) =
            self.commit_drag_preparation(DragDomain::Batch, None, candidates)?;
        Ok(BatchPreparation {
            token,
            available,
            unavailable,
            snapshot,
        })
    }

    fn prepare_items(
        &mut self,
        batch_id: &str,
        item_ids: &[String],
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<ItemPreparation, ShelfError> {
        if item_ids.is_empty() {
            return Err(ShelfError::EmptyItemSelection);
        }
        let mut selected = HashSet::with_capacity(item_ids.len());
        for id in item_ids {
            if !selected.insert(id.as_str()) {
                return Err(ShelfError::DuplicateItemSelection(id.clone()));
            }
        }
        let current = self.snapshot()?;
        let batch = current
            .batches
            .iter()
            .find(|batch| batch.id == batch_id)
            .ok_or_else(|| ShelfError::BatchNotFound(batch_id.to_string()))?;
        for id in item_ids {
            if !batch.items.iter().any(|item| &item.id == id) {
                return Err(ShelfError::ItemNotFound(id.clone()));
            }
        }
        let candidate_items = batch
            .items
            .iter()
            .filter(|item| selected.contains(item.id.as_str()));
        let candidates = classify_and_dedupe_candidates(candidate_items, classifier);
        let (token, available, unavailable, snapshot) =
            self.commit_drag_preparation(DragDomain::Item, Some(batch_id.to_string()), candidates)?;
        Ok(ItemPreparation {
            token,
            available,
            unavailable,
            snapshot,
        })
    }

    pub(super) fn consume_items<'a>(
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
}
