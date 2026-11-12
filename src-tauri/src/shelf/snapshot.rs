//! Read side of the Shelf: snapshots, Manage Items batch, Shell Visual targets, and item actions.

use rusqlite::OptionalExtension;
use super::paths;
use rusqlite::Connection;
use std::fs;
use std::collections::{HashMap, HashSet};
use super::store::ShelfStore;
use super::{Availability, ItemActionKind, ItemOwnership, ManagedContentKind, ResolvedItemAction, ShelfBatch, ShelfError, ShelfItem, ShelfSnapshot, ShelfState};
use super::store::{automatic_height};

impl ShelfState {
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

    pub(crate) fn management_batch(&self, batch_id: &str) -> Result<ShelfBatch, ShelfError> {
        self.management_batch_with(batch_id, paths::classify)
    }

    pub(super) fn management_batch_with(
        &self,
        batch_id: &str,
        classifier: impl FnMut(&str) -> Availability,
    ) -> Result<ShelfBatch, ShelfError> {
        self.store()?.management_batch(batch_id, classifier)
    }

    pub(crate) fn item_visual_target(&self, item_id: &str) -> Result<Option<(String, bool)>, ShelfError> {
        self.store()?.item_visual_target(item_id)
    }

    pub(crate) fn read_managed_text_content(&self, item_id: &str) -> Result<Option<String>, ShelfError> {
        let store = self.store()?;
        let relative_path: Option<String> = store.connection.query_row(
            "SELECT mf.relative_path FROM managed_files mf WHERE mf.item_id = ?1 AND mf.state = 'ready'",
            [item_id],
            |row| row.get(0),
        ).optional()?;
        let Some(relative_path) = relative_path.filter(|path| path.ends_with(".txt")) else {
            return Ok(None);
        };
        let managed_store = crate::shelf::managed_files::ManagedStore::new(&store.profile_root);
        let full_path = managed_store.full_path(&relative_path).map_err(|source| ShelfError::fs("managed text", source))?;
        fs::read_to_string(&full_path).map(Some).map_err(|source| ShelfError::fs("managed text", source))
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

    pub(crate) fn resolve_item_action(
        &self,
        item_id: &str,
        action: ItemActionKind,
    ) -> Result<ResolvedItemAction, ShelfError> {
        let store = self.store()?;
        let item_info: Option<(String, bool, Option<String>, Option<String>)> = store
            .connection
            .query_row(
                "SELECT si.path, si.is_folder, mf.state, mf.relative_path
                 FROM shelf_items si
                 LEFT JOIN managed_files mf ON mf.item_id = si.id
                 WHERE si.id = ?1",
                [item_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;

        let (path, is_folder, mf_state, mf_relative) =
            item_info.ok_or_else(|| ShelfError::ItemNotFound(item_id.to_owned()))?;

        if mf_state.as_deref() == Some("staged") {
            return Err(ShelfError::InvalidItemAction(action, item_id.to_owned()));
        }

        let is_managed = mf_state.is_some();

        if is_managed {
            if mf_state.as_deref() != Some("ready") {
                return Err(ShelfError::InvalidItemAction(action, item_id.to_owned()));
            }
            let rel = mf_relative.unwrap_or_default();
            if !rel.ends_with(".txt") {
                return Err(ShelfError::InvalidItemAction(action, item_id.to_owned()));
            }
            if action != ItemActionKind::CopyText {
                return Err(ShelfError::InvalidItemAction(action, item_id.to_owned()));
            }
            let managed_store = crate::shelf::managed_files::ManagedStore::new(&store.profile_root);
            let full_path = managed_store
                .full_path(&rel)
                .map_err(|source| ShelfError::fs("managed text", source))?;
            drop(store);
            let text = fs::read_to_string(&full_path).map_err(|source| ShelfError::fs("managed text", source))?;
            Ok(ResolvedItemAction::CopyText(text))
        } else {
            match action {
                ItemActionKind::ShowInExplorer => {
                    Ok(ResolvedItemAction::ShowInExplorer { path, is_folder })
                }
                ItemActionKind::CopyPath => Ok(ResolvedItemAction::CopyPath(path)),
                ItemActionKind::CopyText => {
                    Err(ShelfError::InvalidItemAction(action, item_id.to_owned()))
                }
            }
        }
    }
}

impl ShelfStore {
    pub(super) fn snapshot(&mut self) -> Result<ShelfSnapshot, ShelfError> {
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
                item.availability = paths::classify(&item.path);
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

    pub(super) fn record_snapshot(&mut self, snapshot: &ShelfSnapshot) {
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

    pub(super) fn read_snapshot(
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
            "SELECT si.id, si.batch_id, si.position, si.path, si.name, si.is_folder, si.is_pinned,
                    mf.relative_path
             FROM shelf_items si
             LEFT JOIN managed_files mf ON mf.item_id = si.id AND mf.state = 'ready'
             WHERE si.batch_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM managed_files mf_staged
                   WHERE mf_staged.item_id = si.id AND mf_staged.state = 'staged'
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
                    let managed_rel: Option<String> = row.get(7)?;
                    let (ownership, managed_content_kind) = match managed_rel {
                        Some(rel) => {
                            let kind = if rel.ends_with(".txt") {
                                ManagedContentKind::Text
                            } else {
                                ManagedContentKind::Image
                            };
                            (ItemOwnership::Managed, Some(kind))
                        }
                        None => (ItemOwnership::External, None),
                    };
                    Ok(ShelfItem {
                        id,
                        batch_id: row.get(1)?,
                        position: row.get(2)?,
                        path: row.get(3)?,
                        name: row.get(4)?,
                        is_folder: row.get(5)?,
                        is_pinned: row.get(6)?,
                        availability,
                        ownership,
                        managed_content_kind,
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

    pub(super) fn management_batch(
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
            "SELECT si.id, si.batch_id, si.position, si.path, si.name, si.is_folder, si.is_pinned,
                    mf.relative_path
             FROM shelf_items si
             LEFT JOIN managed_files mf ON mf.item_id = si.id AND mf.state = 'ready'
             WHERE si.batch_id = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM managed_files mf_staged
                   WHERE mf_staged.item_id = si.id AND mf_staged.state = 'staged'
               )
             ORDER BY si.position ASC",
        )?;
        batch.items = items_statement
            .query_map([batch_id], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(3)?;
                let availability = classifier(&path);
                let managed_rel: Option<String> = row.get(7)?;
                let (ownership, managed_content_kind) = match managed_rel {
                    Some(rel) => {
                        let kind = if rel.ends_with(".txt") {
                            ManagedContentKind::Text
                        } else {
                            ManagedContentKind::Image
                        };
                        (ItemOwnership::Managed, Some(kind))
                    }
                    None => (ItemOwnership::External, None),
                };
                Ok(ShelfItem {
                    id,
                    batch_id: row.get(1)?,
                    position: row.get(2)?,
                    path,
                    name: row.get(4)?,
                    is_folder: row.get(5)?,
                    is_pinned: row.get(6)?,
                    availability,
                    ownership,
                    managed_content_kind,
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

    pub(super) fn item_visual_target(&self, item_id: &str) -> Result<Option<(String, bool)>, ShelfError> {
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
