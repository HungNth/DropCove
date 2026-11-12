use super::TestDirectory;
use crate::shelf::*;
use std::fs;

#[test]
fn close_releases_database_and_rejects_retained_state_clones() {
    let directory = TestDirectory::new("resident-close");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    state
        .accept_drop(vec![Some(
            directory.source_file().to_string_lossy().into_owned(),
        )])
        .unwrap();
    state.save_size(300.0, Some(220.0)).unwrap();
    let retained = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    let clone = state.clone();
    state.close().unwrap();
    assert!(matches!(
        clone.snapshot(),
        Err(crate::shelf::ShelfError::StateClosed)
    ));
    let reopened = ShelfState::open(&database).unwrap();
    assert_eq!(
        serde_json::to_value(reopened.snapshot().unwrap()).unwrap(),
        retained
    );
    reopened.close().unwrap();
    assert!(!database.with_extension("db-wal").exists());
}

#[test]
fn accepted_path_is_restored_after_reopen() {
    let directory = TestDirectory::new("restore");
    let database = directory.path().join("shelf.db");
    let source = directory.source_file();
    let expected_path = source.to_string_lossy().into_owned();

    let state = ShelfState::open(&database).expect("shelf should open");
    state
        .accept_drop(vec![Some(expected_path.clone())])
        .expect("path should be accepted");
    drop(state);

    let restored = ShelfState::open(&database)
        .expect("shelf should reopen")
        .snapshot()
        .expect("snapshot should load");

    assert_eq!(restored.batches[0].items[0].path, expected_path);
}

#[test]
fn failed_rail_monitor_persistence_preserves_prior_durable_value_and_reopen() {
    let directory = TestDirectory::new("rail-mon-fail");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Initially set remembered rail monitor to "display-1"
    state.remember_rail_monitor(Some("display-1")).unwrap();
    assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

    // Verify durable state on reopen
    drop(state);
    let state = ShelfState::open(&database).unwrap();
    assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

    // Install trigger failing shelf_ui_state updates and inserts
    state.install_test_rail_monitor_failure().unwrap();

    // Attempt to update rail monitor to "display-2"
    let res = state.remember_rail_monitor(Some("display-2"));
    assert!(res.is_err(), "Updating rail monitor must fail when trigger aborts");

    // Querying rail_monitor returns the unchanged durable value ("display-1")
    assert_eq!(state.rail_monitor().unwrap().as_deref(), Some("display-1"));

    // Reopen database: restart reflects the actual durable value ("display-1")
    drop(state);
    let reopened = ShelfState::open(&database).unwrap();
    assert_eq!(reopened.rail_monitor().unwrap().as_deref(), Some("display-1"));
}

#[test]
fn confirming_after_a_previewed_batch_disappears_is_stale_not_not_found() {
    let directory = TestDirectory::new("batches-remove-vanished");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let a = directory.path().join("a.txt");
    let b = directory.path().join("b.txt");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    state.accept_drop(vec![Some(a.display().to_string())]).unwrap();
    let initial = state.accept_drop(vec![Some(b.display().to_string())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();
    let preview = state.prepare_batch_removal(&ids).unwrap();
    state.remove_batch(&ids[0]).unwrap();
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    assert!(matches!(state.confirm_batch_removal(&ids, &preview.token), Err(crate::shelf::ShelfError::StaleRemovalTarget)));
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "a vanished target must remove nothing");
    assert!(matches!(state.prepare_batch_removal(&ids), Err(crate::shelf::ShelfError::BatchNotFound(_))), "a fresh invalid request still reports BatchNotFound");
}

#[test]
fn confirm_with_a_caller_supplied_token_rejects_a_forged_token_and_keeps_real_errors() {
    let directory = TestDirectory::new("batches-remove-token");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let a = directory.path().join("a.txt");
    let b = directory.path().join("b.txt");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();
    state.accept_drop(vec![Some(a.display().to_string())]).unwrap();
    let initial = state.accept_drop(vec![Some(b.display().to_string())]).unwrap().snapshot.unwrap();
    let ids: Vec<String> = initial.batches.iter().map(|batch| batch.id.clone()).collect();
    let real = state.prepare_batch_removal(&ids).unwrap();
    let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();

    assert!(matches!(state.confirm_batch_removal(&ids, "forged"), Err(crate::shelf::ShelfError::StaleRemovalTarget)));
    assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before, "a forged token must remove nothing");
    assert!(matches!(state.confirm_batch_removal(&[], &real.token), Err(crate::shelf::ShelfError::EmptyBatchSelection)), "validation errors stay real, not stale");

    let after = state.confirm_batch_removal(&ids, &real.token).unwrap();
    assert!(after.batches.is_empty());
}
