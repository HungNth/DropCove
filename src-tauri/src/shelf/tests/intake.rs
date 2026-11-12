use super::TestDirectory;
use crate::shelf::*;
use std::fs;

#[test]
fn batch_intake_keeps_first_occurrence_and_source_order() {
    let directory = TestDirectory::new("intake-order");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let result = state
        .accept_drop(vec![
            Some("C:\\B.txt".into()),
            Some("c:\\b.TXT".into()),
            None,
            Some("C:\\A.txt".into()),
        ])
        .unwrap();
    let snapshot = result.snapshot.unwrap();
    assert_eq!(
        snapshot.batches[0]
            .items
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>(),
        vec!["C:\\B.txt", "C:\\A.txt"]
    );
    assert_eq!(result.skipped, 1);
}

#[test]
fn exact_path_set_is_noop_but_partial_overlap_is_new_batch() {
    let directory = TestDirectory::new("path-sets");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let initial = state
        .accept_drop(vec![Some("C:\\A.txt".into()), Some("C:\\B.txt".into())])
        .unwrap()
        .snapshot
        .unwrap();
    let duplicate = state
        .accept_drop(vec![
            Some("c:\\b.TXT".into()),
            None,
            Some("c:\\a.txt".into()),
        ])
        .unwrap();
    assert!(duplicate.snapshot.is_none());
    assert_eq!(duplicate.skipped, 0);
    let overlapping = state
        .accept_drop(vec![Some("C:\\A.txt".into())])
        .unwrap()
        .snapshot
        .unwrap();
    assert_eq!(overlapping.batches[1].id, initial.batches[0].id);
    assert_eq!(overlapping.batches[0].items[0].path, "C:\\A.txt");
}

#[test]
fn failed_batch_intake_preserves_visible_and_durable_state() {
    let directory = TestDirectory::new("intake-rollback");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Populate initial batch
    let initial = state
        .accept_drop(vec![Some(
            directory.source_file().to_string_lossy().into_owned(),
        )])
        .unwrap()
        .snapshot
        .unwrap();
    let initial_json = serde_json::to_value(&initial).unwrap();

    // Install trigger that fails item inserts
    state.install_test_insert_failure().unwrap();

    // Attempt to accept a new batch
    let fail_path = directory.0.join("new_file.txt");
    fs::write(&fail_path, b"new").unwrap();
    let result = state.accept_drop(vec![Some(fail_path.to_string_lossy().into_owned())]);
    assert!(result.is_err(), "Intake must fail when insert fails");

    // Verify visible snapshot is identical to prior snapshot
    let visible = state.snapshot().unwrap();
    assert_eq!(
        serde_json::to_value(&visible).unwrap(),
        initial_json,
        "Visible snapshot must be unmodified after failed intake"
    );

    // Reopen database and verify durable snapshot matches prior state
    drop(visible);
    drop(state);
    let reopened = ShelfState::open(&database).unwrap();
    let durable = reopened.snapshot().unwrap();
    assert_eq!(
        serde_json::to_value(&durable).unwrap(),
        initial_json,
        "Durable snapshot after restart must match prior state"
    );
}

#[test]
fn accept_text_round_trip_utf8_restoration_and_copy_consumption() {
    let directory = TestDirectory::new("accept-text-red");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    let sample_text = "DropCove 0.0.23 plain-text test\nLine 2: 🚀 Unicode characters\r\nLine 3: https://example.com/path?arg=1";
    let result = state.accept_text(sample_text.to_string()).unwrap();
    let snapshot = result.snapshot.expect("accept_text must produce a snapshot");
    assert_eq!(snapshot.batches.len(), 1, "exactly one batch created");
    let batch = &snapshot.batches[0];
    assert_eq!(batch.items.len(), 1, "batch has exactly one item");
    let item = &batch.items[0];
    assert!(!item.is_folder);
    assert!(!item.is_pinned);
    assert_eq!(item.name, "DropCove 0.0.23 plain-tex...");
    assert!(item.path.ends_with(".txt"));

    let item_path = std::path::Path::new(&item.path);
    assert!(item_path.is_file(), "managed file must exist on disk");
    let disk_bytes = std::fs::read(item_path).expect("must read managed file");
    assert_eq!(disk_bytes, sample_text.as_bytes(), "file bytes must match UTF-8 without BOM");

    // Restart verification
    drop(state);
    let reopened = ShelfState::open(&database).unwrap();
    let reopened_snapshot = reopened.snapshot().unwrap();
    assert_eq!(reopened_snapshot.batches.len(), 1);
    let restored_item = &reopened_snapshot.batches[0].items[0];
    assert_eq!(restored_item.id, item.id);
    assert_eq!(restored_item.path, item.path);

    // Copy drag-out consumption permanent deletion
    let prep = reopened.prepare_batch_drag(std::slice::from_ref(&batch.id)).unwrap();
    let token = prep.token.expect("token should be generated");
    let transfer = reopened.begin_batch_drag(&token).unwrap();
    let consumed_snap = reopened
        .complete_batch_drag(&transfer, DragOutcome::Copy)
        .unwrap();
    assert!(consumed_snap.batches.is_empty(), "temporary item consumed");
    assert!(!item_path.exists(), "managed file permanently deleted after Copy consumption");
}

#[test]
fn accept_text_boundary_and_rejections() {
    let directory = TestDirectory::new("accept-text-bounds");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Empty text is rejected
    let res = state.accept_text("".to_string()).unwrap();
    assert!(res.snapshot.is_none());
    assert_eq!(res.skipped, 0);

    // Terminal NUL only text is rejected
    let res = state.accept_text("\0".to_string()).unwrap();
    assert!(res.snapshot.is_none());
    assert_eq!(res.skipped, 0);

    // Whitespace-only text is accepted
    let res = state.accept_text("   \n\t  ".to_string()).unwrap();
    assert!(res.snapshot.is_some());
    let snap = res.snapshot.unwrap();
    assert_eq!(snap.batches.len(), 1);

    // 10 MiB text exact boundary is accepted
    let exact_10mb = vec![b'a'; 10 * 1024 * 1024];
    let text_10mb = String::from_utf8(exact_10mb).unwrap();
    let res = state.accept_text(text_10mb).unwrap();
    assert!(res.snapshot.is_some());

    // 10 MiB + 1 byte is rejected
    let over_10mb = vec![b'b'; 10 * 1024 * 1024 + 1];
    let text_over = String::from_utf8(over_10mb).unwrap();
    let res = state.accept_text(text_over).unwrap();
    assert!(res.snapshot.is_none());
    assert_eq!(res.skipped, 1);
}
