use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::PathBuf;

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
        Err(crate::shelf::ShelfError::ItemNotFound(_))
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
fn batch_removal_preview_reports_authoritative_counts_and_remove_is_atomic_and_source_safe() {
    let directory = TestDirectory::new("batches-remove");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let external = directory.path().join("external_source.txt");
    fs::write(&external, b"safe external bytes").unwrap();
    state.accept_drop(vec![Some(external.display().to_string())]).unwrap();
    let text = state.accept_text("managed bytes".into()).unwrap().snapshot.unwrap();
    let managed_path = PathBuf::from(&text.batches[0].items[0].path);
    let kept = directory.path().join("kept.txt");
    fs::write(&kept, b"kept").unwrap();
    let initial = state.accept_drop(vec![Some(kept.display().to_string())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();
    assert_eq!(ids.len(), 3);
    let removing = [ids[1].clone(), ids[2].clone()];

    let preview = state.prepare_batch_removal(&removing).unwrap();
    assert_eq!((preview.batches, preview.items, preview.managed_files), (2, 2, 1));

    assert!(matches!(state.prepare_batch_removal(&[]), Err(crate::shelf::ShelfError::EmptyBatchSelection)));
    assert!(matches!(state.prepare_batch_removal(&["absent".into()]), Err(crate::shelf::ShelfError::BatchNotFound(_))));

    let lock = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new().read(true).share_mode(3).open(&managed_path).unwrap()
    };
    let after = state.confirm_batch_removal(&removing, &preview.token).unwrap();
    assert_eq!(after.batches.len(), 1);
    assert_eq!(after.batches[0].id, ids[0]);
    assert!(managed_path.exists(), "a locked Managed File stays until durable retry succeeds");
    assert_eq!(after.warning.as_deref(), Some("1 DropCove-created file is pending deletion."));
    assert_eq!(fs::read(&external).unwrap(), b"safe external bytes");
    assert_eq!(fs::read(&kept).unwrap(), b"kept", "External sources are never modified");
    drop(lock);
    drop(state);
    let restored = ShelfState::open(&database).unwrap();
    assert!(!managed_path.exists(), "startup retry must finish the durable cleanup");
    assert!(restored.snapshot().unwrap().warning.is_none());
}

#[test]
fn stale_removal_target_rejects_the_whole_operation() {
    let directory = TestDirectory::new("batches-remove-stale");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let a = directory.path().join("a.txt");
    let b = directory.path().join("b.txt");
    let c = directory.path().join("c.txt");
    for (path, bytes) in [(&a, b"a"), (&b, b"b"), (&c, b"c")] {
        fs::write(path, bytes).unwrap();
    }
    state.accept_drop(vec![Some(a.display().to_string()), Some(b.display().to_string())]).unwrap();
    let initial = state.accept_drop(vec![Some(c.display().to_string())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();
    let preview = state.prepare_batch_removal(&ids).unwrap();
    // Shrinking a two-item batch keeps the batch but changes its membership after the preview.
    let shrunk = initial.batches.iter().find(|batch| batch.items.len() == 2).unwrap();
    state.remove_item(&shrunk.items[0].id).unwrap();
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    assert!(matches!(state.confirm_batch_removal(&ids, &preview.token), Err(crate::shelf::ShelfError::StaleRemovalTarget)));
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "a stale target must remove nothing");
}

#[test]
fn failed_batch_removal_commit_leaves_every_target_in_place() {
    let directory = TestDirectory::new("batches-remove-rollback");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let a = directory.path().join("a.txt");
    let b = directory.path().join("b.txt");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    state.accept_drop(vec![Some(a.display().to_string())]).unwrap();
    let initial = state.accept_drop(vec![Some(b.display().to_string())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();
    let preview = state.prepare_batch_removal(&ids).unwrap();
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    let last = initial.batches[1].id.clone();
    state.store().unwrap().connection.execute_batch(&format!(
        "CREATE TEMP TRIGGER fail_remove BEFORE DELETE ON shelf_batches WHEN OLD.id = '{last}'
         BEGIN SELECT RAISE(ABORT, 'forced removal failure'); END;"
    )).unwrap();
    assert!(state.confirm_batch_removal(&ids, &preview.token).is_err());
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before);
    drop(state);
    assert_eq!(serde_json::to_value(ShelfState::open(&database).unwrap().snapshot().unwrap()).unwrap(), before);
}

#[test]
fn item_removal_preview_and_confirm_are_atomic_and_prune_empty_batch() {
    let directory = TestDirectory::new("items-remove");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let external = directory.path().join("external.txt");
    fs::write(&external, b"safe").unwrap();
    state.accept_drop(vec![Some(external.display().to_string()), Some(external.display().to_string())]).unwrap();
    let text = state.accept_text("managed item".into()).unwrap().snapshot.unwrap();
    let multi = state.accept_drop(vec![Some(external.display().to_string()), Some(directory.source_file().display().to_string())]).unwrap().snapshot.unwrap();
    let target_batch = multi.batches.iter().find(|batch| batch.items.len() == 2).unwrap().clone();
    let ids: Vec<String> = target_batch.items.iter().map(|item| item.id.clone()).collect();

    let preview = state.prepare_item_removal(&target_batch.id, &[ids[0].clone()]).unwrap();
    assert_eq!((preview.batches, preview.items, preview.managed_files), (1, 1, 0));

    let after_partial = state.confirm_item_removal(&target_batch.id, &[ids[0].clone()], &preview.token).unwrap();
    let survivor_batch = after_partial.batches.iter().find(|batch| batch.id == target_batch.id).unwrap();
    assert_eq!(survivor_batch.items.len(), 1);
    assert_eq!(survivor_batch.items[0].id, ids[1], "one-item survivor popup requirement");

    assert!(matches!(state.confirm_item_removal(&target_batch.id, &[ids[1].clone()], "forged"), Err(crate::shelf::ShelfError::StaleRemovalTarget)));
    assert!(matches!(state.prepare_item_removal(&target_batch.id, &[]), Err(crate::shelf::ShelfError::EmptyItemSelection)));
    assert!(matches!(state.prepare_item_removal(&target_batch.id, &[text.batches[0].items[0].id.clone()]), Err(crate::shelf::ShelfError::ItemNotFound(_))));

    let second_preview = state.prepare_item_removal(&target_batch.id, &[ids[1].clone()]).unwrap();
    let after_empty = state.confirm_item_removal(&target_batch.id, &[ids[1].clone()], &second_preview.token).unwrap();
    assert!(!after_empty.batches.iter().any(|batch| batch.id == target_batch.id), "removing the final item prunes the batch");
}
