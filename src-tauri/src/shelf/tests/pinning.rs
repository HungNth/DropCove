use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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
    let batch_id = &restored.snapshot().unwrap().batches[0].id;
    for outcome in [
        DragOutcome::Canceled,
        DragOutcome::Rejected,
        DragOutcome::Copy,
    ] {
        let prep = restored.prepare_item_drag(batch_id, std::slice::from_ref(item_id)).unwrap();
        let transfer = restored.begin_item_drag(prep.token.as_deref().unwrap(), Some(batch_id)).unwrap();
        let retained = restored.complete_item_drag(&transfer, outcome).unwrap();
        assert_eq!(serde_json::to_value(retained).unwrap(), expected);
    }
    restored.set_item_pinned(item_id, false).unwrap();
    drop(restored);
    let temporary = ShelfState::open(&database).unwrap();
    assert!(!temporary.snapshot().unwrap().batches[0].items[0].is_pinned);
    let temp_batch_id = &temporary.snapshot().unwrap().batches[0].id;
    for outcome in [DragOutcome::Canceled, DragOutcome::Rejected] {
        let prep = temporary.prepare_item_drag(temp_batch_id, std::slice::from_ref(item_id)).unwrap();
        let transfer = temporary.begin_item_drag(prep.token.as_deref().unwrap(), Some(temp_batch_id)).unwrap();
        assert_eq!(
            temporary
                .complete_item_drag(&transfer, outcome)
                .unwrap()
                .batches[0]
                .items[0]
                .id,
            *item_id
        );
    }
    let prep = temporary.prepare_item_drag(temp_batch_id, std::slice::from_ref(item_id)).unwrap();
    let transfer = temporary.begin_item_drag(prep.token.as_deref().unwrap(), Some(temp_batch_id)).unwrap();
    assert!(temporary
        .complete_item_drag(&transfer, DragOutcome::Copy)
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
        Err(crate::shelf::ShelfError::ItemNotFound(_))
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
    let prep1 = state.prepare_item_drag(batch_id, std::slice::from_ref(item1_id)).unwrap();
    let transfer1 = state.begin_item_drag(prep1.token.as_deref().unwrap(), Some(batch_id)).unwrap();
    let snap_after_pinned_drag = state
        .complete_item_drag(&transfer1, DragOutcome::Copy)
        .unwrap();
    assert_eq!(snap_after_pinned_drag.batches[0].items.len(), 2);

    // Complete item drag for temporary item2 as Copy - should consume only item2
    let prep2 = state.prepare_item_drag(batch_id, std::slice::from_ref(item2_id)).unwrap();
    let transfer2 = state.begin_item_drag(prep2.token.as_deref().unwrap(), Some(batch_id)).unwrap();
    let snap_after_temp_drag = state
        .complete_item_drag(&transfer2, DragOutcome::Copy)
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
    use crate::shelf::Availability;
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
        Err(crate::shelf::ShelfError::BatchNotFound(_))
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
fn set_batches_pinned_changes_every_selected_item_atomically_and_rejects_invalid_targets() {
    let directory = TestDirectory::new("batches-pin");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let files: Vec<String> = ["a", "b", "c", "d"]
        .iter()
        .map(|name| {
            let path = directory.0.join(format!("{name}.txt"));
            fs::write(&path, name).unwrap();
            path.to_string_lossy().into_owned()
        })
        .collect();
    state.accept_drop(vec![Some(files[0].clone()), Some(files[1].clone())]).unwrap();
    state.accept_drop(vec![Some(files[2].clone())]).unwrap();
    let initial = state.accept_drop(vec![Some(files[3].clone())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();

    let pinned = state.set_batches_pinned(&ids[..2], true).unwrap();
    let pins: Vec<Vec<bool>> = pinned.batches.iter().map(|batch| batch.items.iter().map(|item| item.is_pinned).collect()).collect();
    assert_eq!(pins, vec![vec![true], vec![true], vec![false, false]]);

    assert!(matches!(state.set_batches_pinned(&[], true), Err(crate::shelf::ShelfError::EmptyBatchSelection)));
    assert!(matches!(
        state.set_batches_pinned(&[ids[0].clone(), ids[0].clone()], false),
        Err(crate::shelf::ShelfError::DuplicateBatchSelection(_))
    ));
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    assert!(matches!(
        state.set_batches_pinned(&[ids[0].clone(), "absent".into()], false),
        Err(crate::shelf::ShelfError::BatchNotFound(_))
    ));
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "a stale target must change nothing");

    let item_id = pinned.batches[2].items[1].id.clone();
    state.store().unwrap().connection.execute_batch(&format!(
        "CREATE TEMP TRIGGER fail_halfway BEFORE UPDATE OF is_pinned ON shelf_items WHEN NEW.id = '{item_id}'
         BEGIN SELECT RAISE(ABORT, 'forced failure'); END;"
    )).unwrap();
    assert!(state.set_batches_pinned(&ids, true).is_err());
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "persistence failure must change nothing");
    drop(state);
    let reopened = ShelfState::open(&database).unwrap();
    assert_eq!(serde_json::to_value(reopened.snapshot().unwrap()).unwrap(), before);
}

#[test]
fn set_items_pinned_changes_exactly_the_selected_items_and_rejects_cross_batch_and_invalid_targets() {
    let directory = TestDirectory::new("items-pin");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let files: Vec<String> = ["a", "b", "c", "d"]
        .iter()
        .map(|name| {
            let path = directory.0.join(format!("{name}.txt"));
            fs::write(&path, name).unwrap();
            path.to_string_lossy().into_owned()
        })
        .collect();
    state.accept_drop(vec![Some(files[0].clone()), Some(files[1].clone()), Some(files[2].clone())]).unwrap();
    let initial = state.accept_drop(vec![Some(files[3].clone())]).unwrap().snapshot.unwrap();
    let multi = initial.batches.iter().find(|batch| batch.items.len() == 3).unwrap().clone();
    let other = initial.batches.iter().find(|batch| batch.items.len() == 1).unwrap().clone();
    let ids: Vec<String> = multi.items.iter().map(|item| item.id.clone()).collect();

    let pinned = state.set_items_pinned(&multi.id, &[ids[0].clone(), ids[2].clone()], true).unwrap();
    let after = pinned.batches.iter().find(|batch| batch.id == multi.id).unwrap();
    let flags: Vec<bool> = after.items.iter().map(|item| item.is_pinned).collect();
    assert_eq!(flags, vec![true, false, true], "only the selected items change, in source order");
    assert!(pinned.batches.iter().find(|batch| batch.id == other.id).unwrap().items.iter().all(|item| !item.is_pinned));

    assert!(matches!(state.set_items_pinned(&multi.id, &[], true), Err(crate::shelf::ShelfError::EmptyItemSelection)));
    assert!(matches!(
        state.set_items_pinned(&multi.id, &[ids[0].clone(), ids[0].clone()], false),
        Err(crate::shelf::ShelfError::DuplicateItemSelection(_))
    ));
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    assert!(matches!(
        state.set_items_pinned(&multi.id, &[ids[0].clone(), other.items[0].id.clone()], false),
        Err(crate::shelf::ShelfError::ItemNotFound(_))
    ), "an item from another batch must reject the whole operation");
    assert!(matches!(state.set_items_pinned("absent", &[ids[0].clone()], false), Err(crate::shelf::ShelfError::BatchNotFound(_))));
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "a rejected target must change nothing");

    state.store().unwrap().connection.execute_batch(&format!(
        "CREATE TEMP TRIGGER fail_item_pin BEFORE UPDATE OF is_pinned ON shelf_items WHEN NEW.id = '{}'
         BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
        ids[2]
    )).unwrap();
    assert!(state.set_items_pinned(&multi.id, &ids, false).is_err());
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "persistence failure must change nothing");
    drop(state);
    assert_eq!(serde_json::to_value(ShelfState::open(&database).unwrap().snapshot().unwrap()).unwrap(), before);
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
fn pinned_managed_copy_retains_file_until_explicit_removal() {
    let directory = TestDirectory::new("managed-pinned-copy");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_text("keep these exact bytes".into()).unwrap().snapshot.unwrap();
    let batch_id = &snapshot.batches[0].id;
    let item = &snapshot.batches[0].items[0];
    state.set_item_pinned(&item.id, true).unwrap();
    let prep = state.prepare_item_drag(batch_id, std::slice::from_ref(&item.id)).unwrap();
    let transfer = state.begin_item_drag(prep.token.as_deref().unwrap(), Some(batch_id)).unwrap();
    let retained = state.complete_item_drag(&transfer, DragOutcome::Copy).unwrap();
    assert_eq!(retained.batches[0].items[0].id, item.id);
    assert_eq!(fs::read(&item.path).unwrap(), b"keep these exact bytes");
    state.remove_item(&item.id).unwrap();
    assert!(!Path::new(&item.path).exists());
}
