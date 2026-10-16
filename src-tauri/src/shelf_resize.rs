use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, MutexGuard,
};

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::ReleaseCapture,
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            GetClientRect, GetCursorPos, GetWindowRect, PostMessageW, HTBOTTOM, HTBOTTOMLEFT,
            HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, WMSZ_BOTTOM,
            WMSZ_BOTTOMLEFT, WMSZ_BOTTOMRIGHT, WMSZ_LEFT, WMSZ_RIGHT, WMSZ_TOP, WMSZ_TOPLEFT,
            WMSZ_TOPRIGHT, WM_ENTERSIZEMOVE, WM_EXITSIZEMOVE, WM_MOUSEMOVE, WM_NCDESTROY,
            WM_NCLBUTTONDOWN, WM_NCMOUSELEAVE, WM_NCMOUSEMOVE, WM_SIZING, SWP_NOACTIVATE,
            SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
        },
    },
};

use crate::{shelf::ShelfState, windows_drag::DragSession};

const SUBCLASS_ID: usize = 0xDC05;

#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) enum ResizeEdge {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl ResizeEdge {
    fn hit(self) -> u32 {
        match self {
            Self::North => HTTOP,
            Self::NorthEast => HTTOPRIGHT,
            Self::East => HTRIGHT,
            Self::SouthEast => HTBOTTOMRIGHT,
            Self::South => HTBOTTOM,
            Self::SouthWest => HTBOTTOMLEFT,
            Self::West => HTLEFT,
            Self::NorthWest => HTTOPLEFT,
        }
    }
    fn from_sizing(value: u32) -> Option<Self> {
        match value {
            WMSZ_TOP => Some(Self::North),
            WMSZ_TOPRIGHT => Some(Self::NorthEast),
            WMSZ_RIGHT => Some(Self::East),
            WMSZ_BOTTOMRIGHT => Some(Self::SouthEast),
            WMSZ_BOTTOM => Some(Self::South),
            WMSZ_BOTTOMLEFT => Some(Self::SouthWest),
            WMSZ_LEFT => Some(Self::West),
            WMSZ_TOPLEFT => Some(Self::NorthWest),
            _ => None,
        }
    }
    fn changes_height(self) -> bool {
        !matches!(self, Self::East | Self::West)
    }
}

struct Session {
    initial_height: f64,
    edge: Option<ResizeEdge>,
}
#[derive(Default)]
struct Tracking {
    session: Option<Session>,
    hovered: bool,
}
#[derive(Default)]
pub(crate) struct ResizeState {
    active: AtomicBool,
    tracking: Mutex<Tracking>,
}
impl ResizeState {
    fn lock(&self) -> Result<MutexGuard<'_, Tracking>, String> {
        self.tracking
            .lock()
            .map_err(|_| "native resize tracking lock is poisoned".to_owned())
    }
}

pub(crate) fn is_resizing(app: &AppHandle) -> Result<bool, String> {
    Ok(app.state::<ResizeState>().active.load(Ordering::Acquire))
}

fn scale(hwnd: HWND) -> Result<f64, String> {
    // SAFETY: Callers provide a live owned HWND; no pointer memory is borrowed by this API.
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    if dpi == 0 {
        return Err("could not read Drop Shelf DPI".to_owned());
    }
    Ok(f64::from(dpi) / 96.0)
}

fn client_size(hwnd: HWND) -> Result<(f64, f64), String> {
    let mut rect = RECT::default();
    // SAFETY: RECT is a valid writable output; HWND is the observed native window.
    unsafe { GetClientRect(hwnd, &mut rect) }.map_err(|error| error.to_string())?;
    let scale = scale(hwnd)?;
    Ok((
        f64::from(rect.right - rect.left) / scale,
        f64::from(rect.bottom - rect.top) / scale,
    ))
}

pub(crate) fn register_owner(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let context = Box::into_raw(Box::new(window.app_handle().clone()));
    // SAFETY: Setup is on the owning UI thread. The installed subclass owns the boxed AppHandle
    // through WM_NCDESTROY; failed installation releases it immediately.
    if !unsafe { SetWindowSubclass(hwnd, Some(observe), SUBCLASS_ID, context as usize) }.as_bool() {
        // SAFETY: No observer was installed, so the allocation cannot be accessed by a callback.
        unsafe {
            drop(Box::from_raw(context));
        }
        return Err("could not observe native Drop Shelf resizing".to_owned());
    }
    Ok(())
}

pub(crate) fn start(app: &AppHandle, edge: ResizeEdge) -> Result<(), String> {
    if app.state::<DragSession>().is_active() || is_resizing(app)? {
        return Err("another native interaction is already active".to_owned());
    }
    crate::manage_items::close(app)?;
    let window = app
        .get_webview_window("main")
        .ok_or("Drop Shelf window is missing")?;
    let handle = window.hwnd().map_err(|error| error.to_string())?.0 as isize;
    let error_app = app.clone();
    app.run_on_main_thread(move || {
        let result = (|| {
            let mut point = POINT::default();
            // SAFETY: This is the native UI thread. ReleaseCapture affects only this app's capture.
            // WM_NCLBUTTONDOWN receives packed signed screen coordinates by value, never a pointer
            // to a temporary stack allocation; the message owns its scalar payload after posting.
            unsafe {
                GetCursorPos(&mut point).map_err(|error| error.to_string())?;
                let _ = ReleaseCapture();
                let packed = u32::from(point.x as u16) | (u32::from(point.y as u16) << 16);
                PostMessageW(
                    Some(HWND(handle as *mut _)),
                    WM_NCLBUTTONDOWN,
                    WPARAM(edge.hit() as usize),
                    LPARAM(packed as isize),
                )
                .map_err(|error| error.to_string())
            }
        })();
        if let Err(error) = result {
            let _ = error_app.emit("shelf-error", error);
        }
    })
    .map_err(|error| error.to_string())
}

pub(crate) fn prepare_move(app: &AppHandle) -> Result<(), String> {
    if app.state::<DragSession>().is_active() || is_resizing(app)? {
        return Err("another native interaction is already active".to_owned());
    }
    crate::manage_items::close_explicit(app)
}

fn clamp_rect(hwnd: HWND, edge: ResizeEdge, rect: &mut RECT) -> Result<(), String> {
    let mut monitor = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let mut outer = RECT::default();
    let mut client = RECT::default();
    // SAFETY: All outputs are valid stack allocations, used synchronously for the live HWND.
    unsafe {
        let handle = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if !GetMonitorInfoW(handle, &mut monitor).as_bool() {
            return Err("could not read resize monitor work area".to_owned());
        }
        GetWindowRect(hwnd, &mut outer).map_err(|error| error.to_string())?;
        GetClientRect(hwnd, &mut client).map_err(|error| error.to_string())?;
    }
    let work = monitor.rcWork;
    let dpi = scale(hwnd)?;
    let min_width = ((191.0 * dpi).round() as i32 + outer.right - outer.left - client.right)
        .min(work.right - work.left);
    let min_height = ((180.0 * dpi).round() as i32 + outer.bottom - outer.top - client.bottom)
        .min(work.bottom - work.top);
    rect.left = rect.left.max(work.left);
    rect.right = rect.right.min(work.right);
    rect.top = rect.top.max(work.top);
    rect.bottom = rect.bottom.min(work.bottom);
    if rect.right - rect.left < min_width {
        if matches!(
            edge,
            ResizeEdge::West | ResizeEdge::NorthWest | ResizeEdge::SouthWest
        ) {
            rect.left = rect.right - min_width;
        } else {
            rect.right = rect.left + min_width;
        }
    }
    if rect.bottom - rect.top < min_height {
        if matches!(
            edge,
            ResizeEdge::North | ResizeEdge::NorthWest | ResizeEdge::NorthEast
        ) {
            rect.top = rect.bottom - min_height;
        } else {
            rect.bottom = rect.top + min_height;
        }
    }
    Ok(())
}

fn calculate_work_area_clamp(
    outer_left: i32,
    outer_top: i32,
    width: i32,
    height: i32,
    work_left: i32,
    work_top: i32,
    work_right: i32,
    work_bottom: i32,
) -> (i32, i32) {
    let mut new_x = outer_left;
    let mut new_y = outer_top;
    if outer_left + width > work_right {
        new_x = (work_right - width).max(work_left);
    }
    if outer_left < work_left {
        new_x = work_left;
    }
    if outer_top + height > work_bottom {
        new_y = (work_bottom - height).max(work_top);
    }
    if outer_top < work_top {
        new_y = work_top;
    }
    (new_x, new_y)
}

fn clamp_window_to_work_area(hwnd: HWND) -> Result<(), String> {
    let mut monitor = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let mut outer = RECT::default();
    // SAFETY: Output structs are valid stack allocations synchronously read for the live HWND.
    unsafe {
        let handle = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if !GetMonitorInfoW(handle, &mut monitor).as_bool() {
            return Err("could not read window monitor work area".to_owned());
        }
        GetWindowRect(hwnd, &mut outer).map_err(|error| error.to_string())?;
    }
    let work = monitor.rcWork;
    let (new_x, new_y) = calculate_work_area_clamp(
        outer.left,
        outer.top,
        outer.right - outer.left,
        outer.bottom - outer.top,
        work.left,
        work.top,
        work.right,
        work.bottom,
    );
    if new_x != outer.left || new_y != outer.top {
        // SAFETY: SetWindowPos with SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE moves the window
        // inside the work area without altering z-order, focus, or size.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                new_x,
                new_y,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
            )
            .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn manual_height(initial: f64, height: f64, edge: ResizeEdge) -> Option<f64> {
    (edge.changes_height() && (height - initial).abs() > 0.5).then_some(height.max(180.0))
}

fn complete(app: &AppHandle, hwnd: HWND) -> Result<(), String> {
    if let Err(error) = clamp_window_to_work_area(hwnd) {
        let _ = app.emit("shelf-error", error);
    }
    let session = app.state::<ResizeState>().lock()?.session.take();
    let dimensions = client_size(hwnd);
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| {
            let (width, height) = dimensions?;
            if let Some(Session {
                initial_height,
                edge: Some(edge),
            }) = session
            {
                let snapshot = app
                    .state::<ShelfState>()
                    .save_size(
                        width.max(191.0),
                        manual_height(initial_height, height, edge),
                    )
                    .map_err(|error| error.to_string())?;
                crate::publish_shelf(&app, snapshot)?;
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            let _ = app.emit("shelf-error", error);
        }
        app.state::<ResizeState>()
            .active
            .store(false, Ordering::Release);
        let _ = app.emit("shelf-resize-state", false);
        let _ = app.emit("shelf-changed", ());
    });
    Ok(())
}

fn handle(
    app: &AppHandle,
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Result<(), String> {
    match message {
        WM_ENTERSIZEMOVE => {
            let (_, height) = client_size(hwnd)?;
            crate::manage_items::close(app)?;
            let state = app.state::<ResizeState>();
            state.lock()?.session = Some(Session {
                initial_height: height,
                edge: None,
            });
            state.active.store(true, Ordering::Release);
            app.emit("shelf-resize-state", true)
                .map_err(|error| error.to_string())?;
        }
        WM_SIZING => {
            if let Some(edge) = ResizeEdge::from_sizing(wparam.0 as u32) {
                // SAFETY: WM_SIZING supplies a live writable RECT for the synchronous callback.
                let rect = unsafe { (lparam.0 as *mut RECT).as_mut() }
                    .ok_or("native resize bounds are missing")?;
                clamp_rect(hwnd, edge, rect)?;
                if let Some(session) = app.state::<ResizeState>().lock()?.session.as_mut() {
                    session.edge = Some(edge);
                }
            }
        }
        WM_EXITSIZEMOVE => complete(app, hwnd)?,
        WM_NCMOUSEMOVE | WM_NCMOUSELEAVE | WM_MOUSEMOVE => {
            let hovered = message == WM_NCMOUSEMOVE
                && matches!(
                    wparam.0 as u32,
                    HTTOP
                        | HTTOPRIGHT
                        | HTRIGHT
                        | HTBOTTOMRIGHT
                        | HTBOTTOM
                        | HTBOTTOMLEFT
                        | HTLEFT
                        | HTTOPLEFT
                );
            let state = app.state::<ResizeState>();
            let changed = {
                let mut tracking = state.lock()?;
                let changed = tracking.hovered != hovered;
                tracking.hovered = hovered;
                changed
            };
            if changed {
                app.emit("shelf-resize-hover", hovered)
                    .map_err(|error| error.to_string())?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// # Safety
/// `data` is the live boxed AppHandle owned by this subclass until WM_NCDESTROY.
unsafe extern "system" fn observe(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: register_owner transfers the non-null allocation to this observer's native lifetime.
    let app = unsafe { &*(data as *const AppHandle) };
    if message == WM_NCDESTROY {
        // SAFETY: Remove the observer on its owning thread; retain its context through nested
        // destruction messages, then free exactly the allocation registered for this HWND.
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(observe), id);
            let result = DefSubclassProc(hwnd, message, wparam, lparam);
            drop(Box::from_raw(data as *mut AppHandle));
            return result;
        }
    }
    if let Err(error) = handle(app, hwnd, message, wparam, lparam) {
        let _ = app.emit("shelf-error", error);
    }
    if message == WM_SIZING {
        LRESULT(1)
    } else {
        // SAFETY: Preserve the native procedure and original message arguments for other messages.
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }
}

#[cfg(test)]
mod tests {
    use super::{manual_height, ResizeEdge};
    #[test]
    fn only_actual_vertical_or_corner_height_changes_create_intent() {
        assert_eq!(manual_height(236.0, 260.0, ResizeEdge::East), None);
        assert_eq!(manual_height(236.0, 236.4, ResizeEdge::South), None);
        assert_eq!(manual_height(236.0, 260.0, ResizeEdge::South), Some(260.0));
        assert_eq!(manual_height(236.0, 236.0, ResizeEdge::NorthWest), None);
        assert_eq!(
            manual_height(236.0, 300.0, ResizeEdge::SouthEast),
            Some(300.0)
        );
    }

    #[test]
    fn work_area_clamping_keeps_rect_inside_boundaries() {
        use super::calculate_work_area_clamp;
        assert_eq!(
            calculate_work_area_clamp(100, 100, 200, 200, 0, 0, 1920, 1080),
            (100, 100)
        );
        assert_eq!(
            calculate_work_area_clamp(1800, 100, 200, 200, 0, 0, 1920, 1080),
            (1720, 100)
        );
        assert_eq!(
            calculate_work_area_clamp(100, 1000, 200, 200, 0, 0, 1920, 1080),
            (100, 880)
        );
        assert_eq!(
            calculate_work_area_clamp(-50, -50, 200, 200, 0, 0, 1920, 1080),
            (0, 0)
        );
    }
}
