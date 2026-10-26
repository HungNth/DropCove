use std::fmt::Display;

use tauri::{Manager, State};

mod about;
mod clear_confirmation;
mod edge_rail;
mod manage_items;
pub(crate) mod managed_files;
mod resident;
mod resident_tray;
mod settings;
mod settings_store;
mod settings_transaction;
mod shelf;
mod shelf_resize;
mod windows_auxiliary;
#[cfg(windows)]
mod windows_drag;
#[cfg(windows)]
mod windows_drop;
mod website_images;
mod windows_virtual_files;
mod windows_foreground;
mod windows_hotkey;
mod windows_instance;
mod windows_paths;
mod windows_shake;
mod windows_startup;
mod windows_theme;
pub(crate) mod windows_visuals;
use windows_visuals::{get_shelf_visual, get_surface_generation, invalidate_surface_visuals, cancel_shelf_visual};

use shelf::{ShelfSnapshot, ShelfState};

async fn run_blocking<T, E, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    E: Display + Send + 'static,
    F: FnOnce() -> Result<T, E> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| format!("background task failed: {error}"))?
        .map_err(|error| error.to_string())
}

fn publish_shelf(app: &tauri::AppHandle, snapshot: ShelfSnapshot) -> Result<ShelfSnapshot, String> {
    clear_confirmation::synchronize(app)?;
    manage_items::synchronize(app)?;
    resident::synchronize(app, !snapshot.batches.is_empty())?;
    let _ = resident::update_rail_content(app, snapshot.batches.len());
    if let Some(warning) = snapshot.warning.as_deref() {
        let _ = resident::publish_warning(app, warning);
    }
    use tauri::Emitter;
    let _ = app.emit("shelf-changed", ());
    Ok(snapshot)
}

#[tauri::command]
async fn request_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    let caller = window.label().to_owned();
    run_blocking(move || clear_confirmation::request(&app, &caller)).await
}

#[tauri::command]
async fn confirm_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<ShelfSnapshot, String> {
    clear_confirmation::validate_confirmation_action_caller(window.label())?;
    run_blocking(move || clear_confirmation::confirm(&app)).await
}

#[tauri::command]
async fn cancel_drop_shelf_clear_temporary(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    clear_confirmation::validate_confirmation_action_caller(window.label())?;
    run_blocking(move || clear_confirmation::cancel(&app)).await
}

#[tauri::command]
fn get_drop_shelf_clear_temporary_state(app: tauri::AppHandle) -> bool {
    clear_confirmation::is_modal_active(&app)
}

#[tauri::command]
fn get_about_metadata(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<about::AboutMetadata, String> {
    if window.label() != about::ABOUT_WINDOW_LABEL {
        return Err("About metadata is only available in About".to_owned());
    }
    Ok(about::get_metadata(&app))
}

#[tauri::command]
async fn open_about(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if window.label() != "main" {
        return Err("About can only be requested from the Drop Shelf".to_owned());
    }
    run_blocking(move || about::open(&app, windows_auxiliary::InvocationContext::DropShelf)).await
}

#[tauri::command]
async fn open_settings(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Settings can only be requested from the Drop Shelf".to_owned());
    }
    run_blocking(move || settings::open(&app, windows_auxiliary::InvocationContext::DropShelf)).await
}

#[tauri::command]
async fn start_shelf_resize(
    app: tauri::AppHandle,
    edge: shelf_resize::ResizeEdge,
) -> Result<(), String> {
    run_blocking(move || shelf_resize::start(&app, edge)).await
}

#[tauri::command]
fn get_shelf_resizing(app: tauri::AppHandle) -> Result<bool, String> {
    shelf_resize::is_resizing(&app)
}

#[tauri::command]
async fn dismiss_drop_shelf(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("dismiss_drop_shelf is only permitted from the main Drop Shelf window".to_owned());
    }
    run_blocking(move || resident::dismiss_shelf(&app)).await
}

#[tauri::command]
async fn prepare_shelf_move(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("prepare_shelf_move is only permitted from the main Drop Shelf window".to_owned());
    }
    run_blocking(move || shelf_resize::prepare_move(&app)).await
}

#[tauri::command]
async fn open_manage_items(
    batch_id: String,
    anchor: manage_items::AnchorRect,
    keyboard: bool,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<(), String> {
    let origin = window.label().to_owned();
    let state = state.inner().clone();
    run_blocking(move || {
        if manage_items::active_batch(&app)?.as_deref() == Some(&batch_id) {
            return manage_items::close_explicit(&app);
        }
        let batch = state
            .management_batch(&batch_id)
            .map_err(|error| error.to_string())?;
        if batch.items.len() < 2 {
            return Err("Manage Items requires a multi-item batch".to_owned());
        }
        manage_items::open(&app, &batch, anchor, keyboard, &origin)?;
        manage_items::synchronize(&app)
    })
    .await
}

#[tauri::command]
async fn get_manage_items(
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<Option<shelf::ShelfBatch>, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let Some(batch_id) = manage_items::active_batch(&app)? else {
            return Ok(None);
        };
        match state.management_batch(&batch_id) {
            Ok(batch) => Ok(Some(batch)),
            Err(shelf::ShelfError::BatchNotFound(_)) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    })
    .await
}

#[tauri::command]
async fn get_manage_items_keyboard(app: tauri::AppHandle) -> Result<bool, String> {
    run_blocking(move || manage_items::is_keyboard_activated(&app)).await
}

#[tauri::command]
async fn get_manage_items_origin(app: tauri::AppHandle) -> Result<Option<String>, String> {
    manage_items::origin(&app)
}

#[tauri::command]
async fn prepare_item_drag(
    item_ids: Vec<String>,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<shelf::ItemPreparation, String> {
    if window.label() != "manage-items" {
        return Err("prepare_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    run_blocking(move || {
        let active_batch = manage_items::active_batch(&app)?
            .ok_or_else(|| "Manage Items is not active".to_owned())?;
        let preparation = state
            .prepare_item_drag(&active_batch, &item_ids)
            .map_err(|error| error.to_string())?;
        if let Err(error) = publish_shelf(&app, preparation.snapshot.clone()) {
            let mut message = error;
            if let Some(token) = &preparation.token {
                if let Err(cleanup) = state.cancel_item_drag(token) {
                    message.push_str(&format!("; prepared drag cancellation failed: {cleanup}"));
                }
            }
            if let Err(cleanup) = manage_items::synchronize(&app) {
                message.push_str(&format!("; Manage Items synchronization failed: {cleanup}"));
            }
            return Err(message);
        }
        Ok(preparation)
    })
    .await
}

#[tauri::command]
async fn cancel_item_drag(
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    if window.label() != "manage-items" {
        return Err("cancel_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .cancel_item_drag(&token)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[derive(serde::Serialize)]
struct ItemDragResult {
    outcome: shelf::DragOutcome,
    error: Option<String>,
}

#[tauri::command]
async fn close_manage_items(app: tauri::AppHandle) -> Result<(), String> {
    run_blocking(move || manage_items::close_explicit(&app)).await
}

#[tauri::command]
async fn remove_item(
    item_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .remove_item(&item_id)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
async fn clear_temporary(
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state.clear_temporary().map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
async fn get_shelf(state: State<'_, ShelfState>) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || state.snapshot()).await
}

#[tauri::command]
fn acknowledge_shelf_warning(state: State<'_, ShelfState>) -> Result<(), String> {
    state.acknowledge_warning();
    Ok(())
}

#[tauri::command]
async fn accept_drop(
    paths: Vec<Option<String>>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<shelf::DropResult, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let result = state
            .accept_drop(paths)
            .map_err(|error| error.to_string())?;
        if let Some(snapshot) = result.snapshot.as_ref() {
            resident::synchronize(&app, !snapshot.batches.is_empty())?;
            manage_items::synchronize(&app)?;
            let _ = resident::update_rail_content(&app, snapshot.batches.len());
            use tauri::Emitter;
            let _ = app.emit("shelf-changed", ());
        }
        Ok::<_, String>(result)
    })
    .await
}

#[tauri::command]
async fn save_size(
    width: f64,
    manual_height: Option<f64>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .save_size(width, manual_height)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
async fn remove_batch(
    batch_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .remove_batch(&batch_id)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
async fn set_item_pinned(
    item_id: String,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .set_item_pinned(&item_id, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}
#[tauri::command]
async fn set_batch_pinned(
    batch_id: String,
    pinned: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .set_batch_pinned(&batch_id, pinned)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

fn storage_drag_on_window_thread(
    app: tauri::AppHandle,
    operation: impl FnOnce(
            &windows_drag::DragSession,
        ) -> Result<shelf::DragOutcome, windows_drag::StorageDragError>
        + Send
        + 'static,
) -> Result<shelf::DragOutcome, String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let drag_app = app.clone();
    app.run_on_main_thread(move || {
        struct RailDragGuard<'a>(&'a tauri::AppHandle);
        impl Drop for RailDragGuard<'_> {
            fn drop(&mut self) {
                match self
                    .0
                    .state::<resident::ResidentState>()
                    .set_rail_drag_active(false)
                {
                    Ok(Some(remaining)) => {
                        if let Err(error) = edge_rail::arm_rail_timer(self.0, remaining as u32) {
                            let _ = resident::publish_warning(self.0, &error);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let _ = resident::publish_warning(self.0, &error);
                    }
                }
            }
        }
        if let Err(error) = drag_app
            .state::<resident::ResidentState>()
            .set_rail_drag_active(true)
        {
            let _ = sender.send(Err(error));
            return;
        }
        let _guard = RailDragGuard(&drag_app);
        let session = drag_app.state::<windows_drag::DragSession>();
        let result = operation(&session).map_err(|error| error.to_string());
        // A disappearing caller leaves its references retained.
        let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())?
}

#[tauri::command]
async fn prepare_batch_drag(
    batch_ids: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<shelf::BatchPreparation, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        manage_items::close(&app)?;
        let preparation = state
            .prepare_batch_drag(&batch_ids)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, preparation.snapshot.clone())?;
        Ok::<_, String>(preparation)
    })
    .await
}

#[tauri::command]
async fn cancel_batch_drag(
    token: String,
    state: State<'_, ShelfState>,
    app: tauri::AppHandle,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let snapshot = state
            .cancel_batch_drag(&token)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[derive(serde::Serialize)]
struct BatchDragResult {
    outcome: shelf::DragOutcome,
    error: Option<String>,
}

#[tauri::command]
async fn start_batch_drag(
    token: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<BatchDragResult, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        if let Err(error) = manage_items::close(&app) {
            return match state.cancel_batch_drag(&token) {
                Ok(_) => Err(error),
                Err(cleanup) => Err(format!("{error}; prepared drag cancellation failed: {cleanup}")),
            };
        }
        let transfer = state
            .begin_batch_drag(&token)
            .map_err(|error| error.to_string())?;
        let single_item_text = if transfer.participants.len() == 1 {
            state.read_managed_text_content(&transfer.participants[0].id).map_err(|error| error.to_string())?
        } else {
            None
        };
        let native_transfer = std::sync::Arc::clone(&transfer);
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            windows_drag::perform_storage_drag(
                session,
                native_transfer.paths.iter().map(String::as_str),
                single_item_text.as_deref(),
            )
        });
        match outcome {
            Ok(outcome) => {
                let completion = state
                    .complete_batch_drag(&transfer, outcome)
                    .map_err(|error| error.to_string())
                    .and_then(|snapshot| publish_shelf(&app, snapshot));
                Ok(BatchDragResult { outcome, error: completion.err() })
            }
            Err(error) => match manage_items::synchronize(&app) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!("{error}; Manage Items synchronization failed: {cleanup}")),
            },
        }
    })
    .await
}

#[cfg(windows)]
#[tauri::command]
async fn start_item_drag(
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ItemDragResult, String> {
    if window.label() != "manage-items" {
        return Err("start_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    run_blocking(move || {
        let active_batch = manage_items::active_batch(&app)?
            .ok_or_else(|| "Manage Items is not active".to_owned())?;
        let transfer = state
            .begin_item_drag(&token, Some(&active_batch))
            .map_err(|error| error.to_string())?;
        let single_item_text = if transfer.participants.len() == 1 {
            state.read_managed_text_content(&transfer.participants[0].id).map_err(|error| error.to_string())?
        } else {
            None
        };
        let native_transfer = std::sync::Arc::clone(&transfer);
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            windows_drag::perform_storage_drag(
                session,
                native_transfer.paths.iter().map(String::as_str),
                single_item_text.as_deref(),
            )
        });

        match outcome {
            Ok(outcome) => {
                if outcome == shelf::DragOutcome::Canceled {
                    use tauri::Emitter;
                    let _ = app.emit_to("manage-items", "item-drag-canceled", ());
                }
                let completion = state
                    .complete_item_drag(&transfer, outcome)
                    .map_err(|error| error.to_string())
                    .and_then(|snapshot| publish_shelf(&app, snapshot));
                if let Err(sync_error) = manage_items::synchronize(&app) {
                    let error_message = match completion.err() {
                        Some(err) => format!("{err}; Manage Items synchronization failed: {sync_error}"),
                        None => format!("Manage Items synchronization failed: {sync_error}"),
                    };
                    return Ok(ItemDragResult { outcome, error: Some(error_message) });
                }
                Ok(ItemDragResult { outcome, error: completion.err() })
            }
            Err(error) => {
                match manage_items::synchronize(&app) {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(format!("{error}; Manage Items synchronization failed: {cleanup}")),
                }
            }
        }
    })
    .await
}

#[cfg(windows)]
#[tauri::command]
async fn get_managed_text(
    item_id: String,
    state: State<'_, ShelfState>,
) -> Result<Option<String>, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        state.read_managed_text_content(&item_id).map_err(|error| error.to_string())
    })
    .await
}

fn fatal_startup_error(error: impl Display) -> ! {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let message = windows::core::HSTRING::from(format!("Could not start DropCove:\n\n{error}"));
    // SAFETY: HSTRING and the static title own valid NUL-terminated UTF-16 for this modal call.
    unsafe {
        MessageBoxW(
            None,
            &message,
            windows::core::w!("DropCove startup error"),
            MB_OK | MB_ICONERROR,
        );
    }
    std::process::exit(1)
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<settings::SettingsSnapshot, String> {
    settings::get_snapshot(&app)
}

#[tauri::command]
fn get_appearance(app: tauri::AppHandle) -> Result<windows_theme::AppearanceSnapshot, String> {
    windows_theme::snapshot(&app)
}

#[tauri::command]
fn set_appearance_mode(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    mode: settings_store::AppearanceMode,
) -> Result<settings::SettingsSnapshot, String> {
    if window.label() != settings::SETTINGS_WINDOW_LABEL {
        return Err("Appearance configuration is only available in Settings".into());
    }
    settings::set_appearance_mode(&app, mode)
}

#[tauri::command]
fn start_settings_recording(app: tauri::AppHandle) -> Result<settings::SettingsSnapshot, String> {
    settings::start_recording(&app)
}

#[tauri::command]
fn cancel_settings_recording(app: tauri::AppHandle) -> Result<settings::SettingsSnapshot, String> {
    settings::cancel_recording(&app)
}

#[tauri::command]
fn set_start_with_windows(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<settings::SettingsSnapshot, String> {
    settings::set_start_with_windows(&app, enabled)
}

#[tauri::command]
fn record_settings_key(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    key: settings_store::HotKey,
) -> Result<settings::SettingsSnapshot, String> {
    if window.label() != settings::SETTINGS_WINDOW_LABEL {
        return Err("shortcut recording is only available in Settings".into());
    }
    settings::record_key(&app, key)?;
    settings::get_snapshot(&app)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle) -> Result<(), String> {
    // Win32 hotkey registration/retirement must execute on the main HWND's owner thread.
    run_blocking(move || {
        let (sender, receiver) = std::sync::mpsc::channel();
        let save_app = app.clone();
        app.run_on_main_thread(move || {
            let _ = sender.send(settings::save(&save_app));
        })
        .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|error| error.to_string())?
    })
    .await
}

#[tauri::command]
fn cancel_settings(app: tauri::AppHandle) -> Result<(), String> {
    settings::cancel(&app)
}

#[tauri::command]
fn get_rail(app: tauri::AppHandle) -> Result<edge_rail::Projection, String> {
    let edge = match resident::rail_placement(&app)? {
        Some(placement) => placement.edge,
        None => settings::active(&app)?.rail_edge,
    };
    let rail = app.state::<resident::ResidentState>().rail_presentation()?;
    Ok(edge_rail::Projection {
        edge,
        expanded: rail.expanded,
        width: rail.width,
        height: rail.height,
    })
}

#[tauri::command]
fn on_rail_body_pointer_enter(app: tauri::AppHandle) -> Result<(), String> {
    let now = resident::now_monotonic_ms();
    let state = app.state::<resident::ResidentState>();
    let rail = state.on_rail_body_pointer_enter(now)?;
    if let Some(due) = rail.expand_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        if let Err(error) = edge_rail::arm_rail_timer(&app, remaining) {
            let _ = resident::publish_warning(&app, &error);
        }
    } else {
        edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}

#[tauri::command]
fn on_rail_button_pointer_enter(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<resident::ResidentState>();
    let rail = state.on_rail_button_pointer_enter()?;
    if rail.expand_due_ms.is_none() && rail.collapse_due_ms.is_none() {
        edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}

#[tauri::command]
fn on_rail_pointer_leave(app: tauri::AppHandle) -> Result<(), String> {
    let now = resident::now_monotonic_ms();
    let state = app.state::<resident::ResidentState>();
    let rail = state.on_rail_pointer_leave(now)?;
    if let Some(due) = rail.collapse_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        if let Err(error) = edge_rail::arm_rail_timer(&app, remaining) {
            let _ = resident::publish_warning(&app, &error);
        }
    } else {
        edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}
#[tauri::command]
fn tick_rail(app: tauri::AppHandle) -> Result<(), String> {
    resident::tick_rail(&app)
}
#[tauri::command]
fn open_shelf_from_rail(app: tauri::AppHandle) -> Result<(), String> {
    resident::open_shelf_from_rail(&app)
}
#[tauri::command]
fn set_rail_transient_hold(
    active: bool,
    reason: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let reason_enum = match reason.as_str() {
        "clear" => resident::RailTransientReason::ClearConfirmation,
        "batch-drag" => resident::RailTransientReason::BatchDragConfirmation,
        _ => return Err(format!("unknown rail transient hold reason '{reason}'")),
    };
    if let Some(remaining) = app
        .state::<resident::ResidentState>()
        .set_rail_transient_reason(reason_enum, active)?
    {
        let _ = edge_rail::arm_rail_timer(&app, remaining as u32);
    }
    Ok(())
}
#[tauri::command]
fn set_rail_placement(
    app: tauri::AppHandle,
    edge: settings_store::RailEdge,
    monitor_id: String,
) -> Result<settings::SettingsSnapshot, String> {
    settings::set_rail_placement(&app, edge, monitor_id)
}
#[tauri::command]
fn set_show_rail_over_fullscreen(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<settings::SettingsSnapshot, String> {
    settings::set_show_rail_over_fullscreen(&app, enabled)
}

#[tauri::command]
fn set_shake(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    enabled: bool,
    sensitivity: settings_store::ShakeSensitivity,
) -> Result<settings::SettingsSnapshot, String> {
    if window.label() != settings::SETTINGS_WINDOW_LABEL {
        return Err("Shake configuration is only available in Settings".into());
    }
    settings::set_shake(&app, enabled, sensitivity)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// Acquire before Tauri creates windows or opens profile persistence: secondaries only activate.
pub fn run() {
    let shutdown_requested = std::env::args_os()
        .skip(1)
        .any(|arg| arg == std::ffi::OsStr::new("--shutdown"));
    let launch = windows_instance::resolve_launch().unwrap_or_else(|error| {
        if shutdown_requested {
            eprintln!("{error}");
            std::process::exit(1);
        }
        fatal_startup_error(error)
    });
    let profile = launch.profile;
    if launch.shutdown {
        if let Err(error) = windows_instance::shutdown(&profile) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let mut lease = match windows_instance::acquire(&profile, launch.autostart) {
        Ok(windows_instance::Launch::Primary(lease)) => lease,
        Ok(windows_instance::Launch::Secondary) => return,
        Err(error) => fatal_startup_error(error),
    };
    let (shelf_state, recovery_backup) =
        ShelfState::open_or_recover(profile.join("shelf.db"))
            .unwrap_or_else(|error| fatal_startup_error(error));
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .on_page_load(|webview, _| {
            if let Some(window) = webview.app_handle().get_webview_window(webview.label()) {
                if let Err(error) = windows_theme::register(&window) {
                    if let Err(warning) = resident::publish_warning(webview.app_handle(), &error) {
                        eprintln!("{error}; warning delivery failed: {warning}");
                    }
                }
            }
        })
        .setup(move |app| {
            app.manage(shelf_state);
            app.manage(windows_drag::DragSession::default());
            app.manage(manage_items::ManageItemsState::default());
            app.manage(shelf_resize::ResizeState::default());
            app.manage(resident::ResidentState::default());
            app.manage(clear_confirmation::ClearConfirmationState::default());
            app.manage(windows_shake::ShakeRuntimeState::default());
            app.manage(windows_visuals::ShellVisualState::new());
            let window = app.get_webview_window("main").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Drop Shelf window is missing")
            })?;
            windows_drop::register(&window)?;
            manage_items::register_owner(&window).map_err(std::io::Error::other)?;
            shelf_resize::register_owner(&window).map_err(std::io::Error::other)?;
            settings::initialize(app.handle(), &profile, &window).map_err(std::io::Error::other)?;
            edge_rail::initialize(app.handle()).map_err(std::io::Error::other)?;
            resident_tray::register(app.handle()).map_err(std::io::Error::other)?;
            windows_foreground::register(&window).map_err(std::io::Error::other)?;
            settings::publish_startup_warning_if_any(app.handle());
            app.state::<ShelfState>()
                .deliver_recovery_warning(recovery_backup.as_deref(), |msg| {
                    resident::publish_warning(app.handle(), msg)
                });
            if !launch.autostart {
                resident::show_shelf(app.handle()).map_err(std::io::Error::other)?;
            }
            lease.register(&window).map_err(std::io::Error::other)?;
            app.manage(lease);
            Ok(())
        })
        .on_window_event(|window, event| {
            manage_items::on_window_event(window, event);
            clear_confirmation::on_window_event(window, event);
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = resident::dismiss_shelf(window.app_handle()) {
                        use tauri::Emitter;
                        let _ = window.emit("shelf-error", error);
                    }
                } else if matches!(event, tauri::WindowEvent::Destroyed) {
                    windows_drop::unregister(window.app_handle());
                }
                if matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. }) {
                    use tauri::Emitter;
                    if let Err(error) = window.emit("shelf-changed", ()) {
                        if let Err(warning) = resident::publish_warning(window.app_handle(), &error.to_string()) {
                            eprintln!("display reflow failed: {error}; warning delivery failed: {warning}");
                        }
                    }
                    if let Err(error) = resident::reflow_rail(window.app_handle()) {
                        let _ = resident::publish_warning(window.app_handle(), &error);
                    }
                }
            } else if window.label() == settings::SETTINGS_WINDOW_LABEL {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                    settings::on_settings_window_close_requested(window.app_handle());
                } else if let tauri::WindowEvent::Focused(focused) = event {
                    settings::on_settings_window_focus_changed(window.app_handle(), *focused);
                }
            }
            if window.label() == edge_rail::LABEL && matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. }) {
                if let Err(error) = resident::reflow_rail(window.app_handle()) {
                    let _ = resident::publish_warning(window.app_handle(), &error);
                }
            }
        })
        .invoke_handler(|invoke| {
            let webview = invoke.message.webview_ref();
            let label = webview.label();
            let command = invoke.message.command();
            let allowed = match label {
                "about" => matches!(command, "get_about_metadata" | "get_appearance"),
                "clear-temporary" => matches!(command,
                    "confirm_drop_shelf_clear_temporary" | "cancel_drop_shelf_clear_temporary" | "get_appearance"),
                "main" => command != "clear_temporary",
                _ => true,
            };
            if !allowed {
                invoke.resolver.reject("command is not permitted from this window");
                return true;
            }
            if label == "main" && clear_confirmation::is_modal_active(webview.app_handle())
                && matches!(command, "dismiss_drop_shelf" | "prepare_shelf_move" | "start_shelf_resize"
                    | "start_item_drag" | "prepare_batch_drag" | "start_batch_drag" | "accept_drop"
                    | "remove_item" | "remove_batch" | "set_item_pinned" | "set_batch_pinned"
                    | "open_manage_items" | "open_about" | "open_settings" | "save_size")
            {
                invoke.resolver.reject("Drop Shelf interaction is blocked while Clear confirmation is open");
                return true;
            }
            let handler: fn(tauri::ipc::Invoke) -> bool = tauri::generate_handler![
            request_drop_shelf_clear_temporary,
            get_drop_shelf_clear_temporary_state,
            confirm_drop_shelf_clear_temporary,
            cancel_drop_shelf_clear_temporary,
            get_about_metadata,
            open_about,
            open_settings,
            get_rail,
            open_shelf_from_rail,
            set_rail_transient_hold,
            set_rail_placement,
            set_show_rail_over_fullscreen,
            on_rail_pointer_leave,
            on_rail_body_pointer_enter,
            on_rail_button_pointer_enter,
            tick_rail,
            get_shelf,
            accept_drop,
            save_size,
            start_shelf_resize,
            get_shelf_resizing,
            remove_batch,
            dismiss_drop_shelf,
            prepare_shelf_move,
            set_item_pinned,
            set_batch_pinned,
            remove_item,
            clear_temporary,
            open_manage_items,
            get_manage_items,
            get_manage_items_keyboard,
            close_manage_items,
            get_manage_items_origin,
            prepare_item_drag,
            cancel_item_drag,
            start_item_drag,
            prepare_batch_drag,
            cancel_batch_drag,
            start_batch_drag,
            set_start_with_windows,
            set_shake,
            get_settings,
            get_appearance,
            set_appearance_mode,
            start_settings_recording,
            cancel_settings_recording,
            record_settings_key,
            save_settings,
            cancel_settings,
            get_managed_text,
            get_shelf_visual,
            get_surface_generation,
            invalidate_surface_visuals,
            cancel_shelf_visual,
            acknowledge_shelf_warning,
            ];
            handler(invoke)
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        fatal_startup_error(error);
    }
}
