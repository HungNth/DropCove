//! Resident-owned About Surface module.
//!
//! Provides singleton native window management for About DropCove,
//! centered within the work area of either the Drop Shelf monitor or pointer monitor,
//! measuring the complete outer window frame and positioning in physical coordinates,
//! with authoritative runtime metadata projection from Tauri app package/config metadata.

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub(crate) const ABOUT_WINDOW_LABEL: &str = "about";
pub(crate) const ABOUT_WIDTH: f64 = 400.0;
pub(crate) const ABOUT_HEIGHT: f64 = 460.0;

/// Authoritative display metadata projected from app package / config metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AboutMetadata {
    pub(crate) product_name: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) source_url: String,
}

/// Invocation context determining initial monitor placement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum AboutInvocationContext {
    DropShelf,
    NotificationArea,
}

/// Returns authoritative About metadata using the running app handle.
///
/// - `product_name`: from `app.package_info().name` / Tauri config
/// - `version`: from `app.package_info().version`
/// - `author`: canonical author from Cargo package metadata (`env!("CARGO_PKG_AUTHORS")`)
/// - `source_url`: canonical repository from Cargo package metadata (`env!("CARGO_PKG_REPOSITORY")`)
pub(crate) fn get_metadata(app: &AppHandle) -> AboutMetadata {
    let pkg = app.package_info();
    AboutMetadata {
        product_name: pkg.name.clone(),
        version: pkg.version.to_string(),
        author: env!("CARGO_PKG_AUTHORS").to_owned(),
        source_url: env!("CARGO_PKG_REPOSITORY").to_owned(),
    }
}

/// Pure calculation of centered and clamped physical window position within a physical work area.
pub(crate) fn calculate_centered_outer_position(
    work_left: i32,
    work_top: i32,
    work_width: i32,
    work_height: i32,
    outer_width: i32,
    outer_height: i32,
) -> (i32, i32) {
    let clamped_w = outer_width.min(work_width).max(1);
    let clamped_h = outer_height.min(work_height).max(1);

    let target_x = work_left + (work_width - clamped_w) / 2;
    let target_y = work_top + (work_height - clamped_h) / 2;

    let max_x = work_left + work_width - clamped_w;
    let min_x = work_left;
    let clamped_x = if max_x >= min_x {
        target_x.clamp(min_x, max_x)
    } else {
        min_x
    };

    let max_y = work_top + work_height - clamped_h;
    let min_y = work_top;
    let clamped_y = if max_y >= min_y {
        target_y.clamp(min_y, max_y)
    } else {
        min_y
    };

    (clamped_x, clamped_y)
}

/// Measures the window's complete outer frame size in physical pixels.
fn measure_outer_frame_physical(window: &WebviewWindow) -> Result<(i32, i32), String> {
        use windows::Win32::Foundation::{HWND, RECT};
        use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let mut outer = RECT::default();
        // SAFETY: HWND is valid and live; RECT is valid writable memory.
        unsafe {
            GetWindowRect(HWND(hwnd.0), &mut outer).map_err(|e| e.to_string())?;
        }
        let w = (outer.right - outer.left).max(1);
        let h = (outer.bottom - outer.top).max(1);
        Ok((w, h))
}

/// Positions the About window centered within the target monitor's work area using physical coordinates.
fn center_window_in_work_area(
    window: &WebviewWindow,
    target_monitor: &tauri::Monitor,
) -> Result<(), String> {
    let (outer_w, outer_h) = measure_outer_frame_physical(window)?;
    let work_area = target_monitor.work_area();

    let (x, y) = calculate_centered_outer_position(
        work_area.position.x,
        work_area.position.y,
        work_area.size.width as i32,
        work_area.size.height as i32,
        outer_w,
        outer_h,
    );

    window
        .set_position(tauri::PhysicalPosition::new(x, y))
        .map_err(|e| format!("failed to position about window: {e}"))?;

    Ok(())
}

/// Finds the target monitor based on invocation context.
fn target_monitor(
    app: &AppHandle,
    context: AboutInvocationContext,
) -> Result<tauri::Monitor, String> {
    match context {
        AboutInvocationContext::DropShelf => {
            if let Some(shelf_window) = app.get_webview_window("main") {
                if let Ok(Some(mon)) = shelf_window.current_monitor() {
                    return Ok(mon);
                }
            }
            app.primary_monitor()
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "no monitor available".to_owned())
        }
        AboutInvocationContext::NotificationArea => {
            if let Ok(cursor) = app.cursor_position() {
                if let Ok(monitors) = app.available_monitors() {
                    for mon in monitors {
                        let pos = mon.position();
                        let size = mon.size();
                        let left = pos.x as f64;
                        let top = pos.y as f64;
                        let right = left + size.width as f64;
                        let bottom = top + size.height as f64;
                        if cursor.x >= left
                            && cursor.x <= right
                            && cursor.y >= top
                            && cursor.y <= bottom
                        {
                            return Ok(mon);
                        }
                    }
                }
            }
            app.primary_monitor()
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "no monitor available".to_owned())
        }
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
pub(crate) fn open(app: &AppHandle, context: AboutInvocationContext) -> Result<(), String> {
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

    let mon = target_monitor(app, context)?;

    let window = WebviewWindowBuilder::new(
        app,
        ABOUT_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("About DropCove")
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
        center_window_in_work_area(&window, &mon)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outer_position_centering_and_clamping_invariants() {
        // Standard 1920x1080 monitor, work area 0,0, 1920, 1040, outer window 416x499
        let (x, y) = calculate_centered_outer_position(0, 0, 1920, 1040, 416, 499);
        assert_eq!(x, (1920 - 416) / 2);
        assert_eq!(y, (1040 - 499) / 2);

        // Secondary monitor with negative offset
        let (x2, y2) = calculate_centered_outer_position(-1920, 100, 1920, 1080, 400, 460);
        assert_eq!(x2, -1920 + (1920 - 400) / 2);
        assert_eq!(y2, 100 + (1080 - 460) / 2);

        // Narrow work area: clamp ensures window stays within bounds without underflow
        let (cx, cy) = calculate_centered_outer_position(100, 50, 300, 350, 400, 460);
        assert_eq!(cx, 100);
        assert_eq!(cy, 50);
    }
}
