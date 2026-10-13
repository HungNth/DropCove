use std::fmt::Display;

use tauri::{Manager, State};

mod manage_items;
mod resident;
mod resident_tray;
mod settings;
mod settings_store;
mod settings_transaction;
mod shelf;
mod shelf_resize;
#[cfg(windows)]
mod windows_drag;
#[cfg(windows)]
mod windows_drop;
mod windows_foreground;
mod windows_hotkey;
mod windows_instance;
mod windows_paths;
mod windows_startup;

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
    manage_items::synchronize(app)?;
    Ok(snapshot)
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
async fn open_manage_items(
    batch_id: String,
    anchor: manage_items::AnchorRect,
    keyboard: bool,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<(), String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let batch = state
            .management_batch(&batch_id)
            .map_err(|error| error.to_string())?;
        if batch.items.len() < 2 {
            return Err("Manage Items requires a multi-item batch".to_owned());
        }
        manage_items::open(&app, &batch, anchor, keyboard)?;
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
        if result.snapshot.is_some() {
            manage_items::synchronize(&app)?;
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
    batch_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<shelf::BatchPreparation, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let preparation = state
            .prepare_batch_drag(&batch_id)
            .map_err(|error| error.to_string())?;
        manage_items::synchronize(&app)?;
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
            .complete_batch_drag(&token, shelf::DragOutcome::Canceled)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
async fn start_batch_drag(
    token: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        manage_items::close(&app)?;
        let participants = state
            .prepared_participants(&token)
            .map_err(|error| error.to_string())?;
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            windows_drag::perform_storage_drag(
                session,
                participants.iter().map(|item| item.path.as_str()),
            )
        });
        match outcome {
            Ok(outcome) => {
                let snapshot = state
                    .complete_batch_drag(&token, outcome)
                    .map_err(|error| error.to_string())?;
                publish_shelf(&app, snapshot)
            }
            Err(error) => {
                state
                    .complete_batch_drag(&token, shelf::DragOutcome::Canceled)
                    .map_err(|cancel| cancel.to_string())?;
                manage_items::synchronize(&app)?;
                Err(error)
            }
        }
    })
    .await
}

#[cfg(windows)]
#[tauri::command]
async fn start_item_drag(
    item_id: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let path = state
            .item_path(&item_id)
            .map_err(|error| error.to_string())?;
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            windows_drag::perform_storage_drag(session, std::iter::once(path.as_str()))
        })?;
        if outcome == shelf::DragOutcome::Canceled {
            use tauri::Emitter;
            app.emit_to("manage-items", "item-drag-canceled", ())
                .map_err(|error| error.to_string())?;
        }
        let snapshot = state
            .complete_item_drag(&item_id, outcome)
            .map_err(|error| error.to_string())?;
        publish_shelf(&app, snapshot)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// Acquire before Tauri creates windows or opens profile persistence: secondaries only activate.
pub fn run() {
    let launch =
        windows_instance::resolve_launch().unwrap_or_else(|error| fatal_startup_error(error));
    let profile = launch.profile;
    let mut lease = match windows_instance::acquire(&profile, launch.autostart) {
        Ok(windows_instance::Launch::Primary(lease)) => lease,
        Ok(windows_instance::Launch::Secondary) => return,
        Err(error) => fatal_startup_error(error),
    };
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            app.manage(ShelfState::open(profile.join("shelf.db"))?);
            app.manage(windows_drag::DragSession::default());
            app.manage(manage_items::ManageItemsState::default());
            app.manage(shelf_resize::ResizeState::default());
            app.manage(resident::ResidentState::default());
            let window = app.get_webview_window("main").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Drop Shelf window is missing")
            })?;
            windows_drop::register(&window)?;
            manage_items::register_owner(&window).map_err(std::io::Error::other)?;
            shelf_resize::register_owner(&window).map_err(std::io::Error::other)?;
            settings::initialize(app.handle(), &profile, &window).map_err(std::io::Error::other)?;
            resident_tray::register(app.handle()).map_err(std::io::Error::other)?;
            windows_foreground::register(&window).map_err(std::io::Error::other)?;
            settings::publish_startup_warning_if_any(app.handle());
            if !launch.autostart {
                resident::show_shelf(app.handle()).map_err(std::io::Error::other)?;
            }
            lease.register(&window).map_err(std::io::Error::other)?;
            app.manage(lease);
            Ok(())
        })
        .on_window_event(|window, event| {
            manage_items::on_window_event(window, event);
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = resident::hide_shelf(window.app_handle()) {
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
                }
            } else if window.label() == settings::SETTINGS_WINDOW_LABEL {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                    settings::on_settings_window_close_requested(window.app_handle());
                } else if let tauri::WindowEvent::Focused(focused) = event {
                    settings::on_settings_window_focus_changed(window.app_handle(), *focused);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_shelf,
            accept_drop,
            save_size,
            start_shelf_resize,
            get_shelf_resizing,
            remove_batch,
            set_item_pinned,
            set_batch_pinned,
            remove_item,
            clear_temporary,
            open_manage_items,
            get_manage_items,
            get_manage_items_keyboard,
            close_manage_items,
            start_item_drag,
            prepare_batch_drag,
            cancel_batch_drag,
            start_batch_drag,
            set_start_with_windows,
            get_settings,
            start_settings_recording,
            cancel_settings_recording,
            record_settings_key,
            save_settings,
            cancel_settings
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        fatal_startup_error(error);
    }
}
