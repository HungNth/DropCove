//! Tauri command wrappers for presentation surfaces: confirmations, About, Drop Shelf resize/move, Manage Items, and Item Actions.

use crate::shelf::{ShelfSnapshot, ShelfState};
use tauri::State;

#[tauri::command]
pub(crate) async fn request_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    let caller = window.label().to_owned();
    crate::run_blocking(move || crate::surfaces::clear_confirmation::request(&app, &caller)).await
}

#[tauri::command]
pub(crate) async fn confirm_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<ShelfSnapshot, String> {
    crate::surfaces::clear_confirmation::validate_confirmation_action_caller(window.label())?;
    crate::run_blocking(move || crate::surfaces::clear_confirmation::confirm(&app)).await
}

#[tauri::command]
pub(crate) async fn cancel_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    crate::surfaces::clear_confirmation::validate_confirmation_action_caller(window.label())?;
    crate::run_blocking(move || crate::surfaces::clear_confirmation::cancel(&app)).await
}

#[tauri::command]
pub(crate) fn get_drop_shelf_clear_temporary_state(app: tauri::AppHandle) -> bool {
    crate::surfaces::clear_confirmation::is_modal_active(&app)
}

#[tauri::command]
pub(crate) fn get_about_metadata(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<crate::surfaces::about::AboutMetadata, String> {
    if window.label() != crate::surfaces::about::ABOUT_WINDOW_LABEL {
        return Err("About metadata is only available in About".to_owned());
    }
    Ok(crate::surfaces::about::get_metadata(&app))
}

#[tauri::command]
pub(crate) async fn open_about(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if window.label() != "main" {
        return Err("About can only be requested from the Drop Shelf".to_owned());
    }
    crate::run_blocking(move || crate::surfaces::about::open(&app, crate::surfaces::placement::InvocationContext::DropShelf)).await
}

#[tauri::command]
pub(crate) async fn start_shelf_resize(
    app: tauri::AppHandle,
    edge: crate::surfaces::shelf_resize::ResizeEdge,
) -> Result<(), String> {
    crate::run_blocking(move || crate::surfaces::shelf_resize::start(&app, edge)).await
}

#[tauri::command]
pub(crate) fn get_shelf_resizing(app: tauri::AppHandle) -> Result<bool, String> {
    crate::surfaces::shelf_resize::is_resizing(&app)
}

#[tauri::command]
pub(crate) async fn prepare_shelf_move(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("prepare_shelf_move is only permitted from the main Drop Shelf window".to_owned());
    }
    crate::run_blocking(move || crate::surfaces::shelf_resize::prepare_move(&app)).await
}

#[tauri::command]
pub(crate) async fn open_manage_items(
    batch_id: String,
    anchor: crate::surfaces::manage_items::AnchorRect,
    keyboard: bool,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<(), String> {
    let origin = window.label().to_owned();
    let state = state.inner().clone();
    crate::run_blocking(move || {
        if crate::surfaces::manage_items::active_batch(&app)?.as_deref() == Some(&batch_id) {
            return crate::surfaces::manage_items::close_explicit(&app);
        }
        let batch = state
            .management_batch(&batch_id)
            .map_err(|error| error.to_string())?;
        if batch.items.len() < 2 {
            return Err("Manage Items requires a multi-item batch".to_owned());
        }
        crate::surfaces::manage_items::open(&app, &batch, anchor, keyboard, &origin)?;
        crate::surfaces::manage_items::synchronize(&app)
    })
    .await
}

#[tauri::command]
pub(crate) async fn get_manage_items(
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<Option<crate::shelf::ShelfBatch>, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let Some(batch_id) = crate::surfaces::manage_items::active_batch(&app)? else {
            return Ok(None);
        };
        match state.management_batch(&batch_id) {
            Ok(batch) => Ok(Some(batch)),
            Err(crate::shelf::ShelfError::BatchNotFound(_)) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command]
pub(crate) async fn get_manage_items_keyboard(app: tauri::AppHandle) -> Result<bool, String> {
    crate::run_blocking(move || crate::surfaces::manage_items::is_keyboard_activated(&app)).await
}

#[tauri::command]
pub(crate) async fn get_manage_items_origin(app: tauri::AppHandle) -> Result<Option<String>, String> {
    crate::surfaces::manage_items::origin(&app)
}

#[tauri::command]
pub(crate) async fn close_manage_items(app: tauri::AppHandle) -> Result<(), String> {
    crate::run_blocking(move || crate::surfaces::manage_items::close_explicit(&app)).await
}

#[tauri::command]
pub(crate) fn cancel_selection_removal(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    crate::surfaces::selection_confirmation::cancel(&app, window.label())
}

#[cfg(windows)]
#[tauri::command]
pub(crate) async fn perform_item_action(
    item_id: String,
    action: crate::shelf::ItemActionKind,
    state: State<'_, ShelfState>,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    let state = state.inner().clone();
    let owner = window.hwnd().map_err(|error| error.to_string())?.0 as usize;
    crate::run_blocking(move || {
        let resolved = state
            .resolve_item_action(&item_id, action)
            .map_err(|error| error.to_string())?;
        match resolved {
            crate::shelf::ResolvedItemAction::ShowInExplorer { path, is_folder } => {
                crate::surfaces::item_actions::show_in_explorer(&path, is_folder)
            }
            crate::shelf::ResolvedItemAction::CopyPath(path) => {
                crate::surfaces::item_actions::write_unicode_clipboard(windows::Win32::Foundation::HWND(owner as *mut _), &path)
            }
            crate::shelf::ResolvedItemAction::CopyText(text) => {
                crate::surfaces::item_actions::write_unicode_clipboard(windows::Win32::Foundation::HWND(owner as *mut _), &text)
            }
        }
    })
    .await
}
