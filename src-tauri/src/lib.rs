use std::fmt::Display;

use tauri::Manager;

mod drag_drop;
mod resident;
mod settings;
mod shelf;
mod surfaces;
use surfaces::shell_visuals::{get_shelf_visual, get_surface_generation, invalidate_surface_visuals, cancel_shelf_visual};

use shelf::ShelfState;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
// Acquire before Tauri creates windows or opens profile persistence: secondaries only activate.
pub fn run() {
    let shutdown_requested = std::env::args_os()
        .skip(1)
        .any(|arg| arg == std::ffi::OsStr::new("--shutdown"));
    let launch = resident::instance::resolve_launch().unwrap_or_else(|error| {
        if shutdown_requested {
            eprintln!("{error}");
            std::process::exit(1);
        }
        fatal_startup_error(error)
    });
    let profile = launch.profile;
    if launch.shutdown {
        if let Err(error) = resident::instance::shutdown(&profile) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let mut lease = match resident::instance::acquire(&profile, launch.autostart) {
        Ok(resident::instance::Launch::Primary(lease)) => lease,
        Ok(resident::instance::Launch::Secondary) => return,
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
                if let Err(error) = settings::appearance::register(&window) {
                    if let Err(warning) = resident::publish_warning(webview.app_handle(), &error) {
                        eprintln!("{error}; warning delivery failed: {warning}");
                    }
                }
            }
        })
        .setup(move |app| {
            app.manage(shelf_state);
            app.manage(drag_drop::drag::DragSession::default());
            app.manage(surfaces::manage_items::ManageItemsState::default());
            app.manage(surfaces::shelf_resize::ResizeState::default());
            app.manage(resident::ResidentState::default());
            app.manage(surfaces::clear_confirmation::ClearConfirmationState::default());
            app.manage(surfaces::selection_confirmation::SelectionConfirmations::default());
            app.manage(resident::shake::ShakeRuntimeState::default());
            app.manage(surfaces::shell_visuals::ShellVisualState::new());
            let window = app.get_webview_window("main").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Drop Shelf window is missing")
            })?;
            drag_drop::drop::register(&window)?;
            surfaces::manage_items::register_owner(&window).map_err(std::io::Error::other)?;
            surfaces::shelf_resize::register_owner(&window).map_err(std::io::Error::other)?;
            settings::initialize(app.handle(), &profile, &window).map_err(std::io::Error::other)?;
            surfaces::edge_rail::initialize(app.handle()).map_err(std::io::Error::other)?;
            resident::tray::register(app.handle()).map_err(std::io::Error::other)?;
            resident::foreground::register(&window).map_err(std::io::Error::other)?;
            settings::publish_startup_warning_if_any(app.handle());
            app.state::<ShelfState>()
                .deliver_recovery_warning(recovery_backup.as_deref(), |msg| {
                    resident::publish_warning(app.handle(), msg)
                });
            if !launch.autostart {
                resident::drop_shelf::show_shelf(app.handle()).map_err(std::io::Error::other)?;
            }
            lease.register(&window).map_err(std::io::Error::other)?;
            app.manage(lease);
            Ok(())
        })
        .on_window_event(|window, event| {
            surfaces::manage_items::on_window_event(window, event);
            surfaces::clear_confirmation::on_window_event(window, event);
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = resident::drop_shelf::dismiss_shelf(window.app_handle()) {
                        use tauri::Emitter;
                        let _ = window.emit("shelf-error", error);
                    }
                } else if matches!(event, tauri::WindowEvent::Destroyed) {
                    drag_drop::drop::unregister(window.app_handle());
                }
                if matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. }) {
                    use tauri::Emitter;
                    if let Err(error) = window.emit("shelf-changed", ()) {
                        if let Err(warning) = resident::publish_warning(window.app_handle(), &error.to_string()) {
                            eprintln!("display reflow failed: {error}; warning delivery failed: {warning}");
                        }
                    }
                    if let Err(error) = resident::rail::reflow_rail(window.app_handle()) {
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
            if window.label() == surfaces::edge_rail::LABEL && matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. }) {
                if let Err(error) = resident::rail::reflow_rail(window.app_handle()) {
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
            if label == "main" && surfaces::clear_confirmation::is_modal_active(webview.app_handle())
                && matches!(command, "dismiss_drop_shelf" | "prepare_shelf_move" | "start_shelf_resize"
                    | "start_item_drag" | "prepare_batch_drag" | "start_batch_drag" | "accept_drop"
                    | "remove_item" | "remove_batch" | "set_item_pinned" | "set_batch_pinned" | "set_batches_pinned"
                    | "prepare_batches_removal" | "remove_batches" | "set_items_pinned"
                    | "prepare_items_removal" | "remove_items"
                    | "open_manage_items" | "open_about" | "open_settings" | "save_size"
                    | "perform_item_action")
            {
                invoke.resolver.reject("Drop Shelf interaction is blocked while Clear confirmation is open");
                return true;
            }
            if surfaces::selection_confirmation::blocks(webview.app_handle(), label)
                && matches!(command, "dismiss_drop_shelf" | "prepare_shelf_move" | "start_shelf_resize"
                    | "start_item_drag" | "prepare_item_drag" | "prepare_batch_drag" | "start_batch_drag" | "accept_drop"
                    | "remove_item" | "remove_batch" | "set_item_pinned" | "set_batch_pinned" | "set_batches_pinned"
                    | "set_items_pinned" | "prepare_batches_removal" | "prepare_items_removal"
                    | "open_manage_items" | "close_manage_items" | "open_about" | "open_settings" | "save_size"
                    | "perform_item_action" | "clear_temporary" | "request_drop_shelf_clear_temporary")
            {
                invoke.resolver.reject("Interaction is blocked while selection removal confirmation is open");
                return true;
            }
            let handler: fn(tauri::ipc::Invoke) -> bool = tauri::generate_handler![
            surfaces::commands::cancel_selection_removal,
            surfaces::commands::request_drop_shelf_clear_temporary,
            surfaces::commands::get_drop_shelf_clear_temporary_state,
            surfaces::commands::confirm_drop_shelf_clear_temporary,
            surfaces::commands::cancel_drop_shelf_clear_temporary,
            surfaces::commands::get_about_metadata,
            surfaces::commands::open_about,
            settings::commands::open_settings,
            resident::commands::get_rail,
            resident::commands::open_shelf_from_rail,
            resident::commands::set_rail_transient_hold,
            settings::commands::set_rail_placement,
            settings::commands::set_show_rail_over_fullscreen,
            resident::commands::on_rail_pointer_leave,
            resident::commands::on_rail_body_pointer_enter,
            resident::commands::on_rail_button_pointer_enter,
            resident::commands::tick_rail,
            shelf::commands::get_shelf,
            shelf::commands::accept_drop,
            shelf::commands::save_size,
            surfaces::commands::start_shelf_resize,
            surfaces::commands::get_shelf_resizing,
            shelf::commands::remove_batch,
            resident::commands::dismiss_drop_shelf,
            surfaces::commands::prepare_shelf_move,
            shelf::commands::set_item_pinned,
            shelf::commands::set_batch_pinned,
            shelf::commands::set_batches_pinned,
            shelf::commands::prepare_batches_removal,
            shelf::commands::remove_batches,
            shelf::commands::set_items_pinned,
            shelf::commands::prepare_items_removal,
            shelf::commands::remove_items,
            shelf::commands::remove_item,
            shelf::commands::clear_temporary,
            surfaces::commands::open_manage_items,
            surfaces::commands::get_manage_items,
            surfaces::commands::get_manage_items_keyboard,
            surfaces::commands::close_manage_items,
            surfaces::commands::get_manage_items_origin,
            drag_drop::commands::prepare_item_drag,
            drag_drop::commands::cancel_item_drag,
            drag_drop::commands::start_item_drag,
            drag_drop::commands::prepare_batch_drag,
            drag_drop::commands::cancel_batch_drag,
            drag_drop::commands::start_batch_drag,
            settings::commands::set_start_with_windows,
            settings::commands::set_shake,
            settings::commands::get_settings,
            settings::commands::get_appearance,
            settings::commands::set_appearance_mode,
            settings::commands::start_settings_recording,
            settings::commands::cancel_settings_recording,
            settings::commands::record_settings_key,
            settings::commands::save_settings,
            settings::commands::cancel_settings,
            shelf::commands::get_managed_text,
            get_shelf_visual,
            get_surface_generation,
            invalidate_surface_visuals,
            cancel_shelf_visual,
            shelf::commands::acknowledge_shelf_warning,
            surfaces::commands::perform_item_action,
            ];
            handler(invoke)
        })
        .run(tauri::generate_context!());

    if let Err(error) = result {
        fatal_startup_error(error);
    }
}
