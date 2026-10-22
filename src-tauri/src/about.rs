//! Resident-owned About Surface module.
//!
//! Provides singleton native window management for About DropCove,
//! centered within the work area of either the Drop Shelf monitor or pointer monitor,
//! measuring the complete outer window frame and positioning in physical coordinates,
//! with authoritative runtime metadata projection from Tauri app package/config metadata.

use serde::Serialize;
use crate::windows_auxiliary::{self, InvocationContext};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub(crate) const ABOUT_WINDOW_LABEL: &str = "about";
pub(crate) const ABOUT_WIDTH: f64 = 400.0;
pub(crate) const ABOUT_HEIGHT: f64 = 460.0;

/// Authoritative display metadata projected from app package / config metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AboutMetadata {
    pub(crate) product_name: String,
    pub(crate) version: String,
    pub(crate) description: &'static str,
    pub(crate) author: String,
    pub(crate) source_url: String,
}

/// Returns authoritative About metadata using the running app handle.
///
/// - `product_name`: from `app.package_info().name` / Tauri config
/// - `version`: from `app.package_info().version`
/// - `description`: canonical Cargo package description
/// - `author`: canonical author from Cargo package metadata (`env!("CARGO_PKG_AUTHORS")`)
/// - `source_url`: canonical repository from Cargo package metadata (`env!("CARGO_PKG_REPOSITORY")`)
pub(crate) fn get_metadata(app: &AppHandle) -> AboutMetadata {
    let pkg = app.package_info();
    AboutMetadata {
        product_name: pkg.name.clone(),
        version: pkg.version.to_string(),
        description: env!("CARGO_PKG_DESCRIPTION"),
        author: env!("CARGO_PKG_AUTHORS").to_owned(),
        source_url: env!("CARGO_PKG_REPOSITORY").to_owned(),
    }
}

/// Opens or focuses the singleton About Surface.
///
/// If About already exists:
/// - Shows, raises, and focuses it without moving the window.
/// If About does not exist:
/// - Creates a decorated, fixed 400x460 logical pixel, taskbar-visible, non-modal window (hidden).
/// - Measures complete outer frame and positions in physical coordinates.
/// - Shows and focuses the window. On failure, destroys any partially constructed window.
pub(crate) fn open(app: &AppHandle, context: InvocationContext) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ABOUT_WINDOW_LABEL) {
        window
            .show()
            .map_err(|e| format!("failed to show about window: {e}"))?;
        window
            .unminimize()
            .map_err(|e| format!("failed to unminimize about window: {e}"))?;
        window
            .set_focus()
            .map_err(|e| format!("failed to focus about window: {e}"))?;
        return Ok(());
    }

    let mon = windows_auxiliary::target_monitor(app, context)?;

    let window = WebviewWindowBuilder::new(
        app,
        ABOUT_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("About DropCove")
    .theme(crate::windows_theme::current_theme(app)?)
    .inner_size(ABOUT_WIDTH, ABOUT_HEIGHT)
    .min_inner_size(ABOUT_WIDTH, ABOUT_HEIGHT)
    .max_inner_size(ABOUT_WIDTH, ABOUT_HEIGHT)
    .resizable(false)
    .decorations(true)
    .skip_taskbar(false)
    .always_on_top(false)
    .visible(false)
    .build()
    .map_err(|e| format!("failed to build about window: {e}"))?;

    let setup_result = (|| {
        crate::windows_theme::apply_to_window(&window)?;
        windows_auxiliary::center_in_work_area(&window, &mon)?;
        window
            .show()
            .map_err(|e| format!("failed to show about window: {e}"))?;
        window
            .set_focus()
            .map_err(|e| format!("failed to focus about window: {e}"))?;
        Ok(())
    })();

    if let Err(e) = setup_result {
        let _ = window.close();
        return Err(e);
    }

    Ok(())
}

/// Closes the About window if open.
pub(crate) fn close(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ABOUT_WINDOW_LABEL) {
        window
            .close()
            .map_err(|e| format!("failed to close about window: {e}"))?;
    }
    Ok(())
}

/// Clean shutdown of About resources.
pub(crate) fn shutdown(app: &AppHandle) -> Result<(), String> {
    close(app)
}

