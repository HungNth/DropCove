//! Tauri command wrappers for Settings and Appearance Mode.

use super::SettingsSnapshot;

#[tauri::command]
pub(crate) async fn open_settings(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Settings can only be requested from the Drop Shelf".to_owned());
    }
    crate::run_blocking(move || super::open(&app, crate::surfaces::placement::InvocationContext::DropShelf)).await
}

#[tauri::command]
pub(crate) fn get_settings(app: tauri::AppHandle) -> Result<SettingsSnapshot, String> {
    super::get_snapshot(&app)
}

#[tauri::command]
pub(crate) fn get_appearance(app: tauri::AppHandle) -> Result<super::appearance::AppearanceSnapshot, String> {
    super::appearance::snapshot(&app)
}

#[tauri::command]
pub(crate) fn set_appearance_mode(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    mode: super::store::AppearanceMode,
) -> Result<SettingsSnapshot, String> {
    if window.label() != super::SETTINGS_WINDOW_LABEL {
        return Err("Appearance configuration is only available in Settings".into());
    }
    super::set_appearance_mode(&app, mode)
}

#[tauri::command]
pub(crate) fn start_settings_recording(app: tauri::AppHandle) -> Result<SettingsSnapshot, String> {
    super::start_recording(&app)
}

#[tauri::command]
pub(crate) fn cancel_settings_recording(app: tauri::AppHandle) -> Result<SettingsSnapshot, String> {
    super::cancel_recording(&app)
}

#[tauri::command]
pub(crate) fn set_start_with_windows(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<SettingsSnapshot, String> {
    super::set_start_with_windows(&app, enabled)
}

#[tauri::command]
pub(crate) fn record_settings_key(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    key: super::store::HotKey,
) -> Result<SettingsSnapshot, String> {
    if window.label() != super::SETTINGS_WINDOW_LABEL {
        return Err("shortcut recording is only available in Settings".into());
    }
    super::record_key(&app, key)?;
    super::get_snapshot(&app)
}

#[tauri::command]
pub(crate) async fn save_settings(app: tauri::AppHandle) -> Result<(), String> {
    // Win32 hotkey registration/retirement must execute on the main HWND's owner thread.
    crate::run_blocking(move || {
        let (sender, receiver) = std::sync::mpsc::channel();
        let save_app = app.clone();
        app.run_on_main_thread(move || {
            let _ = sender.send(super::save(&save_app));
        })
        .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|error| error.to_string())?
    })
    .await
}

#[tauri::command]
pub(crate) fn cancel_settings(app: tauri::AppHandle) -> Result<(), String> {
    super::cancel(&app)
}

#[tauri::command]
pub(crate) fn set_rail_placement(
    app: tauri::AppHandle,
    edge: super::store::RailEdge,
    monitor_id: String,
) -> Result<SettingsSnapshot, String> {
    super::set_rail_placement(&app, edge, monitor_id)
}

#[tauri::command]
pub(crate) fn set_show_rail_over_fullscreen(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<SettingsSnapshot, String> {
    super::set_show_rail_over_fullscreen(&app, enabled)
}

#[tauri::command]
pub(crate) fn set_shake(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    enabled: bool,
    sensitivity: super::store::ShakeSensitivity,
) -> Result<SettingsSnapshot, String> {
    if window.label() != super::SETTINGS_WINDOW_LABEL {
        return Err("Shake configuration is only available in Settings".into());
    }
    super::set_shake(&app, enabled, sensitivity)
}
