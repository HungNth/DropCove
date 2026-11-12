pub(crate) mod commands;
pub(crate) mod drop_shelf;
pub(crate) mod foreground;
pub(crate) mod fullscreen;
pub(crate) mod hotkey;
pub(crate) mod instance;
pub(crate) mod rail;
pub(crate) mod shake;
pub(crate) mod shake_session;
pub(crate) mod startup;
pub(crate) mod tray;
pub(crate) mod window_ops;
#[cfg(test)]
mod tests;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::{
        HiDpi::{GetDpiForMonitor, GetDpiForWindow},
        WindowsAndMessaging::{
            GetClientRect, GetCursorPos, GetWindowRect, IsWindowVisible, SetWindowPos,
            HWND_TOPMOST, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            SWP_SHOWWINDOW,
        },
    },
};

use crate::{settings::store::Settings, shelf::ShelfState};
use window_ops::{NativeWindowOps, WindowOps};
use rail::{RailPresentation, RailTransientReason};
use shake_session::ShakeSession;


const SHELF_WINDOW_LABEL: &str = "main";

/// Transient presentation state of the resident coordinator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Visibility {
    Hidden,
    Visible,
    EdgeDocked,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Hidden
    }
}

static MONOTONIC_START: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);

pub(crate) fn now_monotonic_ms() -> u64 {
    std::time::Instant::now()
        .duration_since(*MONOTONIC_START)
        .as_millis() as u64
}

#[derive(Default)]
struct Presentation {
    visibility: Visibility,
    rail_placement: Option<crate::surfaces::edge_rail::Placement>,
    rail: RailPresentation,
    transient_holds: u8,
    pointer_inside: bool,
    fullscreen_occluded: bool,
    shake_session: Option<ShakeSession>,
    next_shake_id: u64,
}

#[derive(Default)]
pub(crate) struct ResidentState {
    presentation: Mutex<Presentation>,
    had_content: AtomicBool,
    showing: AtomicBool,
    exiting: AtomicBool,
}

fn coordinate_repair(ops: &dyn WindowOps, state: &ResidentState) -> Result<(), String> {
    if state.visibility()? == Visibility::Visible && !state.exiting.load(Ordering::Acquire) {
        ops.reassert_topmost()?;
    }
    Ok(())
}

fn coordinate_exit(
    state: &ResidentState,
    busy: bool,
    teardown: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if busy {
        return Err("Finish the current Shelf operation before exiting DropCove".to_owned());
    }
    if state.exiting.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    if let Err(error) = teardown() {
        state.exiting.store(false, Ordering::Release);
        return Err(error);
    }
    state.set_visibility(Visibility::Hidden)
}

pub(crate) fn repair_topmost(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::Visible {
        return Ok(());
    }
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    coordinate_repair(&NativeWindowOps::new(&window), state.inner())
}

pub(crate) fn publish_warning(app: &AppHandle, message: &str) -> Result<(), String> {
    crate::resident::tray::warning(app, message)
}

pub(crate) fn exit(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    let busy = app.state::<crate::drag_drop::drag::DragSession>().is_active()
        || crate::surfaces::shelf_resize::is_resizing(app)?
        || app.state::<ShelfState>().has_pending_operations();
    coordinate_exit(state.inner(), busy, || {
        crate::surfaces::clear_confirmation::shutdown(app)?;
        crate::surfaces::about::shutdown(app)?;
        crate::surfaces::manage_items::close(app)?;
        crate::surfaces::edge_rail::hide(app)?;
        crate::resident::foreground::shutdown(app)?;
        crate::drag_drop::drop::unregister(app);
        app.state::<ShelfState>()
            .close()
            .map_err(|error| error.to_string())?;
        crate::settings::shutdown(app)?;
        crate::resident::tray::shutdown(app)?;
        app.exit(0);
        Ok(())
    })
}

fn coordinate_content(
    state: &ResidentState,
    has_content: bool,
    hide: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if has_content {
        state.had_content.store(true, Ordering::Release);
        return Ok(());
    }
    let is_docked = state.visibility()? == Visibility::EdgeDocked;
    let had_content = state.had_content.load(Ordering::Acquire);
    if !had_content && !is_docked {
        return Ok(());
    }
    hide()?;
    state.set_visibility(Visibility::Hidden)?;
    state.had_content.store(false, Ordering::Release);
    Ok(())
}

fn synchronize_on_window_thread(app: &AppHandle, has_content: bool) -> Result<(), String> {
    coordinate_content(&app.state::<ResidentState>(), has_content, || {
        crate::surfaces::manage_items::close(app)?;
        let _ = app.emit("rail-transients-closed", ());
        crate::surfaces::edge_rail::hide(app)?;
        if let Some(window) = app.get_webview_window(SHELF_WINDOW_LABEL) {
            NativeWindowOps::new(&window).hide_window()?;
        }
        Ok(())
    })
}

pub(crate) fn synchronize(app: &AppHandle, has_content: bool) -> Result<(), String> {
    use windows::Win32::{
        System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    // SAFETY: Our live main HWND determines the serialized owner-thread transition path.
    if unsafe { GetWindowThreadProcessId(hwnd, None) == GetCurrentThreadId() } {
        return synchronize_on_window_thread(app, has_content);
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = sender.send(synchronize_on_window_thread(&app_handle, has_content));
    })
    .map_err(|e| e.to_string())?;
    receiver.recv().map_err(|e| e.to_string())?
}

impl ResidentState {
    pub(crate) fn visibility(&self) -> Result<Visibility, String> {
        self.presentation
            .lock()
            .map(|v| v.visibility)
            .map_err(|_| "resident coordinator state lock is poisoned".to_owned())
    }

    fn placement(&self) -> Result<Option<crate::surfaces::edge_rail::Placement>, String> {
        self.presentation
            .lock()
            .map(|value| value.rail_placement.clone())
            .map_err(|_| "resident presentation lock is poisoned".into())
    }

    fn set_visibility(&self, next: Visibility) -> Result<(), String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident coordinator state lock is poisoned".to_owned())?;
        guard.visibility = next;
        guard.shake_session = None;
        if next != Visibility::EdgeDocked {
            guard.rail_placement = None;
            guard.rail = RailPresentation::default();
            guard.transient_holds &= RailTransientReason::ActiveDrag as u8;
            guard.fullscreen_occluded = false;
        }
        Ok(())
    }
}
