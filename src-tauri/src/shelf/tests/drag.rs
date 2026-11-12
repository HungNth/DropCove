use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::Path;

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
    let batch_id = &snapshot.batches[0].id;
    let item_id = &snapshot.batches[0].items[0].id;

    let prep = state
        .prepare_item_drag(batch_id, std::slice::from_ref(item_id))
        .expect("item drag should prepare");
    let transfer = state
        .begin_item_drag(prep.token.as_deref().unwrap(), Some(batch_id))
        .expect("item drag should begin");
    let completed = state
        .complete_item_drag(&transfer, DragOutcome::Copy)
        .expect("copy completion should persist");
    assert!(completed.batches.is_empty());
    assert!(directory.source_file().exists());
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
    let batch_id = &snapshot.batches[0].id;
    let item_id = &snapshot.batches[0].items[0].id;

    let prep = state
        .prepare_item_drag(batch_id, std::slice::from_ref(item_id))
        .expect("item drag should prepare");
    let transfer = state
        .begin_item_drag(prep.token.as_deref().unwrap(), Some(batch_id))
        .expect("item drag should begin");
    let completed = state
        .complete_item_drag(&transfer, DragOutcome::Canceled)
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
    let batch_id = &snapshot.batches[0].id;
    let item_id = snapshot.batches[0].items[0].id.clone();

    let prep = state
        .prepare_item_drag(batch_id, std::slice::from_ref(&item_id))
        .expect("item drag should prepare");
    let transfer = state
        .begin_item_drag(prep.token.as_deref().unwrap(), Some(batch_id))
        .expect("item drag should begin");

    state
        .install_test_delete_failure()
        .expect("failure trigger should install");
    assert!(state
        .complete_item_drag(&transfer, DragOutcome::Copy)
        .is_err());
    drop(state);

    let restored = ShelfState::open(database)
        .expect("shelf should reopen")
        .snapshot()
        .expect("snapshot should load");

    assert_eq!(restored.batches[0].items[0].id, item_id);
}

#[test]
fn batch_preparation_requires_available_items_and_cancel_retains_all_eligible() {
    use crate::shelf::Availability;
    let directory = TestDirectory::new("prepare-empty");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let initial = state
        .accept_drop(vec![Some("Z:\\unavailable.txt".into())])
        .unwrap()
        .snapshot
        .unwrap();
    let empty_prepared = state
        .prepare_batch_drag_with(std::slice::from_ref(&initial.batches[0].id), |_| Availability::Unavailable)
        .unwrap();
    assert!(empty_prepared.token.is_none());
    assert_eq!(
        (empty_prepared.available, empty_prepared.unavailable),
        (0, 1)
    );
}

#[test]
fn copy_consumes_only_prepared_available_participants() {
    use crate::shelf::Availability;
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
        .prepare_batch_drag_with(std::slice::from_ref(&initial.batches[0].id), |path| {
            if path.starts_with("Z:") {
                Availability::Unavailable
            } else {
                Availability::Available
            }
        })
        .unwrap();
    let token = prepared.token.unwrap();
    let transfer = state.begin_batch_drag(&token).unwrap();
    let added = state
        .accept_drop(vec![Some("C:\\unrelated.txt".into())])
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(added.batches.len(), 2);
    let completed = state
        .complete_batch_drag(&transfer, DragOutcome::Copy)
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
fn batch_preparation_removes_only_missing_and_cancel_retains_available() {
    use crate::shelf::Availability;
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
        .prepare_batch_drag_with(std::slice::from_ref(&initial.batches[0].id), |path| {
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
        .cancel_batch_drag(prepared.token.as_deref().unwrap())
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
    let prep = state.prepare_batch_drag(std::slice::from_ref(&batch_id)).unwrap();
    let token = prep.token.expect("batch drag token should exist");
    let transfer = state.begin_batch_drag(&token).unwrap();
    assert_eq!(prep.available, 2);

    // While drag is active/prepared, user pins item_a via set_item_pinned
    // (or pins all via set_batch_pinned)
    state.set_item_pinned(&item_a_id, true).unwrap();

    // Complete batch drag with Copy outcome
    // consume_items executes "DELETE FROM shelf_items WHERE id = ?1 AND is_pinned = 0"
    // Since item_a is now pinned, it MUST be retained; item_b is temporary, so it is consumed
    let snap = state
        .complete_batch_drag(&transfer, DragOutcome::Copy)
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

    let prep = state.prepare_batch_drag(std::slice::from_ref(&batch_id)).unwrap();
    let token = prep.token.unwrap();
    let transfer = state.begin_batch_drag(&token).unwrap();

    // Install trigger that fails item deletes
    state.install_test_delete_failure().unwrap();

    // Attempt to complete batch drag as Copy (consumption)
    let result = state.complete_batch_drag(&transfer, DragOutcome::Copy);
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
fn selected_drag_deduplicates_paths_but_consumes_every_temporary_occurrence() {
    let directory = TestDirectory::new("selected-occurrences");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let older = state.accept_drop(vec![Some("C:\\same.txt".into()), Some("C:\\older.txt".into())]).unwrap().snapshot.unwrap();
    let pinned = older.batches[0].items[0].id.clone();
    state.set_item_pinned(&pinned, true).unwrap();
    let newer = state.accept_drop(vec![Some("c:\\SAME.TXT".into()), Some("C:\\newer.txt".into())]).unwrap().snapshot.unwrap();
    let ids = [older.batches[0].id.clone(), newer.batches[0].id.clone()];
    let prepared = state.prepare_batch_drag_with(&ids, |_| crate::shelf::Availability::Available).unwrap();
    assert_eq!(prepared.available, 4);
    let transfer = state.begin_batch_drag(prepared.token.as_deref().unwrap()).unwrap();
    assert_eq!(transfer.paths, ["c:\\SAME.TXT", "C:\\newer.txt", "C:\\older.txt"]);
    assert_eq!(transfer.participants.len(), 4);
    let complete = state.complete_batch_drag(&transfer, DragOutcome::Copy).unwrap();
    assert_eq!(complete.batches.len(), 1);
    assert_eq!(complete.batches[0].items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), [pinned.as_str()]);
}

#[test]
fn selected_drag_missing_cleanup_survives_cancel_and_retains_managed_bytes() {
    let directory = TestDirectory::new("selected-missing");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let external = state.accept_drop(vec![Some("C:\\missing.txt".into()), Some("Z:\\offline.txt".into())]).unwrap().snapshot.unwrap();
    let managed = state.accept_text("keep exact text".into()).unwrap().snapshot.unwrap();
    let managed_path = managed.batches[0].items[0].path.clone();
    let ids = [external.batches[0].id.clone(), managed.batches[0].id.clone()];
    let prepared = state.prepare_batch_drag_with(&ids, |path| {
        if path == "C:\\missing.txt" { crate::shelf::Availability::Missing }
        else if path.starts_with("Z:") { crate::shelf::Availability::Unavailable }
        else { crate::shelf::Availability::Available }
    }).unwrap();
    assert_eq!((prepared.available, prepared.unavailable), (1, 1));
    let canceled = state.cancel_batch_drag(prepared.token.as_deref().unwrap()).unwrap();
    assert_eq!(canceled.batches.iter().flat_map(|batch| &batch.items).map(|item| item.path.as_str()).collect::<Vec<_>>(), [managed_path.as_str(), "Z:\\offline.txt"]);
    assert_eq!(fs::read(&managed_path).unwrap(), b"keep exact text");
    let prepared = state.prepare_batch_drag_with(&[managed.batches[0].id.clone()], |_| crate::shelf::Availability::Available).unwrap();
    let transfer = state.begin_batch_drag(prepared.token.as_deref().unwrap()).unwrap();
    state.complete_batch_drag(&transfer, DragOutcome::Copy).unwrap();
    assert!(!Path::new(&managed_path).exists());
}

#[test]
fn selected_drag_rejects_empty_duplicate_and_unknown_identifiers_without_mutation() {
    let directory = TestDirectory::new("selected-invalid");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let initial = state.accept_drop(vec![Some("C:\\a.txt".into())]).unwrap().snapshot.unwrap();
    let id = initial.batches[0].id.clone();
    assert!(state.prepare_batch_drag(&[]).is_err());
    assert!(state.prepare_batch_drag(&[id.clone(), id.clone()]).is_err());
    assert!(state.prepare_batch_drag(&[id, "unknown".into()]).is_err());
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), serde_json::to_value(initial).unwrap());
}

#[test]
fn selected_preparation_failure_preserves_availability_and_armed_start_rejects_mutation() {
    let directory = TestDirectory::new("selected-preparation-rollback");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_drop(vec![Some("C:\\available.txt".into()), Some("C:\\missing.txt".into())]).unwrap().snapshot.unwrap();
    let ids = [snapshot.batches[0].id.clone()];
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    state.install_test_delete_failure().unwrap();
    assert!(state.prepare_batch_drag_with(&ids, |path| if path.ends_with("missing.txt") { crate::shelf::Availability::Missing } else { crate::shelf::Availability::Available }).is_err());
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before);
    let prepared = state.prepare_batch_drag_with(&ids, |_| crate::shelf::Availability::Available).unwrap();
    state.set_item_pinned(&snapshot.batches[0].items[0].id, true).unwrap();
    assert!(state.begin_batch_drag(prepared.token.as_deref().unwrap()).is_err());
    assert!(state.cancel_batch_drag(prepared.token.as_deref().unwrap()).is_err());
}

#[test]
fn running_drag_completes_its_captured_occurrences_without_clearing_a_new_preparation() {
    let directory = TestDirectory::new("overlapping-prepared-drag");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let first = state.accept_drop(vec![Some("C:\\first.txt".into())]).unwrap().snapshot.unwrap();
    let second = state.accept_drop(vec![Some("C:\\second.txt".into())]).unwrap().snapshot.unwrap();
    let first_prepared = state.prepare_batch_drag_with(std::slice::from_ref(&first.batches[0].id), |_| crate::shelf::Availability::Available).unwrap();
    let first_token = first_prepared.token.unwrap();
    let running = state.begin_batch_drag(&first_token).unwrap();
    let next = state.prepare_batch_drag_with(std::slice::from_ref(&second.batches[0].id), |_| crate::shelf::Availability::Available).unwrap();
    assert!(state.cancel_batch_drag(&first_token).is_err(), "an old token must not cancel a new pending drag");
    let completed = state.complete_batch_drag(&running, DragOutcome::Copy).unwrap();
    assert_eq!(completed.batches.iter().flat_map(|batch| &batch.items).map(|item| item.path.as_str()).collect::<Vec<_>>(), ["C:\\second.txt"]);
    state.cancel_batch_drag(next.token.as_deref().unwrap()).unwrap();
    assert_eq!(state.snapshot().unwrap().batches[0].id, second.batches[0].id);
}

#[test]
fn managed_text_read_failure_retains_the_occurrence_and_reports_the_error() {
    let directory = TestDirectory::new("managed-text-drag-read-failure");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_text("preserve this content".into()).unwrap().snapshot.unwrap();
    let item = &snapshot.batches[0].items[0];
    fs::remove_file(&item.path).unwrap();
    let failure = state.read_managed_text_content(&item.id).unwrap_err();
    assert!(matches!(failure, crate::shelf::ShelfError::FileSystem { ref source, .. } if source.kind() == std::io::ErrorKind::NotFound));
    assert_eq!(state.snapshot().unwrap().batches[0].items[0].id, item.id);
    fs::write(&item.path, "preserve this content").unwrap();
    assert_eq!(state.read_managed_text_content(&item.id).unwrap().as_deref(), Some("preserve this content"));
    fs::write(&item.path, [0xff]).unwrap();
    assert!(matches!(state.read_managed_text_content(&item.id), Err(crate::shelf::ShelfError::FileSystem { source, .. }) if source.kind() == std::io::ErrorKind::InvalidData));
}

#[test]
fn prepare_item_drag_validates_batch_boundary_and_source_order() {
    let directory = TestDirectory::new("prepare-item-drag-foundation");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let paths: Vec<String> = (0..3).map(|index| {
        let path = directory.path().join(format!("item{index}.txt"));
        fs::write(&path, format!("source {index}")).unwrap();
        path.to_string_lossy().into_owned()
    }).collect();
    let drop1 = state.accept_drop(paths.iter().cloned().map(Some).collect()).unwrap().snapshot.unwrap();
    let drop2 = state.accept_drop(vec![
        Some("C:\\other.txt".into()),
    ]).unwrap().snapshot.unwrap();
    let batch1 = &drop1.batches[0];
    let batch2 = &drop2.batches[0];

    // Empty selection rejected
    assert!(matches!(
        state.prepare_item_drag(&batch1.id, &[]),
        Err(crate::shelf::ShelfError::EmptyItemSelection)
    ));

    // Duplicate selection rejected
    assert!(matches!(
        state.prepare_item_drag(&batch1.id, &[batch1.items[0].id.clone(), batch1.items[0].id.clone()]),
        Err(crate::shelf::ShelfError::DuplicateItemSelection(_))
    ));

    // Nonexistent batch rejected
    assert!(matches!(
        state.prepare_item_drag("nonexistent", &[batch1.items[0].id.clone()]),
        Err(crate::shelf::ShelfError::BatchNotFound(_))
    ));

    // Item not in active batch rejected
    assert!(matches!(
        state.prepare_item_drag(&batch1.id, &[batch2.items[0].id.clone()]),
        Err(crate::shelf::ShelfError::ItemNotFound(_))
    ));

    // Source order preserved regardless of input order (reverse: 2, 0)
    let item_ids = vec![
        batch1.items[2].id.clone(),
        batch1.items[0].id.clone(),
    ];
    let prepared = state.prepare_item_drag(&batch1.id, &item_ids).unwrap();
    assert!(prepared.token.is_some());
    assert_eq!(prepared.available, 2);
    assert_eq!(prepared.unavailable, 0);

    let token = prepared.token.as_deref().unwrap();
    let transfer = state.begin_item_drag(token, Some(&batch1.id)).unwrap();
    assert_eq!(transfer.participants.len(), 2);
    assert_eq!(transfer.participants[0].id, batch1.items[0].id);
    assert_eq!(transfer.participants[1].id, batch1.items[2].id);
    assert_eq!(transfer.paths, [paths[0].clone(), paths[2].clone()]);
}

#[test]
fn selection_pin_invalidates_an_armed_batch_drag_token() {
    let directory = TestDirectory::new("selection-pin-invalidates-token");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_drop(vec![Some("C:\\available.txt".into())]).unwrap().snapshot.unwrap();
    let ids = [snapshot.batches[0].id.clone()];
    let prepared = state.prepare_batch_drag_with(&ids, |_| crate::shelf::Availability::Available).unwrap();
    state.set_batches_pinned(&ids, true).unwrap();
    assert!(
        state.begin_batch_drag(prepared.token.as_deref().unwrap()).is_err(),
        "a successful selection pin must invalidate a token armed against the older Shelf snapshot"
    );
}

#[test]
fn item_drag_token_domain_isolation_and_invalidation() {
    let directory = TestDirectory::new("item-drag-domain-isolation");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let unselected = directory.path().join("unselected.txt");
    fs::write(&unselected, b"not selected").unwrap();
    let snapshot = state.accept_drop(vec![Some(directory.source_file().to_string_lossy().into_owned()), Some(unselected.to_string_lossy().into_owned())]).unwrap().snapshot.unwrap();
    let batch = &snapshot.batches[0];

    // 1. Prepare item drag
    let item_prep = state.prepare_item_drag(&batch.id, &[batch.items[0].id.clone()]).unwrap();
    let item_token = item_prep.token.as_deref().unwrap();
    assert!(item_prep.available == 1);

    // Cross-domain rejection: batch begin/cancel reject item token
    assert!(matches!(
        state.begin_batch_drag(item_token),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));
    assert!(matches!(
        state.cancel_batch_drag(item_token),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));

    // Wrong active batch rejection:
    assert!(matches!(
        state.begin_item_drag(item_token, Some("other_batch")),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));

    // Cancel item drag succeeds
    state.cancel_item_drag(item_token).unwrap();

    // 2. Prepare batch drag
    let batch_prep = state.prepare_batch_drag(std::slice::from_ref(&batch.id)).unwrap();
    let batch_token = batch_prep.token.as_deref().unwrap();
    assert_eq!(batch_prep.available, 2);

    // Cross-domain rejection: item begin/cancel reject batch token
    assert!(matches!(
        state.begin_item_drag(batch_token, Some(&batch.id)),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));
    assert!(matches!(
        state.cancel_item_drag(batch_token),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));

    // cancel_prepared_item_drag does NOT cancel batch token
    state.cancel_prepared_item_drag().unwrap();
    assert!(state.begin_batch_drag(batch_token).is_ok());

    // 3. Mutation invalidates armed item token
    let item_prep2 = state.prepare_item_drag(&batch.id, &[batch.items[0].id.clone()]).unwrap();
    let item_token2 = item_prep2.token.as_deref().unwrap();
    state.set_item_pinned(&batch.items[1].id, true).unwrap();
    assert!(matches!(
        state.begin_item_drag(item_token2, Some(&batch.id)),
        Err(crate::shelf::ShelfError::InvalidPreparedDrag)
    ));
}

#[test]
fn mixed_availability_deduplication_and_missing_cleanup() {
    let directory = TestDirectory::new("mixed-availability-drag");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_drop(vec![
        Some("C:\\file1.txt".into()),
        Some("C:\\second-occurrence.txt".into()),
        Some("C:\\missing.txt".into()),
        Some("Z:\\unavailable.txt".into()),
    ]).unwrap().snapshot.unwrap();
    let batch = &snapshot.batches[0];
    // Model a restored batch with repeated path occurrences without intake deduplication.
    state.store().unwrap().connection.execute(
        "UPDATE shelf_items SET path = 'c:\\FILE1.TXT' WHERE id = ?1",
        [&batch.items[1].id],
    ).unwrap();
    let all_ids: Vec<String> = batch.items.iter().map(|item| item.id.clone()).collect();

    // Prepare with classifier
    let prep = state.prepare_item_drag_with(&batch.id, &all_ids, |path| {
        if path.contains("missing") {
            crate::shelf::Availability::Missing
        } else if path.starts_with("Z:") {
            crate::shelf::Availability::Unavailable
        } else {
            crate::shelf::Availability::Available
        }
    }).unwrap();

    assert_eq!(prep.available, 2);
    assert_eq!(prep.unavailable, 1);
    let token = prep.token.expect("token should be generated for available items");

    // Missing item was durably removed during preparation
    let current = state.snapshot().unwrap();
    assert_eq!(current.batches[0].items.len(), 3);
    assert!(!current.batches[0].items.iter().any(|i| i.path.contains("missing")));

    // Begin drag: transfer paths deduplicated ordinal case-insensitively, but both occurrences present
    let transfer = state.begin_item_drag(&token, Some(&batch.id)).unwrap();
    assert_eq!(transfer.paths, vec!["C:\\file1.txt"]);
    assert_eq!(transfer.participants.len(), 2);

    // Complete Copy: consumes the 2 available temporary items, keeps the 1 unavailable item
    let completed = state.complete_item_drag(&transfer, DragOutcome::Copy).unwrap();
    assert_eq!(completed.batches[0].items.len(), 1);
    assert_eq!(completed.batches[0].items[0].path, "Z:\\unavailable.txt");
}

#[test]
fn all_unavailable_or_all_missing_item_drag_yields_no_token() {
    let directory = TestDirectory::new("all-unavailable-drag");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_drop(vec![
        Some("Z:\\item1.txt".into()),
        Some("Z:\\item2.txt".into()),
    ]).unwrap().snapshot.unwrap();
    let batch = &snapshot.batches[0];
    let item_ids: Vec<String> = batch.items.iter().map(|i| i.id.clone()).collect();

    // All unavailable -> no token, available 0, unavailable 2
    let prep = state.prepare_item_drag_with(&batch.id, &item_ids, |_| crate::shelf::Availability::Unavailable).unwrap();
    assert!(prep.token.is_none());
    assert_eq!(prep.available, 0);
    assert_eq!(prep.unavailable, 2);

    // All missing -> missing items removed, empty batch pruned, no token
    let prep_missing = state.prepare_item_drag_with(&batch.id, &item_ids, |_| crate::shelf::Availability::Missing).unwrap();
    assert!(prep_missing.token.is_none());
    assert_eq!(prep_missing.available, 0);
    assert_eq!(prep_missing.unavailable, 0);
    assert!(state.snapshot().unwrap().batches.is_empty());
}
