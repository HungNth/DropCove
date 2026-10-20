//! Notification-area (system tray) residency adapter for DropCove.
//!
//! Provides the primary resident tray icon with context menu ("Exit DropCove"),
//! click/double-click always-show routing through the coordinator, warning notifications,
//! and clean shutdown removing tray resources.

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

pub(crate) const TRAY_ID: &str = "resident";
pub(crate) const MENU_SETTINGS_ID: &str = "settings";
pub(crate) const MENU_SETTINGS_LABEL: &str = "Settings";
pub(crate) const MENU_ABOUT_ID: &str = "about";
pub(crate) const MENU_ABOUT_LABEL: &str = "About";
pub(crate) const MENU_EXIT_ID: &str = "exit-dropcove";
pub(crate) const MENU_EXIT_LABEL: &str = "Exit DropCove";
pub(crate) const WARNING_TITLE: &str = "DropCove warning";

/// Registers the primary notification-area tray icon and its context menu.
///
/// Context menu exposes "Exit DropCove" (id "exit-dropcove").
/// Left-click and double-left-click route through coordinator `resident::show_shelf`.
/// Menu Exit DropCove routes through coordinator `resident::exit`.
pub(crate) fn register(app: &AppHandle) -> Result<(), String> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Err("resident tray is already registered".to_owned());
    }
    let icon = app
        .default_window_icon()
        .ok_or("resident tray icon is missing")?;
    let menu = tauri::menu::MenuBuilder::new(app)
        .text(MENU_SETTINGS_ID, MENU_SETTINGS_LABEL)
        .text(MENU_ABOUT_ID, MENU_ABOUT_LABEL)
        .text(MENU_EXIT_ID, MENU_EXIT_LABEL)
        .build()
        .map_err(|e| format!("failed to build resident tray menu: {e}"))?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon.clone())
        .tooltip("DropCove")
        .menu(&menu)
        .show_menu_on_left_click(false);

    let app_handle = app.clone();
    builder = builder.on_tray_icon_event(move |_tray, event| match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            if let Err(e) = crate::resident::show_shelf(&app_handle) {
                if let Err(warning) = crate::resident::publish_warning(&app_handle, &e) {
                    eprintln!("{e}; warning delivery failed: {warning}");
                }
            }
        }
        _ => {}
    });

    let app_handle = app.clone();
    builder = builder.on_menu_event(move |_app, event| {
        if event.id().as_ref() == MENU_SETTINGS_ID {
            if let Err(e) = crate::settings::open(&app_handle, crate::windows_auxiliary::InvocationContext::NotificationArea) {
                if let Err(warning) = crate::resident::publish_warning(&app_handle, &e) {
                    eprintln!("{e}; warning delivery failed: {warning}");
                }
            }
        } else if event.id().as_ref() == MENU_ABOUT_ID {
            if let Err(e) = crate::about::open(&app_handle, crate::windows_auxiliary::InvocationContext::NotificationArea) {
                if let Err(warning) = crate::resident::publish_warning(&app_handle, &e) {
                    eprintln!("{e}; warning delivery failed: {warning}");
                }
            }
        } else if event.id().as_ref() == MENU_EXIT_ID {
            if let Err(e) = crate::resident::exit(&app_handle) {
                if let Err(warning) = crate::resident::publish_warning(&app_handle, &e) {
                    eprintln!("{e}; warning delivery failed: {warning}");
                }
            }
        }
    });

    builder
        .build(app)
        .map_err(|e| format!("failed to create resident tray icon: {e}"))?;

    Ok(())
}

/// Shuts down and removes tray resources from the system tray.
///
/// Tauri's `remove_tray_by_id` drops the internal `TrayIcon` resource and calls
/// `TrayIcon::close`, removing the notification area icon from Windows Shell.
pub(crate) fn shutdown(app: &AppHandle) -> Result<(), String> {
    drop(app.remove_tray_by_id(TRAY_ID));
    Ok(())
}

/// Publishes a resident warning notification via the official Tauri notification plugin.
///
/// Title is fixed to "DropCove warning" with the provided body text.
pub(crate) fn warning(app: &AppHandle, message: &str) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    app.notification()
        .builder()
        .title(WARNING_TITLE)
        .body(message)
        .show()
        .map_err(|e| format!("failed to show resident notification: {e}"))
}
