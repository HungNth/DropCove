use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::Path;

#[test]
fn management_batch_refreshes_availability_without_cleanup_or_mutating_order() {
    use crate::shelf::Availability;
    let directory = TestDirectory::new("mgmt-batch");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let file_a = directory.0.join("a.txt");
    let file_b = directory.0.join("b.txt");
    let file_c = directory.0.join("c.txt");
    fs::write(&file_a, b"a").unwrap();
    fs::write(&file_b, b"b").unwrap();
    fs::write(&file_c, b"c").unwrap();

    let initial = state
        .accept_drop(vec![
            Some(file_a.to_string_lossy().into_owned()),
            Some(file_b.to_string_lossy().into_owned()),
            Some(file_c.to_string_lossy().into_owned()),
        ])
        .unwrap()
        .snapshot
        .unwrap();
    let batch_id = &initial.batches[0].id;

    let managed = state
        .management_batch_with(batch_id, |path| {
            if path.ends_with("a.txt") {
                Availability::Available
            } else if path.ends_with("b.txt") {
                Availability::Missing
            } else {
                Availability::Unavailable
            }
        })
        .unwrap();

    assert_eq!(managed.items.len(), 3);
    assert_eq!(managed.items[0].availability, Availability::Available);
    assert_eq!(managed.items[1].availability, Availability::Missing);
    assert_eq!(managed.items[2].availability, Availability::Unavailable);
    assert_eq!(managed.items[0].position, 0);
    assert_eq!(managed.items[1].position, 1);
    assert_eq!(managed.items[2].position, 2);

    // Verifying Missing items were NOT purged from the shelf
    let snapshot = state.snapshot().unwrap();
    assert_eq!(snapshot.batches[0].items.len(), 3);
    assert_eq!(
        snapshot.batches[0].items[1].availability,
        Availability::Missing
    );
}

#[test]
fn pending_cleanup_warning_survives_snapshot_and_restart() {
    use std::os::windows::fs::OpenOptionsExt;
    let directory = TestDirectory::new("managed-pending-snapshot");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let snapshot = state.accept_text("temporarily locked".into()).unwrap().snapshot.unwrap();
    let batch = &snapshot.batches[0];
    let path = &batch.items[0].path;
    let lock = fs::OpenOptions::new().read(true).share_mode(3).open(path).unwrap();
    state.remove_batch(&batch.id).unwrap();
    assert_eq!(state.snapshot().unwrap().warning.as_deref(), Some("1 DropCove-created file is pending deletion."));
    drop(state);
    let restored = ShelfState::open(&database).unwrap();
    assert_eq!(restored.snapshot().unwrap().warning.as_deref(), Some("1 DropCove-created file is pending deletion."));
    drop(lock);
    restored.clear_temporary().unwrap();
    assert!(!Path::new(path).exists());
    assert!(restored.snapshot().unwrap().warning.is_none());
}
