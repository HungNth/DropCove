use super::TestDirectory;
use crate::shelf::*;

#[test]
fn automatic_sizing_fits_rows_and_shrinks_after_removal_and_width_changes() {
    let directory = TestDirectory::new("sizing");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    for (index, expected) in [180.0, 180.0, 230.0, 282.0, 334.0, 334.0].into_iter().enumerate() {
        let snapshot = state.accept_drop(vec![Some(format!("C:\\{index}.txt"))]).unwrap().snapshot.unwrap();
        assert_eq!(snapshot.height, expected, "accepted batch {}", index + 1);
    }
    for (width, expected) in [(348.0, 230.0), (516.0, 180.0), (347.0, 334.0)] {
        assert_eq!(state.save_size(width, None).unwrap().height, expected);
    }
    for expected in [334.0, 282.0, 230.0, 180.0, 180.0] {
        let id = state.snapshot().unwrap().batches[0].id.clone();
        assert_eq!(state.remove_batch(&id).unwrap().height, expected);
    }
    drop(state);
    let restored = ShelfState::open(database).unwrap();
    let snapshot = restored.snapshot().unwrap();
    assert_eq!((snapshot.preferred_width, snapshot.height), (347.0, 180.0));
    let empty = restored.remove_batch(&snapshot.batches[0].id).unwrap();
    assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));
}

#[test]
fn batch_pruning_reclaims_automatic_height_but_preserves_manual_height() {
    for manual in [None, Some(400.0)] {
        for operation in ["remove-batch", "remove-item", "clear", "item-copy", "batch-copy", "missing"] {
            let directory = TestDirectory::new(operation);
            let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
            for index in 0..4 {
                let snapshot = state.accept_drop(vec![Some(format!("C:\\keep-{index}.txt"))]).unwrap().snapshot.unwrap();
                state.set_batch_pinned(&snapshot.batches[0].id, true).unwrap();
            }
            let incoming = if operation == "remove-item" {
                vec![Some("C:\\remove-a.txt".into()), Some("C:\\remove-b.txt".into())]
            } else { vec![Some("C:\\remove-a.txt".into())] };
            let snapshot = state.accept_drop(incoming).unwrap().snapshot.unwrap();
            let batch = &snapshot.batches[0];
            state.save_size(191.0, manual).unwrap();
            let result = match operation {
                "remove-batch" => state.remove_batch(&batch.id).unwrap(),
                "remove-item" => {
                    let unchanged = state.remove_item(&batch.items[0].id).unwrap();
                    assert_eq!(unchanged.height, manual.unwrap_or(334.0));
                    state.remove_item(&batch.items[1].id).unwrap()
                }
                "clear" => state.clear_temporary().unwrap(),
                "item-copy" => {
                    let prep = state.prepare_item_drag_with(&batch.id, std::slice::from_ref(&batch.items[0].id), |_| crate::shelf::Availability::Available).unwrap();
                    let transfer = state.begin_item_drag(prep.token.as_deref().unwrap(), Some(&batch.id)).unwrap();
                    state.complete_item_drag(&transfer, DragOutcome::Copy).unwrap()
                }
                "batch-copy" => {
                    let prepared = state.prepare_batch_drag_with(std::slice::from_ref(&batch.id), |_| crate::shelf::Availability::Available).unwrap();
                    let transfer = state.begin_batch_drag(prepared.token.as_deref().unwrap()).unwrap();
                    state.complete_batch_drag(&transfer, DragOutcome::Copy).unwrap()
                }
                "missing" => state.prepare_batch_drag_with(std::slice::from_ref(&batch.id), |_| crate::shelf::Availability::Missing).unwrap().snapshot,
                _ => unreachable!(),
            };
            assert_eq!(result.batches.len(), 4, "{operation}");
            assert_eq!(result.height, manual.unwrap_or(282.0), "{operation}");
            assert_eq!(state.snapshot().unwrap().height, result.height, "{operation}");
        }
    }
}

#[test]
fn restored_width_respects_minimum() {
    let directory = TestDirectory::new("minimum-width");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    state.accept_drop(vec![Some("C:\\first.txt".into())]).unwrap();
    state.accept_drop(vec![Some("C:\\second.txt".into())]).unwrap();
    state.save_size(191.0, None).unwrap();
    drop(state);
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.execute("UPDATE shelf_settings SET preferred_width = 180", []).unwrap();
    drop(connection);
    let restored = ShelfState::open(&database).unwrap();
    let snapshot = restored.snapshot().unwrap();
    assert_eq!(snapshot.preferred_width, 191.0);
    assert_eq!(snapshot.batches.len(), 2);
}

#[test]
fn save_size_validation_rejects_below_minimum_and_non_finite() {
    let directory = TestDirectory::new("size-validation");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();

    assert!(matches!(
        state.save_size(190.9, None),
        Err(crate::shelf::ShelfError::InvalidWidth)
    ));
    assert!(matches!(
        state.save_size(f64::NAN, None),
        Err(crate::shelf::ShelfError::InvalidWidth)
    ));
    assert!(matches!(
        state.save_size(f64::INFINITY, None),
        Err(crate::shelf::ShelfError::InvalidWidth)
    ));
    assert!(matches!(
        state.save_size(200.0, Some(179.9)),
        Err(crate::shelf::ShelfError::InvalidHeight)
    ));
    assert!(matches!(
        state.save_size(200.0, Some(f64::NAN)),
        Err(crate::shelf::ShelfError::InvalidHeight)
    ));
    assert!(matches!(
        state.save_size(200.0, Some(f64::NEG_INFINITY)),
        Err(crate::shelf::ShelfError::InvalidHeight)
    ));
}

#[test]
fn width_only_eligibility_and_existing_override_preservation() {
    let directory = TestDirectory::new("width-eligibility");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Intake 1 batch
    state
        .accept_drop(vec![Some("C:\\file1.txt".into())])
        .unwrap();
    assert_eq!(state.snapshot().unwrap().height, 180.0);

    // Width-only resize: height is None, auto-growth eligibility preserved
    let snap = state.save_size(250.0, None).unwrap();
    assert_eq!(snap.preferred_width, 250.0);
    assert_eq!(snap.height, 180.0);

    // Add more batches: auto-growth kicks in because no manual override exists
    for i in 2..=4 {
        state
            .accept_drop(vec![Some(format!("C:\\file{i}.txt"))])
            .unwrap();
    }
    let grown = state.snapshot().unwrap();
    assert_eq!(grown.height, 282.0);

    // Now set manual override
    let manual_snap = state.save_size(250.0, Some(220.0)).unwrap();
    assert_eq!(manual_snap.height, 220.0);

    // Another width-only resize with None must PRESERVE the existing manual override
    let width_only = state.save_size(300.0, None).unwrap();
    assert_eq!(width_only.preferred_width, 300.0);
    assert_eq!(width_only.height, 220.0);
}

#[test]
fn empty_pending_intent_and_first_valid_intake_atomicity() {
    let directory = TestDirectory::new("empty-pending");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Empty shelf: resize width + manual height
    let empty_snap = state.save_size(320.0, Some(260.0)).unwrap();
    assert_eq!(empty_snap.preferred_width, 320.0);
    assert_eq!(empty_snap.height, 260.0);
    assert!(empty_snap.batches.is_empty());

    // Check reading empty snapshot preserves pending intent
    let read_snap = state.snapshot().unwrap();
    assert_eq!(read_snap.preferred_width, 320.0);
    assert_eq!(read_snap.height, 260.0);

    // No settings row in SQLite while empty
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shelf_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // Unsupported/empty drop must be no-op: does not create settings row or alter intent
    let drop_empty = state.accept_drop(vec![None, Some("".into())]).unwrap();
    assert!(drop_empty.snapshot.is_none());
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shelf_settings", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
    let check_snap = state.snapshot().unwrap();
    assert_eq!(check_snap.preferred_width, 320.0);
    assert_eq!(check_snap.height, 260.0);

    // First valid accepted batch commits pending intent into shelf_settings atomically
    let accepted = state
        .accept_drop(vec![Some("C:\\first.txt".into())])
        .unwrap();
    let snap = accepted.snapshot.unwrap();
    assert_eq!(snap.preferred_width, 320.0);
    assert_eq!(snap.height, 260.0);

    // Verify settings row exists with persisted manual height
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        let (pw, mh): (f64, Option<f64>) = conn
            .query_row(
                "SELECT preferred_width, manual_height FROM shelf_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(pw, 320.0);
        assert_eq!(mh, Some(260.0));
    }

    // Duplicate drop is a no-op and does not alter persistence
    let dup = state
        .accept_drop(vec![Some("C:\\first.txt".into())])
        .unwrap();
    assert!(dup.snapshot.is_none());
}

#[test]
fn restart_manual_versus_automatic_behavior() {
    let directory = TestDirectory::new("restart-manual-vs-auto");
    let database = directory.path().join("shelf.db");

    // Case 1: Manual override persists across restart
    {
        let state = ShelfState::open(&database).unwrap();
        state
            .accept_drop(vec![Some("C:\\item.txt".into())])
            .unwrap();
        state.save_size(280.0, Some(250.0)).unwrap();
    }
    {
        let restored = ShelfState::open(&database).unwrap();
        let snap = restored.snapshot().unwrap();
        assert_eq!(snap.preferred_width, 280.0);
        assert_eq!(snap.height, 250.0);
    }

    // Clear shelf
    {
        let state = ShelfState::open(&database).unwrap();
        state.clear_temporary().unwrap();
    }

    // Case 2: Automatic growth does NOT persist manual intent; recalculates on restart
    {
        let state = ShelfState::open(&database).unwrap();
        for i in 0..4 {
            state
                .accept_drop(vec![Some(format!("C:\\file{i}.txt"))])
                .unwrap();
        }
        assert_eq!(state.snapshot().unwrap().height, 282.0);
        // Set preferred width only (None for manual height)
        state.save_size(300.0, None).unwrap();
        // Removing complete batches reclaims unused automatic height immediately.
        let snap = state.snapshot().unwrap();
        for batch in &snap.batches[..3] {
            state.remove_batch(&batch.id).unwrap();
        }
        assert_eq!(state.snapshot().unwrap().height, 180.0);
    }
    {
        // On restart, automatic height recalculates to 180.0 for 1 batch
        let restored = ShelfState::open(&database).unwrap();
        let snap = restored.snapshot().unwrap();
        assert_eq!(snap.preferred_width, 300.0);
        assert_eq!(snap.height, 180.0);
    }
}

#[test]
fn final_item_removal_resets_manual_and_staged_sizing_to_minimum() {
    let directory = TestDirectory::new("final-item-reset");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // 1. Remove final item via remove_batch
    state
        .accept_drop(vec![Some("C:\\batch1.txt".into())])
        .unwrap();
    state.save_size(240.0, Some(220.0)).unwrap();
    let snap = state.snapshot().unwrap();
    let empty = state.remove_batch(&snap.batches[0].id).unwrap();
    assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));

    // Next intake after final removal defaults to 191x180.
    let reintake = state
        .accept_drop(vec![Some("C:\\batch2.txt".into())])
        .unwrap();
    assert_eq!(
        (
            reintake.snapshot.as_ref().unwrap().preferred_width,
            reintake.snapshot.as_ref().unwrap().height
        ),
        (191.0, 180.0)
    );

    // 2. Remove final item via consume_items (Copy outcome)
    state.save_size(270.0, Some(230.0)).unwrap();
    let snap = state.snapshot().unwrap();
    let prep = state
        .prepare_item_drag_with(&snap.batches[0].id, std::slice::from_ref(&snap.batches[0].items[0].id), |_| crate::shelf::Availability::Available)
        .unwrap();
    let transfer = state
        .begin_item_drag(prep.token.as_deref().unwrap(), Some(&snap.batches[0].id))
        .unwrap();
    let empty = state
        .complete_item_drag(&transfer, DragOutcome::Copy)
        .unwrap();
    assert_eq!((empty.preferred_width, empty.height), (191.0, 180.0));

    // 3. Remove final item via preparation pruning (missing path)
    state
        .accept_drop(vec![Some("C:\\missing_item.txt".into())])
        .unwrap();
    state.save_size(290.0, Some(240.0)).unwrap();
    let snap = state.snapshot().unwrap();
    let prep = state
        .prepare_batch_drag_with(std::slice::from_ref(&snap.batches[0].id), |_| crate::shelf::Availability::Missing)
        .unwrap();
    assert_eq!(
        (prep.snapshot.preferred_width, prep.snapshot.height),
        (191.0, 180.0)
    );
}

#[test]
fn failed_sizing_transaction_and_staged_commit_rollback() {
    let directory = TestDirectory::new("sizing-rollback");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Case 1: Failed save_size on populated shelf preserves previous retained intent
    state
        .accept_drop(vec![Some("C:\\item.txt".into())])
        .unwrap();
    state.save_size(250.0, Some(210.0)).unwrap();

    // Install trigger that fails updates on shelf_settings
    state.install_test_settings_failure().unwrap();

    let fail_res = state.save_size(300.0, Some(250.0));
    assert!(fail_res.is_err());

    // Previous cached intent retained
    let cached = state.snapshot().unwrap();
    assert_eq!(cached.preferred_width, 250.0);
    assert_eq!(cached.height, 210.0);

    // Drop and reopen to verify SQLite state was also rolled back
    drop(cached);
    drop(state);
    let reopened = ShelfState::open(&database).unwrap();
    let reopened_snap = reopened.snapshot().unwrap();
    assert_eq!(reopened_snap.preferred_width, 250.0);
    assert_eq!(reopened_snap.height, 210.0);

    // Case 2: Failed batch insert rolls back staged commit and keeps staged intent pending
    reopened.clear_temporary().unwrap();
    reopened.save_size(320.0, Some(240.0)).unwrap();

    // Install trigger that fails item inserts
    reopened.install_test_insert_failure().unwrap();

    let insert_res = reopened.accept_drop(vec![Some("C:\\fail.txt".into())]);
    assert!(insert_res.is_err());

    // Cached state is still pending staged intent
    let pending_snap = reopened.snapshot().unwrap();
    assert_eq!(pending_snap.preferred_width, 320.0);
    assert_eq!(pending_snap.height, 240.0);
}
