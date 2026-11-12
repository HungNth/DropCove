use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn managed_file_sharing_violation_retry_and_external_safety() {
    let directory = TestDirectory::new("sharing-retry");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // External file drop
    let external_file = directory.path().join("external_source.txt");
    fs::write(&external_file, b"safe external bytes").unwrap();
    let _drop_res = state
        .accept_drop(vec![Some(external_file.display().to_string())])
        .unwrap();

    // Managed text drop
    let text_res = state.accept_text("managed text content".to_string()).unwrap();
    let text_snap = text_res.snapshot.unwrap();
    let text_batch = &text_snap.batches[0];
    let text_item = &text_batch.items[0];
    let text_path = PathBuf::from(&text_item.path);

    // Hold the managed file open on Windows denying delete
    #[cfg(windows)]
    use std::os::windows::fs::OpenOptionsExt;
    let lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2) // FILE_SHARE_READ | FILE_SHARE_WRITE (no FILE_SHARE_DELETE)
        .open(&text_path)
        .unwrap();

    // Remove batch while locked: mutation commits in SQLite, pending deletion queued, file remains
    let snap_after_remove = state.remove_batch(&text_batch.id).unwrap();
    assert!(text_path.exists(), "file remains on disk due to sharing violation");
    assert_eq!(
        snap_after_remove.warning.as_deref(),
        Some("1 DropCove-created file is pending deletion.")
    );

    // External file must remain untouched
    assert_eq!(fs::read(&external_file).unwrap(), b"safe external bytes");

    // Release lock
    drop(lock);

    // Later mutation triggers cleanup retry
    let retry_snap = state.clear_temporary().unwrap();
    assert!(!text_path.exists(), "managed file deleted after lock release and retry");
    assert!(retry_snap.warning.is_none(), "pending warning cleared");
    // External file was temporary so reference removed, but source object remains untouched!
    assert_eq!(fs::read(&external_file).unwrap(), b"safe external bytes");
}

#[test]
fn startup_reconciles_promoted_and_staged_states() {
    let directory = TestDirectory::new("crash-reconciliation");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Seed state: a staged managed file that was promoted to final path before crash
    let store = crate::shelf::managed_files::ManagedStore::new(directory.path());
    store.init_directories().unwrap();
    let item_id = "0123456789abcdef0123456789abcdef";
    let filename = "Dropped text 2026-10-23 12-00-00.txt";
    let staging_rel = crate::shelf::managed_files::ManagedStore::staging_rel_path(item_id);
    let final_rel = crate::shelf::managed_files::ManagedStore::final_rel_path(item_id, filename);
    let final_full = store.full_path(&final_rel).unwrap();
    if let Some(p) = final_full.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&final_full, b"promoted bytes").unwrap();

    // Seed SQLite as 'staged'
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b1', 100, 0)", []).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b1', 0, ?2, ?3, 0, 0)",
            [item_id, final_full.to_str().unwrap(), filename],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [item_id, &final_rel, &staging_rel],
        ).unwrap();
    }
    drop(state);

    // Reopen database: startup reconciliation must promote 'staged' to 'ready'
    let reopened = ShelfState::open(&database).unwrap();
    let snap = reopened.snapshot().unwrap();
    assert_eq!(snap.batches.len(), 1, "reconciled item becomes visible");
    assert_eq!(snap.batches[0].items[0].id, item_id);
    assert_eq!(fs::read(&final_full).unwrap(), b"promoted bytes");
}

#[test]
fn missing_managed_file_removal_also_removes_its_empty_directory() {
    let directory = TestDirectory::new("managed-missing-directory");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    let snapshot = state.accept_text("discard me".into()).unwrap().snapshot.unwrap();
    let batch = &snapshot.batches[0];
    let file = PathBuf::from(&batch.items[0].path);
    let item_directory = file.parent().unwrap().to_path_buf();
    fs::remove_file(&file).unwrap();
    state.remove_batch(&batch.id).unwrap();
    assert!(!item_directory.exists());
    assert!(state.snapshot().unwrap().warning.is_none());
}

#[test]
fn failed_ready_publication_compensates_without_leaking_managed_bytes() {
    let directory = TestDirectory::new("managed-ready-rollback");
    let state = ShelfState::open(directory.path().join("shelf.db")).unwrap();
    state.store().unwrap().connection.execute_batch(
        "CREATE TEMP TRIGGER reject_ready BEFORE UPDATE OF state ON managed_files WHEN NEW.state = 'ready' BEGIN SELECT RAISE(ABORT, 'ready blocked'); END;"
    ).unwrap();
    assert!(state.accept_text("must not appear".into()).is_err());
    assert!(state.snapshot().unwrap().batches.is_empty());
    assert_eq!(fs::read_dir(directory.path().join("managed")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(directory.path().join("staging")).unwrap().count(), 0);
}

#[test]
fn clear_temporary_mixed_ownership_preserves_external_and_cleans_managed() {
    let directory = TestDirectory::new("clear-mixed-ownership");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let external = directory.source_file();
    state.accept_drop(vec![Some(external.to_str().unwrap().to_owned())]).unwrap();
    let managed = state.accept_text("managed text for clear".into()).unwrap();
    let managed_item = &managed.snapshot.unwrap().batches[0].items[0];
    let managed_path = std::path::PathBuf::from(&managed_item.path);
    assert!(managed_path.exists());
    assert!(external.exists());

    // Clear temporary items
    state.clear_temporary().unwrap();
    assert!(!managed_path.exists(), "managed file deleted");
    assert!(external.exists(), "external source object preserved");
    assert_eq!(state.snapshot().unwrap().batches.len(), 0);
}

#[test]
fn remove_item_cleans_only_managed_occurrence_leaving_same_name_external_intact() {
    let directory = TestDirectory::new("remove-mixed-ownership");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let external = directory.source_file();
    state.accept_drop(vec![Some(external.to_str().unwrap().to_owned())]).unwrap();
    let managed = state.accept_text("managed text for remove".into()).unwrap();
    let managed_item = &managed.snapshot.unwrap().batches[0].items[0];
    let managed_path = std::path::PathBuf::from(&managed_item.path);
    assert!(managed_path.exists());
    assert!(external.exists());

    state.remove_item(&managed_item.id).unwrap();
    assert!(!managed_path.exists(), "managed file deleted");
    assert!(external.exists(), "external source object preserved");
    assert_eq!(state.snapshot().unwrap().batches.len(), 1);
}

#[test]
fn image_group_acceptance_ordered_and_copy_lifecycle() {
    let directory = TestDirectory::new("image-group-copy-lifecycle");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "11111111111111111111111111111111";
    let id2 = "22222222222222222222222222222222";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);

    // Minimal 1x1 valid PNG bytes
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    store.stage_bytes(&st1, png_bytes).unwrap();
    store.stage_bytes(&st2, png_bytes).unwrap();

    let candidates = vec![
        (st1.clone(), "first.png".to_string()),
        (st2.clone(), "second.png".to_string()),
    ];
    let res = state.accept_image_files(candidates, 1).unwrap();
    assert_eq!(res.skipped, 1);
    assert_eq!(res.message, Some("Added 2 images; skipped 1.".to_string()));
    let snap = res.snapshot.unwrap();
    assert_eq!(snap.batches.len(), 1);
    let batch = &snap.batches[0];
    assert_eq!(batch.items.len(), 2);
    assert_eq!(batch.items[0].name, "first.png");
    assert_eq!(batch.items[1].name, "second.png");

    let path1 = PathBuf::from(&batch.items[0].path);
    let path2 = PathBuf::from(&batch.items[1].path);
    assert!(path1.exists());
    assert!(path2.exists());

    // Pin first item, leave second temporary
    state.set_item_pinned(&batch.items[0].id, true).unwrap();

    // Complete Copy drag for second item (temporary): consumed & cleaned
    let prep2 = state.prepare_item_drag(&batch.id, std::slice::from_ref(&batch.items[1].id)).unwrap();
    let transfer2 = state.begin_item_drag(prep2.token.as_deref().unwrap(), Some(&batch.id)).unwrap();
    state.complete_item_drag(&transfer2, DragOutcome::Copy).unwrap();
    assert!(!path2.exists(), "temporary image deleted on Copy completion");

    // Complete Copy drag for first item (pinned): retained
    let prep1 = state.prepare_item_drag(&batch.id, std::slice::from_ref(&batch.items[0].id)).unwrap();
    let transfer1 = state.begin_item_drag(prep1.token.as_deref().unwrap(), Some(&batch.id)).unwrap();
    let retained = state.complete_item_drag(&transfer1, DragOutcome::Copy).unwrap();
    assert_eq!(retained.batches[0].items.len(), 1);
    assert!(path1.exists(), "pinned image retained after Copy");

    // Explicit removal removes pinned image file
    state.remove_item(&batch.items[0].id).unwrap();
    assert!(!path1.exists(), "explicit removal cleans pinned image file");
}

#[test]
fn image_group_startup_reconciliation_restores_complete_ready_batch() {
    let directory = TestDirectory::new("image-group-reconciliation");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "33333333333333333333333333333333";
    let id2 = "44444444444444444444444444444444";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let fin1 = crate::shelf::managed_files::ManagedStore::final_rel_path(id1, "a.png");
    let fin2 = crate::shelf::managed_files::ManagedStore::final_rel_path(id2, "b.png");

    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    // Item 1 was promoted before crash
    let full_fin1 = store.full_path(&fin1).unwrap();
    fs::create_dir_all(full_fin1.parent().unwrap()).unwrap();
    fs::write(&full_fin1, png_bytes).unwrap();

    // Item 2 was still in staging before crash
    store.stage_bytes(&st2, png_bytes).unwrap();

    // Seed SQLite as 'staged' for both items in one batch
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_img', 100, 0)", []).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_img', 0, ?2, 'a.png', 0, 0)",
            [id1, full_fin1.to_str().unwrap()],
        ).unwrap();
        let full_fin2 = store.full_path(&fin2).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_img', 1, ?2, 'b.png', 0, 0)",
            [id2, full_fin2.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id1, &fin1, &st1],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id2, &fin2, &st2],
        ).unwrap();
    }
    drop(state);

    // Reopen database: startup reconciliation must promote both and mark Ready atomically
    let reopened = ShelfState::open(&database).unwrap();
    let snap = reopened.snapshot().unwrap();
    assert_eq!(snap.batches.len(), 1, "reconciled image batch becomes visible");
    assert_eq!(snap.batches[0].items.len(), 2);
    assert_eq!(snap.batches[0].items[0].id, id1);
    assert_eq!(snap.batches[0].items[1].id, id2);
}

#[test]
fn image_group_startup_reconciliation_rolls_back_if_any_sibling_corrupt() {
    let directory = TestDirectory::new("image-group-sibling-fail");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "55555555555555555555555555555555";
    let id2 = "66666666666666666666666666666666";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let fin1 = crate::shelf::managed_files::ManagedStore::final_rel_path(id1, "ok.png");
    let fin2 = crate::shelf::managed_files::ManagedStore::final_rel_path(id2, "bad.png");

    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    // Item 1 has valid PNG in staging
    store.stage_bytes(&st1, png_bytes).unwrap();
    // Item 2 has corrupt bytes in staging
    store.stage_bytes(&st2, b"not a png").unwrap();

    // Seed SQLite as 'staged' for both items in one batch
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_fail', 100, 0)", []).unwrap();
        let full_fin1 = store.full_path(&fin1).unwrap();
        let full_fin2 = store.full_path(&fin2).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_fail', 0, ?2, 'ok.png', 0, 0)",
            [id1, full_fin1.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_fail', 1, ?2, 'bad.png', 0, 0)",
            [id2, full_fin2.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id1, &fin1, &st1],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id2, &fin2, &st2],
        ).unwrap();
    }
    drop(state);

    // Reopen database: sibling failure means NO batch is published, all are discarded
    let reopened = ShelfState::open(&database).unwrap();
    let snap = reopened.snapshot().unwrap();
    assert_eq!(snap.batches.len(), 0, "corrupt sibling discards entire batch");
}

#[test]
fn image_group_insert_staged_transaction_failure_cleans_up_staging_files() {
    let directory = TestDirectory::new("image-group-tx-fail");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "11111111111111111111111111111111";
    let id2 = "22222222222222222222222222222222";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let path1 = store.stage_bytes(&st1, png_bytes).unwrap();
    let path2 = store.stage_bytes(&st2, png_bytes).unwrap();
    assert!(path1.exists());
    assert!(path2.exists());

    // Inject trigger to fail the staged transaction on insert into shelf_batches
    state.store().unwrap().connection.execute_batch(
        "CREATE TEMP TRIGGER abort_batch_insert BEFORE INSERT ON shelf_batches BEGIN SELECT RAISE(ABORT, 'staged tx blocked'); END;"
    ).unwrap();

    let candidates = vec![
        (st1.clone(), "img1.png".to_string()),
        (st2.clone(), "img2.png".to_string()),
    ];
    let res = state.accept_image_files(candidates, 0);
    assert!(res.is_err());

    // Staging files must be cleaned up immediately, and no shelf batches exist
    assert!(!path1.exists(), "staging file 1 deleted on tx failure");
    assert!(!path2.exists(), "staging file 2 deleted on tx failure");
    let snap = state.snapshot().unwrap();
    assert!(snap.batches.is_empty());
}

#[test]
fn image_group_insert_sibling_promotion_failure_compensates_whole_batch() {
    let directory = TestDirectory::new("image-group-promo-fail");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "33333333333333333333333333333333";
    let id2 = "44444444444444444444444444444444";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let fin1 = crate::shelf::managed_files::ManagedStore::final_rel_path(id1, "first.png");
    let fin2 = crate::shelf::managed_files::ManagedStore::final_rel_path(id2, "second.png");
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let path1 = store.stage_bytes(&st1, png_bytes).unwrap();
    let _path2 = store.stage_bytes(&st2, png_bytes).unwrap();

    // Pre-create destination directory of item 2 as a file to force fs::create_dir_all failure on second promote
    let full_fin2 = store.full_path(&fin2).unwrap();
    let conflict_parent = full_fin2.parent().unwrap();
    fs::write(conflict_parent, b"blocking file").unwrap();

    let candidates = vec![
        (st1.clone(), "first.png".to_string()),
        (st2.clone(), "second.png".to_string()),
    ];
    let res = state.accept_image_files(candidates, 0);
    assert!(res.is_err());

    // Sibling failure must compensate entire batch: no shelf items/batches, item1 final cleaned up
    let full_fin1 = store.full_path(&fin1).unwrap();
    assert!(!full_fin1.exists(), "item 1 promoted file must be compensated");
    assert!(!path1.exists(), "item 1 staging cleaned up");
    let snap = state.snapshot().unwrap();
    assert!(snap.batches.is_empty(), "no partial batch visible");
}

#[test]
fn image_group_insert_ready_transaction_failure_cleans_up_and_rolls_back() {
    let directory = TestDirectory::new("image-group-ready-fail");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "77777777777777777777777777777777";
    let id2 = "88888888888888888888888888888888";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let fin1 = crate::shelf::managed_files::ManagedStore::final_rel_path(id1, "r1.png");
    let fin2 = crate::shelf::managed_files::ManagedStore::final_rel_path(id2, "r2.png");
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    store.stage_bytes(&st1, png_bytes).unwrap();
    store.stage_bytes(&st2, png_bytes).unwrap();

    // Block transition to 'ready'
    state.store().unwrap().connection.execute_batch(
        "CREATE TEMP TRIGGER block_ready BEFORE UPDATE OF state ON managed_files WHEN NEW.state = 'ready' BEGIN SELECT RAISE(ABORT, 'ready blocked'); END;"
    ).unwrap();

    let candidates = vec![
        (st1.clone(), "r1.png".to_string()),
        (st2.clone(), "r2.png".to_string()),
    ];
    let res = state.accept_image_files(candidates, 0);
    assert!(res.is_err());

    // No items or batches visible, promoted files deleted
    let full_fin1 = store.full_path(&fin1).unwrap();
    let full_fin2 = store.full_path(&fin2).unwrap();
    assert!(!full_fin1.exists(), "item 1 promoted file deleted");
    assert!(!full_fin2.exists(), "item 2 promoted file deleted");
    let snap = state.snapshot().unwrap();
    assert!(snap.batches.is_empty(), "no partial ready batch visible");
}

#[test]
fn startup_reconciliation_mixed_staged_promoted_preserves_batch() {
    let directory = TestDirectory::new("reconcile-mixed-staged-promoted");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let id2 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let st1 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id1);
    let st2 = crate::shelf::managed_files::ManagedStore::staging_rel_path(id2);
    let fin1 = crate::shelf::managed_files::ManagedStore::final_rel_path(id1, "mix1.png");
    let fin2 = crate::shelf::managed_files::ManagedStore::final_rel_path(id2, "mix2.png");
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    // Sibling 1 already promoted to final path
    let full_fin1 = store.full_path(&fin1).unwrap();
    if let Some(p) = full_fin1.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&full_fin1, png_bytes).unwrap();

    // Sibling 2 still in staging
    store.stage_bytes(&st2, png_bytes).unwrap();

    // Seed both as 'staged' in DB
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_mix', 100, 0)", []).unwrap();
        let full_fin2 = store.full_path(&fin2).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_mix', 0, ?2, 'mix1.png', 0, 0)",
            [id1, full_fin1.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_mix', 1, ?2, 'mix2.png', 0, 0)",
            [id2, full_fin2.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id1, &fin1, &st1],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id2, &fin2, &st2],
        ).unwrap();
    }
    drop(state);

    // Startup reconciliation reconciles both into a complete ready batch
    let reopened = ShelfState::open(&database).unwrap();
    let snap = reopened.snapshot().unwrap();
    assert_eq!(snap.batches.len(), 1, "reconciled mixed batch is visible");
    assert_eq!(snap.batches[0].items.len(), 2);
    assert_eq!(snap.batches[0].items[0].id, id1);
    assert_eq!(snap.batches[0].items[1].id, id2);
    let full_fin2 = store.full_path(&fin2).unwrap();
    assert!(full_fin2.exists(), "sibling 2 promoted to final path");
}

#[test]
fn unavailable_staging_file_io_error_fails_startup_safely_without_discard() {
    let directory = TestDirectory::new("reconcile-unavailable-io");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id = "cccccccccccccccccccccccccccccccc";
    let st = crate::shelf::managed_files::ManagedStore::staging_rel_path(id);
    let fin = crate::shelf::managed_files::ManagedStore::final_rel_path(id, "locked.png");
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let full_st = store.stage_bytes(&st, png_bytes).unwrap();

    // Seed staged item
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b_lock', 100, 0)", []).unwrap();
        let full_fin = store.full_path(&fin).unwrap();
        conn.execute(
            "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
             VALUES (?1, 'b_lock', 0, ?2, 'locked.png', 0, 0)",
            [id, full_fin.to_str().unwrap()],
        ).unwrap();
        conn.execute(
            "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
             VALUES (?1, ?2, 'staged', ?3)",
            [id, &fin, &st],
        ).unwrap();
    }
    drop(state);

    // Lock staging file exclusively with Windows sharing mode 0 (no sharing)
    use std::os::windows::fs::OpenOptionsExt;
    let _lock_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&full_st)
        .unwrap();

    // Reopen database: reading staging file should fail with I/O sharing violation, NOT treated as invalid content!
    // Must return Err rather than silently discarding/deleting the file.
    let reopen_res = ShelfState::open(&database);
    assert!(reopen_res.is_err(), "startup must fail on inaccessible I/O error");

    // File must NOT be deleted or discarded
    drop(_lock_handle);
    assert!(full_st.exists(), "staged file preserved across temporary I/O lock");
}

#[test]
fn pending_cleanup_queue_persists_across_restart_and_unowned_prefix_is_not_deleted() {
    let directory = TestDirectory::new("cleanup-queue-persistence");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    // Create a managed file and an external file in the same directory structure or profile
    let id = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    let fin = crate::shelf::managed_files::ManagedStore::final_rel_path(id, "test.png");
    let full_fin = store.full_path(&fin).unwrap();
    if let Some(p) = full_fin.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(&full_fin, b"test").unwrap();

    // Insert into pending_managed_file_deletions manually
    {
        let store_guard = state.store().unwrap();
        let conn = &store_guard.connection;
        conn.execute("INSERT INTO pending_managed_file_deletions (relative_path) VALUES (?1)", [&fin]).unwrap();
    }

    // Also create an external unowned file that shares the directory prefix or name
    let external_file = directory.path().join("managed").join("external.txt");
    fs::write(&external_file, b"unowned external file").unwrap();

    // Lock the managed file so deletion cannot succeed immediately
    use std::os::windows::fs::OpenOptionsExt;
    let lock_handle = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&full_fin)
        .unwrap();

    drop(state);

    // Reopen database: pending deletion was attempted and failed due to lock, so it must remain in pending queue!
    let reopened = ShelfState::open(&database).unwrap();
    let pending_count: i64 = reopened.store().unwrap().connection.query_row(
        "SELECT count(*) FROM pending_managed_file_deletions",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(pending_count, 1, "pending deletion must survive failed deletion on startup");
    assert!(external_file.exists(), "unowned file never deleted by prefix or location");

    // Release lock
    drop(lock_handle);
    drop(reopened);

    // Reopen database again: pending deletion should now succeed and row removed
    let reopened2 = ShelfState::open(&database).unwrap();
    let pending_count2: i64 = reopened2.store().unwrap().connection.query_row(
        "SELECT count(*) FROM pending_managed_file_deletions",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(pending_count2, 0, "pending deletion completed successfully once lock released");
    assert!(!full_fin.exists(), "managed file deleted");
    assert!(external_file.exists(), "unowned external file still untouched");
}
