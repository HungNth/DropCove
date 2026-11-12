//! Tauri command wrappers for Shelf state, pinning, removal, and Managed File text.

use super::{ShelfSnapshot, ShelfState};
use tauri::State;

pub(crate) fn publish_shelf(app: &tauri::AppHandle, snapshot: ShelfSnapshot) -> Result<ShelfSnapshot, String> {
    crate::surfaces::clear_confirmation::synchronize(app)?;
    crate::surfaces::manage_items::synchronize(app)?;
    crate::resident::synchronize(app, !snapshot.batches.is_empty())?;
    let _ = crate::resident::update_rail_content(app, snapshot.batches.len());
    if let Some(warning) = snapshot.warning.as_deref() {
        let _ = crate::resident::publish_warning(app, warning);
    }
    use tauri::Emitter;
    let _ = app.emit("shelf-changed", ());
    Ok(snapshot)
}

#[tauri::command]
pub(crate) async fn get_shelf(state: State<'_, ShelfState>) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || state.snapshot()).await
}

#[tauri::command]
pub(crate) fn acknowledge_shelf_warning(state: State<'_, ShelfState>) -> Result<(), String> {
    state.acknowledge_warning();
    Ok(())
}

#[tauri::command]
pub(crate) async fn accept_drop(
    paths: Vec<Option<String>>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<super::DropResult, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let result = state
            .accept_drop(paths)
            .map_err(|error| error.to_string())?;
        if let Some(snapshot) = result.snapshot.as_ref() {
            crate::resident::synchronize(&app, !snapshot.batches.is_empty())?;
            crate::surfaces::manage_items::synchronize(&app)?;
            let _ = crate::resident::update_rail_content(&app, snapshot.batches.len());
            use tauri::Emitter;
            let _ = app.emit("shelf-changed", ());
        }
        Ok::<_, String>(result)
    })
    .await
}

#[tauri::command]
pub(crate) async fn save_size(
    width: f64,
    manual_height: Option<f64>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .save_size(width, manual_height)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn remove_batch(
    batch_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .remove_batch(&batch_id)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn remove_item(
    item_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .remove_item(&item_id)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn clear_temporary(
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state.clear_temporary().map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn set_item_pinned(
    item_id: String,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .set_item_pinned(&item_id, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn set_batch_pinned(
    batch_id: String,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .set_batch_pinned(&batch_id, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn set_batches_pinned(
    batch_ids: Vec<String>,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .set_batches_pinned(&batch_ids, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn set_items_pinned(
    batch_id: String,
    item_ids: Vec<String>,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .set_items_pinned(&batch_id, &item_ids, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn prepare_batches_removal(
    batch_ids: Vec<String>,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<super::RemovalPreview, String> {
    let caller = window.label().to_owned();
    crate::surfaces::selection_confirmation::begin(&app, &caller)?;
    let state = state.inner().clone();
    let result = crate::run_blocking(move || state.prepare_batch_removal(&batch_ids).map_err(|error| error.to_string())).await;
    if result.is_err() { crate::surfaces::selection_confirmation::cancel(&app, &caller)?; }
    result
}

#[tauri::command]
pub(crate) async fn prepare_items_removal(
    batch_id: String,
    item_ids: Vec<String>,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<super::RemovalPreview, String> {
    let caller = window.label().to_owned();
    crate::surfaces::selection_confirmation::begin(&app, &caller)?;
    let state = state.inner().clone();
    let result = crate::run_blocking(move || state.prepare_item_removal(&batch_id, &item_ids).map_err(|error| error.to_string())).await;
    if result.is_err() { crate::surfaces::selection_confirmation::cancel(&app, &caller)?; }
    result
}

#[tauri::command]
pub(crate) async fn remove_items(
    batch_id: String,
    item_ids: Vec<String>,
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let caller = window.label().to_owned();
    let release_app = app.clone();
    let state = state.inner().clone();
    let result = crate::run_blocking(move || {
        let snapshot = state
            .confirm_item_removal(&batch_id, &item_ids, &token)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    }).await;
    crate::surfaces::selection_confirmation::cancel(&release_app, &caller)?;
    result
}

#[tauri::command]
pub(crate) async fn remove_batches(
    batch_ids: Vec<String>,
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let caller = window.label().to_owned();
    let release_app = app.clone();
    let state = state.inner().clone();
    let result = crate::run_blocking(move || {
        let snapshot = state
            .confirm_batch_removal(&batch_ids, &token)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    }).await;
    crate::surfaces::selection_confirmation::cancel(&release_app, &caller)?;
    result
}

#[cfg(windows)]
#[tauri::command]
pub(crate) async fn get_managed_text(
    item_id: String,
    state: State<'_, ShelfState>,
) -> Result<Option<String>, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        state.read_managed_text_content(&item_id).map_err(|error| error.to_string())
    })
    .await
}
