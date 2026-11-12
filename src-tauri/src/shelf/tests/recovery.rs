use super::TestDirectory;
use crate::shelf::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn invalid_database_is_refused_without_modification() {
    let directory = TestDirectory::new("integrity");
    let database = directory.path().join("shelf.db");
    let invalid_bytes = b"not a sqlite database";
    fs::write(&database, invalid_bytes).expect("invalid database should be written");

    assert!(ShelfState::open(&database).is_err());
    assert_eq!(
        fs::read(database).expect("database should remain readable"),
        invalid_bytes
    );
}

#[test]
fn corrupt_database_with_wal_and_shm_is_preserved_byte_for_byte_and_recovers_empty_shelf() {
    let directory = TestDirectory::new("recovery-corrupt-all");
    let database = directory.path().join("shelf.db");
    let wal = directory.path().join("shelf.db-wal");
    let shm = directory.path().join("shelf.db-shm");

    let corrupt_db_bytes = b"corrupted sqlite database header and content";
    let corrupt_wal_bytes = b"corrupted wal content";
    let corrupt_shm_bytes = b"corrupted shm content";

    fs::write(&database, corrupt_db_bytes).unwrap();
    fs::write(&wal, corrupt_wal_bytes).unwrap();
    fs::write(&shm, corrupt_shm_bytes).unwrap();

    let (state, backup_dir) =
        ShelfState::open_or_recover(&database).expect("recovery should succeed");
    let backup_dir = backup_dir.expect("backup directory should be returned");

    assert!(backup_dir.exists(), "backup directory must exist");
    assert_eq!(
        backup_dir.parent().unwrap(),
        directory.path().join("recovery"),
        "backup directory must live under profile recovery area"
    );
    let dir_name = backup_dir.file_name().unwrap().to_string_lossy();
    assert!(
        dir_name.starts_with("shelf-corrupt-"),
        "backup directory must start with shelf-corrupt-, got {dir_name}"
    );

    assert_eq!(
        fs::read(backup_dir.join("shelf.db")).unwrap(),
        corrupt_db_bytes
    );
    assert_eq!(
        fs::read(backup_dir.join("shelf.db-wal")).unwrap(),
        corrupt_wal_bytes
    );
    assert_eq!(
        fs::read(backup_dir.join("shelf.db-shm")).unwrap(),
        corrupt_shm_bytes
    );

    if wal.exists() {
        assert_ne!(
            fs::read(&wal).unwrap(),
            corrupt_wal_bytes,
            "active wal must not contain original corrupt bytes"
        );
    }
    if shm.exists() {
        assert_ne!(
            fs::read(&shm).unwrap(),
            corrupt_shm_bytes,
            "active shm must not contain original corrupt bytes"
        );
    }

    let snapshot = state
        .snapshot()
        .expect("recovered shelf snapshot should load");
    assert!(snapshot.batches.is_empty(), "recovered shelf must start empty");
}

#[test]
fn recovery_accepts_missing_sidecars_and_preserves_only_existing() {
    // Subcase A: No sidecars
    {
        let directory = TestDirectory::new("recovery-no-sidecars");
        let database = directory.path().join("shelf.db");
        fs::write(&database, b"corrupted db without sidecars").unwrap();

        let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
        let backup = backup_dir.unwrap();
        assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
        assert_eq!(
            fs::read(backup.join("shelf.db")).unwrap(),
            b"corrupted db without sidecars"
        );
        assert!(!backup.join("shelf.db-wal").exists());
        assert!(!backup.join("shelf.db-shm").exists());
        assert!(state.snapshot().unwrap().batches.is_empty());
    }

    // Subcase B: WAL only
    {
        let directory = TestDirectory::new("recovery-wal-only");
        let database = directory.path().join("shelf.db");
        let wal = directory.path().join("shelf.db-wal");
        fs::write(&database, b"corrupted db").unwrap();
        fs::write(&wal, b"wal only").unwrap();

        let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
        let backup = backup_dir.unwrap();
        assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
        assert_eq!(fs::read(backup.join("shelf.db")).unwrap(), b"corrupted db");
        assert_eq!(fs::read(backup.join("shelf.db-wal")).unwrap(), b"wal only");
        assert!(!backup.join("shelf.db-shm").exists());
        assert!(state.snapshot().unwrap().batches.is_empty());
    }

    // Subcase C: SHM only
    {
        let directory = TestDirectory::new("recovery-shm-only");
        let database = directory.path().join("shelf.db");
        let shm = directory.path().join("shelf.db-shm");
        fs::write(&database, b"corrupted db").unwrap();
        fs::write(&shm, b"shm only").unwrap();

        let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
        let backup = backup_dir.unwrap();
        assert_eq!(backup.parent().unwrap(), directory.path().join("recovery"));
        assert_eq!(fs::read(backup.join("shelf.db")).unwrap(), b"corrupted db");
        assert_eq!(fs::read(backup.join("shelf.db-shm")).unwrap(), b"shm only");
        assert!(!backup.join("shelf.db-wal").exists());
        assert!(state.snapshot().unwrap().batches.is_empty());
    }
}

#[test]
fn recovery_backup_name_collision_resolves_deterministic_numeric_suffix_under_recovery_area() {
    use std::time::{Duration, UNIX_EPOCH};
    let directory = TestDirectory::new("recovery-collision");
    let database = directory.path().join("shelf.db");
    let fixed_time = UNIX_EPOCH + Duration::from_secs(1776182400); // 2026-04-14T16:00:00Z

    // First corrupt episode
    fs::write(&database, b"first corrupt db").unwrap();
    let (state1, backup1) =
        ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
    let backup1 = backup1.unwrap();
    assert_eq!(
        backup1,
        directory
            .path()
            .join("recovery")
            .join("shelf-corrupt-20260414T160000Z")
    );
    assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
    assert!(state1.snapshot().unwrap().batches.is_empty());
    drop(state1);

    // Second corrupt episode at same timestamp -> must produce -1
    fs::write(&database, b"second corrupt db").unwrap();
    let (state2, backup2) =
        ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
    let backup2 = backup2.unwrap();
    assert_eq!(
        backup2,
        directory
            .path()
            .join("recovery")
            .join("shelf-corrupt-20260414T160000Z-1")
    );
    assert_eq!(fs::read(backup2.join("shelf.db")).unwrap(), b"second corrupt db");
    // Prior backup1 must be completely untouched
    assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
    assert!(state2.snapshot().unwrap().batches.is_empty());
    drop(state2);

    // Third corrupt episode at same timestamp -> must produce -2
    fs::write(&database, b"third corrupt db").unwrap();
    let (state3, backup3) =
        ShelfState::open_or_recover_at(&database, fixed_time).unwrap();
    let backup3 = backup3.unwrap();
    assert_eq!(
        backup3,
        directory
            .path()
            .join("recovery")
            .join("shelf-corrupt-20260414T160000Z-2")
    );
    assert_eq!(fs::read(backup3.join("shelf.db")).unwrap(), b"third corrupt db");
    assert_eq!(fs::read(backup1.join("shelf.db")).unwrap(), b"first corrupt db");
    assert_eq!(fs::read(backup2.join("shelf.db")).unwrap(), b"second corrupt db");
    assert!(state3.snapshot().unwrap().batches.is_empty());
}

#[test]
fn clean_and_missing_database_do_not_create_recovery_backup() {
    let directory = TestDirectory::new("recovery-clean");
    let database = directory.path().join("shelf.db");
    let recovery_area = directory.path().join("recovery");

    // 1. Missing database
    assert!(!database.exists());
    let (state, backup) = ShelfState::open_or_recover(&database).unwrap();
    assert!(backup.is_none(), "missing db must not create recovery backup");
    assert!(!recovery_area.exists(), "recovery directory must not be created");
    assert!(database.exists(), "clean database must be initialized");
    assert!(state.snapshot().unwrap().batches.is_empty());

    // Populate database
    state
        .accept_drop(vec![Some(
            directory.source_file().to_string_lossy().into_owned(),
        )])
        .unwrap();
    drop(state);

    // 2. Existing clean database
    let (reopened, backup) = ShelfState::open_or_recover(&database).unwrap();
    assert!(backup.is_none(), "clean existing db must not create recovery backup");
    assert!(!recovery_area.exists(), "recovery directory must not be created");
    assert_eq!(reopened.snapshot().unwrap().batches.len(), 1);
}

#[test]
fn schema_invalid_database_is_recovered_and_preserves_evidence() {
    let directory = TestDirectory::new("recovery-schema-invalid");
    let database = directory.path().join("shelf.db");

    // Create a sqlite database with wrong columns in shelf_batches
    {
        let conn = rusqlite::Connection::open(&database).unwrap();
        conn.execute_batch(
            "CREATE TABLE shelf_batches (id TEXT PRIMARY KEY, wrong_column INTEGER);",
        )
        .unwrap();
    }
    let original_bytes = fs::read(&database).unwrap();

    let (state, backup) = ShelfState::open_or_recover(&database).unwrap();
    let backup_dir = backup.expect("schema-invalid db must trigger recovery");
    assert_eq!(backup_dir.parent().unwrap(), directory.path().join("recovery"));
    assert_eq!(fs::read(backup_dir.join("shelf.db")).unwrap(), original_bytes);

    // Clean database now has correct schema and starts empty
    assert!(state.snapshot().unwrap().batches.is_empty());
    // Normal operations work on the clean database
    let dropped = state
        .accept_drop(vec![Some(
            directory.source_file().to_string_lossy().into_owned(),
        )])
        .unwrap();
    assert!(dropped.snapshot.is_some());
}

#[test]
fn unrecoverable_filesystem_failure_aborts_without_clean_shelf_or_data_loss() {
    let directory = TestDirectory::new("recovery-fs-fail");
    let database = directory.path().join("shelf.db");
    let wal = directory.path().join("shelf.db-wal");

    let corrupt_db = b"corrupted db content";
    let corrupt_wal = b"corrupted wal content";
    fs::write(&database, corrupt_db).unwrap();
    fs::write(&wal, corrupt_wal).unwrap();

    // Block the recovery directory by creating a regular file named "recovery"
    fs::write(directory.path().join("recovery"), b"blocking file").unwrap();

    let result = ShelfState::open_or_recover(&database);
    assert!(result.is_err(), "must fail fatally when recovery directory cannot be created");

    // Original evidence must remain intact and untouched
    assert_eq!(fs::read(&database).unwrap(), corrupt_db);
    assert_eq!(fs::read(&wal).unwrap(), corrupt_wal);
}

#[test]
fn recovery_partial_move_failure_restores_moved_first_artifact() {
    use std::time::{Duration, UNIX_EPOCH};
    let directory = TestDirectory::new("recovery-partial-move");
    let database = directory.path().join("shelf.db");
    let wal = directory.path().join("shelf.db-wal");
    let fixed_time = UNIX_EPOCH + Duration::from_secs(1776182400);

    let corrupt_db = b"corrupted db content";
    let corrupt_wal = b"corrupted wal content";
    fs::write(&database, corrupt_db).unwrap();
    fs::write(&wal, corrupt_wal).unwrap();
    #[cfg(windows)]
    use std::os::windows::fs::OpenOptionsExt;
    let _wal_lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2) // FILE_SHARE_READ | FILE_SHARE_WRITE (no FILE_SHARE_DELETE)
        .open(&wal)
        .unwrap();

    let result = ShelfState::open_or_recover_at(&database, fixed_time);
    assert!(result.is_err(), "must fail when second artifact move fails");

    drop(_wal_lock);

    // First artifact (database) was moved then restored back to source
    assert_eq!(fs::read(&database).unwrap(), corrupt_db);
    assert_eq!(fs::read(&wal).unwrap(), corrupt_wal);
}

#[test]
fn recovery_warning_delivery_failure_retains_snapshot_warning_until_acknowledged() {
    let directory = TestDirectory::new("recovery-warning-fallback");
    let database = directory.path().join("shelf.db");
    fs::write(&database, b"corrupted db content").unwrap();

    let (state, backup_dir) = ShelfState::open_or_recover(&database).unwrap();
    let backup_path = backup_dir.expect("recovery backup should exist");

    // Simulate notification-area delivery failure at the startup seam
    let mut attempted_message = None;
    state.deliver_recovery_warning(Some(&backup_path), |msg| {
        attempted_message = Some(msg.to_owned());
        Err("notification channel unavailable".into())
    });

    let msg = attempted_message.expect("notifier should be called with warning message");
    assert!(
        msg.contains(&backup_path.display().to_string()),
        "warning must include Recovery Backup path"
    );
    assert!(
        msg.contains("empty Shelf"),
        "warning must identify that Shelf started empty"
    );

    // Warning is exposed on ShelfSnapshot and retained across multiple reads before dismissal
    let first_read = state.snapshot().unwrap();
    assert_eq!(first_read.warning.as_deref(), Some(msg.as_str()));
    let second_read = state.snapshot().unwrap();
    assert_eq!(second_read.warning.as_deref(), Some(msg.as_str()));

    // Acknowledging warning after user dismissal clears it from subsequent snapshots
    state.acknowledge_warning();
    assert_eq!(state.snapshot().unwrap().warning, None);
}

#[test]
fn valid_header_corrupt_btree_with_wal_and_shm_preserves_shm_bytes_without_wal_index_overwrite() {
    let directory = TestDirectory::new("recovery-valid-header-shm");
    let database = directory.path().join("shelf.db");
    let wal = directory.path().join("shelf.db-wal");
    let shm = directory.path().join("shelf.db-shm");

    // Create a real WAL-mode SQLite database and close the connection cleanly
    {
        let state = ShelfState::open(&database).unwrap();
        state
            .accept_drop(vec![Some(
                directory.source_file().to_string_lossy().into_owned(),
            )])
            .unwrap();
        state.close().unwrap();
    }

    // Corrupt the interior B-tree page data while keeping the 16-byte SQLite format 3 header intact
    let mut corrupt_db_bytes = fs::read(&database).unwrap();
    for byte in &mut corrupt_db_bytes[100..200] {
        *byte = 0xFF;
    }
    let corrupt_wal_bytes = b"original WAL evidence";
    let corrupt_shm_bytes = b"original SHM evidence";

    fs::write(&database, &corrupt_db_bytes).unwrap();
    fs::write(&wal, corrupt_wal_bytes).unwrap();
    fs::write(&shm, corrupt_shm_bytes).unwrap();

    let (state, backup_dir) =
        ShelfState::open_or_recover(&database).expect("recovery should succeed");
    let backup_dir = backup_dir.expect("backup directory should be returned");

    assert_eq!(
        fs::read(backup_dir.join("shelf.db")).unwrap(),
        corrupt_db_bytes,
        "preserved shelf.db must match original corrupt bytes"
    );
    assert_eq!(
        fs::read(backup_dir.join("shelf.db-wal")).unwrap(),
        corrupt_wal_bytes,
        "preserved shelf.db-wal must match original WAL evidence"
    );
    assert_eq!(
        fs::read(backup_dir.join("shelf.db-shm")).unwrap(),
        corrupt_shm_bytes,
        "preserved shelf.db-shm must be the exact 21-byte original SHM evidence, not overwritten by 32KB WAL index"
    );
    assert!(state.snapshot().unwrap().batches.is_empty());
}

#[test]
fn managed_file_recovery_preserves_managed_and_staging_roots() {
    let directory = TestDirectory::new("recovery-managed-preservation");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    let text_res = state.accept_text("recovery text bytes".to_string()).unwrap();
    let text_snap = text_res.snapshot.unwrap();
    let text_item = &text_snap.batches[0].items[0];
    let text_path = PathBuf::from(&text_item.path);
    assert!(text_path.exists());

    // Also create a dummy file in staging
    let staging_file = directory.path().join("staging").join("0123456789abcdef0123456789abcdef.tmp");
    fs::write(&staging_file, b"staged bytes").unwrap();

    // Corrupt the database
    drop(state);
    fs::write(&database, b"corrupted shelf db").unwrap();

    // Trigger open_or_recover
    let (reopened, backup_dir_opt) = ShelfState::open_or_recover(&database).unwrap();
    let backup_dir = backup_dir_opt.expect("recovery backup must be created");

    // Active managed root is now clean and empty
    let active_snap = reopened.snapshot().unwrap();
    assert!(active_snap.batches.is_empty(), "clean empty shelf on recovery");
    assert!(!text_path.exists(), "old managed file moved from active store");

    // Backup directory must preserve database, managed root, and staging root
    assert!(backup_dir.join("shelf.db").exists());
    let preserved_managed = backup_dir.join("managed");
    assert!(preserved_managed.exists(), "backup must contain managed root");
    let preserved_staging = backup_dir.join("staging");
    assert!(preserved_staging.exists(), "backup must contain staging root");
    assert!(preserved_staging.join("0123456789abcdef0123456789abcdef.tmp").exists());
}

#[test]
fn image_recovery_backup_preserves_bytes_and_ads() {
    let directory = TestDirectory::new("recovery-image-ads");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();
    let store = state.image_staging_store().unwrap();
    store.init_directories().unwrap();

    let id = "dddddddddddddddddddddddddddddddd";
    let st = crate::shelf::managed_files::ManagedStore::staging_rel_path(id);
    let fin = crate::shelf::managed_files::ManagedStore::final_rel_path(id, "zone.png");
    let png_bytes = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00,
        0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
        0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let _path = store.stage_bytes(&st, png_bytes).unwrap();
    let candidates = vec![(st, "zone.png".to_string())];
    let drop_res = state.accept_image_files(candidates, 0).unwrap();
    let snap = drop_res.snapshot.unwrap();
    assert_eq!(snap.batches.len(), 1);

    let full_fin = store.full_path(&fin).unwrap();
    assert!(full_fin.exists());

    // Write Zone.Identifier ADS to the managed file
    let ads_path = format!("{}:Zone.Identifier", full_fin.display());
    fs::write(&ads_path, b"[ZoneTransfer]\r\nZoneId=3\r\n").unwrap();
    assert_eq!(fs::read(&ads_path).unwrap(), b"[ZoneTransfer]\r\nZoneId=3\r\n");

    // Corrupt the database to force Recovery Backup
    drop(state);
    fs::write(&database, b"corrupt db forcing recovery").unwrap();

    // Reopen via open_or_recover
    let (reopened, backup_dir_opt) = ShelfState::open_or_recover(&database).unwrap();
    let backup_dir = backup_dir_opt.expect("recovery backup created");

    // Active shelf is clean and empty
    assert!(reopened.snapshot().unwrap().batches.is_empty());
    assert!(!full_fin.exists(), "moved out of active managed store");

    // Check preserved file and ADS in backup directory
    let preserved_file = backup_dir.join("managed").join(id).join("zone.png");
    assert!(preserved_file.exists(), "preserved managed file in backup");
    assert_eq!(fs::read(&preserved_file).unwrap(), png_bytes);

    let preserved_ads = format!("{}:Zone.Identifier", preserved_file.display());
    assert_eq!(fs::read(&preserved_ads).unwrap(), b"[ZoneTransfer]\r\nZoneId=3\r\n", "Zone.Identifier ADS preserved in Recovery Backup");
}
