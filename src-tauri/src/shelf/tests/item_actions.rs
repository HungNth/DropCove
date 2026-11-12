use super::TestDirectory;
use crate::shelf::*;
use std::fs;

#[test]
fn item_action_resolution_and_classification_safety() {
    use crate::shelf::{ItemActionKind, ItemOwnership, ManagedContentKind, ResolvedItemAction};
    let directory = TestDirectory::new("item-action-classification");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    let external_parent = directory.path().join("sources").join("managed");
    fs::create_dir_all(&external_parent).unwrap();
    let external_file = external_parent.join("external.txt");
    fs::write(&external_file, b"external content").unwrap();
    let ext_drop = state.accept_drop(vec![Some(external_file.to_str().unwrap().to_owned())]).unwrap();
    let ext_item = &ext_drop.snapshot.as_ref().unwrap().batches[0].items[0];

    assert_eq!(ext_item.ownership, ItemOwnership::External);
    assert_eq!(ext_item.managed_content_kind, None);

    let txt_drop = state.accept_text("managed text payload".into()).unwrap();
    let txt_item = &txt_drop.snapshot.as_ref().unwrap().batches[0].items[0];

    assert_eq!(txt_item.ownership, ItemOwnership::Managed);
    assert_eq!(txt_item.managed_content_kind, Some(ManagedContentKind::Text));

    let store = state.image_staging_store().unwrap();
    let img_bytes = include_bytes!("../../fixtures/website-image.avif");
    let img_rel = format!("staging/{}.tmp", "a".repeat(32));
    store.stage_bytes(&img_rel, img_bytes).unwrap();
    let img_drop = state.accept_image_files(vec![(img_rel, "photo.png".into())], 0).unwrap();
    let img_item = &img_drop.snapshot.as_ref().unwrap().batches[0].items[0];

    assert_eq!(img_item.ownership, ItemOwnership::Managed);
    assert_eq!(img_item.managed_content_kind, Some(ManagedContentKind::Image));

    // Action resolution for External Reference
    let res_show = state.resolve_item_action(&ext_item.id, ItemActionKind::ShowInExplorer).unwrap();
    assert_eq!(res_show, ResolvedItemAction::ShowInExplorer {
        path: external_file.to_str().unwrap().to_owned(),
        is_folder: false,
    });
    let res_path = state.resolve_item_action(&ext_item.id, ItemActionKind::CopyPath).unwrap();
    assert_eq!(res_path, ResolvedItemAction::CopyPath(external_file.to_str().unwrap().to_owned()));
    // External Reference rejects CopyText
    assert!(state.resolve_item_action(&ext_item.id, ItemActionKind::CopyText).is_err());

    // Action resolution for Managed Text
    let res_text = state.resolve_item_action(&txt_item.id, ItemActionKind::CopyText).unwrap();
    assert_eq!(res_text, ResolvedItemAction::CopyText("managed text payload".into()));
    // Managed Text rejects ShowInExplorer and CopyPath
    assert!(state.resolve_item_action(&txt_item.id, ItemActionKind::ShowInExplorer).is_err());
    assert!(state.resolve_item_action(&txt_item.id, ItemActionKind::CopyPath).is_err());

    // Managed Image rejects all actions
    assert!(state.resolve_item_action(&img_item.id, ItemActionKind::ShowInExplorer).is_err());
    assert!(state.resolve_item_action(&img_item.id, ItemActionKind::CopyPath).is_err());
    assert!(state.resolve_item_action(&img_item.id, ItemActionKind::CopyText).is_err());

    // Stale or missing item ID rejects
    assert!(state.resolve_item_action("nonexistent-item-id", ItemActionKind::CopyPath).is_err());
}

#[test]
fn item_action_read_only_and_cached_availability_safety() {
    use crate::shelf::{Availability, ItemActionKind, ResolvedItemAction};
    let directory = TestDirectory::new("item-action-read-only");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // External reference
    let ext_file = directory.path().join("external_readonly.txt");
    fs::write(&ext_file, b"content").unwrap();
    let ext_drop = state.accept_drop(vec![Some(ext_file.to_str().unwrap().to_owned())]).unwrap();
    let ext_item_id = ext_drop.snapshot.as_ref().unwrap().batches[0].items[0].id.clone();

    // Managed text
    let txt_drop = state.accept_text("read-only managed text".into()).unwrap();
    let txt_item_id = txt_drop.snapshot.as_ref().unwrap().batches[0].items[0].id.clone();
    let txt_path_on_disk = txt_drop.snapshot.as_ref().unwrap().batches[0].items[0].path.clone();

    // Capture snapshot before actions
    let snap_before = state.snapshot().unwrap();
    let snap_before_val = serde_json::to_value(&snap_before).unwrap();

    // CopyPath
    let res = state.resolve_item_action(&ext_item_id, ItemActionKind::CopyPath).unwrap();
    assert_eq!(res, ResolvedItemAction::CopyPath(ext_file.to_str().unwrap().to_owned()));

    // ShowInExplorer
    let res = state.resolve_item_action(&ext_item_id, ItemActionKind::ShowInExplorer).unwrap();
    assert_eq!(res, ResolvedItemAction::ShowInExplorer {
        path: ext_file.to_str().unwrap().to_owned(),
        is_folder: false,
    });

    // CopyText
    let res = state.resolve_item_action(&txt_item_id, ItemActionKind::CopyText).unwrap();
    assert_eq!(res, ResolvedItemAction::CopyText("read-only managed text".into()));

    // Snapshot after actions must be completely identical (no mutation, no consumption, no pin change)
    let snap_after = state.snapshot().unwrap();
    let snap_after_val = serde_json::to_value(&snap_after).unwrap();
    assert_eq!(snap_before_val, snap_after_val);

    let text_batch_id = &txt_drop.snapshot.as_ref().unwrap().batches[0].id;
    for cached in [Availability::Missing, Availability::Unavailable] {
        let projection = state.management_batch_with(text_batch_id, |_| cached).unwrap();
        assert_eq!(projection.items[0].availability, cached);
        let before = serde_json::to_value(state.snapshot().unwrap()).unwrap();
        assert_eq!(
            state.resolve_item_action(&txt_item_id, ItemActionKind::CopyText).unwrap(),
            ResolvedItemAction::CopyText("read-only managed text".into()),
        );
        assert_eq!(serde_json::to_value(state.snapshot().unwrap()).unwrap(), before);
    }
    let cached_snapshot = serde_json::to_value(state.snapshot().unwrap()).unwrap();

    // Now test live read when cached availability differs:
    // 1. Delete the managed text file on disk -> live read fails without mutating shelf
    fs::remove_file(&txt_path_on_disk).unwrap();
    let err = state.resolve_item_action(&txt_item_id, ItemActionKind::CopyText).unwrap_err();
    assert!(!err.to_string().contains(&txt_path_on_disk)); // ensure no private path leakage
    let snap_after_fail = state.snapshot().unwrap();
    assert_eq!(serde_json::to_value(&snap_after_fail).unwrap(), cached_snapshot);

    // 2. Restore file bytes -> live read succeeds immediately without any projection refresh
    fs::write(&txt_path_on_disk, b"restored bytes").unwrap();
    let res = state.resolve_item_action(&txt_item_id, ItemActionKind::CopyText).unwrap();
    assert_eq!(res, ResolvedItemAction::CopyText("restored bytes".into()));
}

#[test]
fn item_action_rejects_staged_managed_file() {
    use crate::shelf::ItemActionKind;
    let directory = TestDirectory::new("item-action-staged");
    let database = directory.path().join("shelf.db");
    let state = ShelfState::open(&database).unwrap();

    // Directly insert an uncommitted staged record into DB
    let store = state.store().unwrap();
    let conn = &store.connection;
    conn.execute("INSERT INTO shelf_batches (id, created_at, position) VALUES ('b1', 100, 0)", []).unwrap();
    conn.execute(
        "INSERT INTO shelf_items (id, batch_id, position, path, name, is_folder, is_pinned)
         VALUES ('item_staged', 'b1', 0, 'dummy', 'dummy', 0, 0)",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO managed_files (item_id, relative_path, state, staging_path)
         VALUES ('item_staged', 'managed/item_staged/file.txt', 'staged', 'staging/item_staged.tmp')",
        [],
    ).unwrap();
    drop(store);

    // Staged record must be rejected for all actions
    assert!(state.resolve_item_action("item_staged", ItemActionKind::CopyText).is_err());
    assert!(state.resolve_item_action("item_staged", ItemActionKind::CopyPath).is_err());
    assert!(state.resolve_item_action("item_staged", ItemActionKind::ShowInExplorer).is_err());
}
