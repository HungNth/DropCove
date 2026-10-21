use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, MutexGuard,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::{
        Input::KeyboardAndMouse::EnableWindow,
        WindowsAndMessaging::{
            GetWindowRect, SetForegroundWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE,
            SWP_NOZORDER,
        },
    },
};

use crate::shelf::ShelfState;

pub(crate) const LABEL: &str = "clear-temporary";
pub(crate) const DROP_SHELF_LABEL: &str = "main";
const WIDTH: f64 = 360.0;
const HEIGHT: f64 = 130.0;

#[derive(Default)]
pub(crate) struct ClearConfirmationState {
    pub(crate) active: AtomicBool,
    lock: Mutex<()>,
}

impl ClearConfirmationState {
    fn lock(&self) -> Result<MutexGuard<'_, ()>, String> {
        self.lock
            .lock()
            .map_err(|_| "clear confirmation state lock poisoned".to_owned())
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
}

pub(crate) fn is_modal_active(app: &AppHandle) -> bool {
    app.try_state::<ClearConfirmationState>()
        .map(|state| state.is_active())
        .unwrap_or(false)
}

pub(crate) fn validate_clear_request_caller(caller_label: &str) -> Result<(), String> {
    if caller_label != DROP_SHELF_LABEL {
        return Err("clear temporary items confirmation is only permitted from the Drop Shelf".into());
    }
    Ok(())
}

pub(crate) fn validate_confirmation_action_caller(caller_label: &str) -> Result<(), String> {
    if caller_label != LABEL {
        return Err("confirmation action is only permitted from the clear confirmation window".into());
    }
    Ok(())
}

pub(crate) fn guard_interaction(app: &AppHandle) -> Result<(), String> {
    if is_modal_active(app) {
        return Err("Drop Shelf interaction is blocked while clear confirmation is open".into());
    }
    Ok(())
}

fn set_modal_active(app: &AppHandle, active: bool) {
    if let Some(state) = app.try_state::<ClearConfirmationState>() {
        state.active.store(active, Ordering::Release);
    }
    let _ = app.emit("shelf-clear-confirmation-state", active);
}

fn set_owner_enabled(app: &AppHandle, enabled: bool) {
    if let Some(owner) = app.get_webview_window(DROP_SHELF_LABEL) {
        if let Ok(hwnd) = owner.hwnd() {
            // SAFETY: owner window HWND is live and owned by this process on the native UI thread.
            unsafe {
                let _ = EnableWindow(hwnd, enabled);
            }
        }
    }
}

pub(crate) fn request(app: &AppHandle, caller_label: &str) -> Result<(), String> {
    validate_clear_request_caller(caller_label)?;

    let state = app
        .try_state::<ClearConfirmationState>()
        .ok_or("clear confirmation state missing")?;
    let _guard = state.lock()?;

    let shelf = app.state::<ShelfState>();
    let snapshot = shelf.snapshot().map_err(|e| e.to_string())?;
    let has_temp = snapshot
        .batches
        .iter()
        .any(|b| b.items.iter().any(|i| !i.is_pinned));

    if !has_temp {
        return Ok(());
    }

    let owner = app
        .get_webview_window(DROP_SHELF_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let owner_hwnd = owner.hwnd().map_err(|e| e.to_string())?;

    if let Some(existing) = app.get_webview_window(LABEL) {
        set_owner_enabled(app, false);
        set_modal_active(app, true);
        let show_res = (|| {
            existing.show().map_err(|e| e.to_string())?;
            existing.set_focus().map_err(|e| e.to_string())?;
            if let Ok(hwnd) = existing.hwnd() {
                // SAFETY: existing window HWND is live and owned by this process.
                unsafe {
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            Ok::<(), String>(())
        })();
        if let Err(err) = show_res {
            let _ = close_window_and_release(app);
            return Err(err);
        }
        return Ok(());
    }

    // Build hidden first so we can measure the actual outer non-client frame and accurately clamp to work area.
    let confirmation = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("DropCove")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .owner(&owner)
        .map_err(|e| e.to_string())?
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

    let setup_res = (|| {
        let conf_hwnd = confirmation.hwnd().map_err(|e| e.to_string())?;

        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let mut outer = RECT::default();
        // SAFETY: Output structs are valid stack allocations read synchronously for live owned HWNDs.
        unsafe {
            let monitor = MonitorFromWindow(owner_hwnd, MONITOR_DEFAULTTONEAREST);
            if !GetMonitorInfoW(monitor, &mut mi).as_bool() {
                return Err("failed to read Drop Shelf monitor work area".to_owned());
            }
            GetWindowRect(conf_hwnd, &mut outer).map_err(|e| e.to_string())?;
        }

        let (x, y) = compute_work_area_placement(mi.rcWork, outer);

        // SAFETY: SetWindowPos positions the measured frame within work area bounds on the native UI thread.
        unsafe {
            SetWindowPos(
                conf_hwnd,
                None,
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .map_err(|e| e.to_string())?;
        }

        set_owner_enabled(app, false);
        set_modal_active(app, true);

        confirmation.show().map_err(|e| e.to_string())?;
        confirmation.set_focus().map_err(|e| e.to_string())?;

        Ok::<(), String>(())
    })();

    if let Err(err) = setup_res {
        let _ = confirmation.close();
        set_owner_enabled(app, true);
        set_modal_active(app, false);
        return Err(err);
    }

    Ok(())
}

pub(crate) fn compute_work_area_placement(work: RECT, outer: RECT) -> (i32, i32) {
    let frame_w = outer.right - outer.left;
    let frame_h = outer.bottom - outer.top;

    let work_w = work.right - work.left;
    let work_h = work.bottom - work.top;

    let target_x = work.left + (work_w - frame_w) / 2;
    let target_y = work.top + (work_h - frame_h) / 2;

    let max_x = (work.right - frame_w).max(work.left);
    let max_y = (work.bottom - frame_h).max(work.top);

    let clamped_x = target_x.clamp(work.left, max_x);
    let clamped_y = target_y.clamp(work.top, max_y);

    (clamped_x, clamped_y)
}

pub(crate) fn cancel(app: &AppHandle) -> Result<(), String> {
    close_window_and_release(app)
}

pub(crate) fn confirm(app: &AppHandle) -> Result<crate::shelf::ShelfSnapshot, String> {
    let state = app
        .try_state::<ClearConfirmationState>()
        .ok_or("clear confirmation state missing")?;
    let _guard = state.lock()?;

    if !state.is_active() {
        return Err("clear confirmation is not active".into());
    }

    let shelf = app.state::<ShelfState>();
    let current_snapshot = shelf.snapshot().map_err(|e| e.to_string())?;
    let has_temp = current_snapshot
        .batches
        .iter()
        .any(|b| b.items.iter().any(|i| !i.is_pinned));

    if !has_temp {
        close_window_and_release(app)?;
        return crate::publish_shelf(app, current_snapshot);
    }

    close_window_and_release(app)?;

    match shelf.clear_temporary() {
        Ok(cleared_snapshot) => crate::publish_shelf(app, cleared_snapshot),
        Err(err) => {
            let error_msg = err.to_string();
            let _ = app.emit("shelf-error", error_msg.clone());
            Err(error_msg)
        }
    }
}

pub(crate) fn synchronize(app: &AppHandle) -> Result<(), String> {
    if !is_modal_active(app) {
        return Ok(());
    }
    let shelf = app.state::<ShelfState>();
    let snapshot = shelf.snapshot().map_err(|e| e.to_string())?;
    let has_temp = snapshot
        .batches
        .iter()
        .any(|b| b.items.iter().any(|i| !i.is_pinned));
    if !has_temp {
        close_window_and_release(app)?;
    }
    Ok(())
}

pub(crate) fn shutdown(app: &AppHandle) -> Result<(), String> {
    close_window_and_release(app)
}

fn close_window_and_release(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        window.close().map_err(|e| e.to_string())?;
    }
    set_owner_enabled(app, true);
    set_modal_active(app, false);
    let _ = app.emit("shelf-clear-confirmation-closed", ());
    Ok(())
}

pub(crate) fn on_window_event(window: &Window, event: &WindowEvent) {
    if window.label() == LABEL {
        if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
            let app = window.app_handle();
            set_owner_enabled(app, true);
            set_modal_active(app, false);
            let _ = app.emit("shelf-clear-confirmation-closed", ());
        }
    } else if window.label() == DROP_SHELF_LABEL && matches!(event, WindowEvent::Destroyed) {
        let app = window.app_handle();
        let _ = close_window_and_release(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_work_area_placement_centers_when_fitting() {
        let work = RECT {
            left: 100,
            top: 50,
            right: 1100,
            bottom: 850,
        };
        let outer = RECT {
            left: 0,
            top: 0,
            right: 360,
            bottom: 200,
        };
        let (x, y) = compute_work_area_placement(work, outer);
        assert_eq!(x, 420);
        assert_eq!(y, 350);
        assert!(x >= work.left && x + 360 <= work.right);
        assert!(y >= work.top && y + 200 <= work.bottom);
    }

    #[test]
    fn compute_work_area_placement_clamps_when_narrower_than_frame() {
        let work = RECT {
            left: 0,
            top: 0,
            right: 300,
            bottom: 150,
        };
        let outer = RECT {
            left: 0,
            top: 0,
            right: 360,
            bottom: 200,
        };
        let (x, y) = compute_work_area_placement(work, outer);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }

    #[test]
    fn caller_validation_permits_drop_shelf_only() {
        assert!(validate_clear_request_caller("main").is_ok());
        assert!(validate_clear_request_caller("clear-temporary").is_err());
        assert!(validate_clear_request_caller("about").is_err());
        assert!(validate_clear_request_caller("settings").is_err());
        assert!(validate_clear_request_caller("manage-items").is_err());
        assert!(validate_clear_request_caller("edge-rail").is_err());
    }

    #[test]
    fn confirmation_action_validation_permits_clear_window_only() {
        assert!(validate_confirmation_action_caller("clear-temporary").is_ok());
        assert!(validate_confirmation_action_caller("main").is_err());
        assert!(validate_confirmation_action_caller("about").is_err());
        assert!(validate_confirmation_action_caller("settings").is_err());
        assert!(validate_confirmation_action_caller("edge-rail").is_err());
    }
}
