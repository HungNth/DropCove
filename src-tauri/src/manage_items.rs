use std::sync::{Mutex, MutexGuard};

use crate::{
    shelf::{ShelfBatch, ShelfState},
    windows_drag::DragSession,
};
use serde::Deserialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    Window, WindowEvent,
};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller4;
use windows::core::Interface;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, MA_NOACTIVATE,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
            WM_MOUSEACTIVATE, WM_NCDESTROY, WM_SHOWWINDOW, WS_EX_NOACTIVATE,
        },
    },
};

const LABEL: &str = "manage-items";
const WIDTH: f64 = 280.0;
const MAX_HEIGHT: f64 = 480.0;
const MARGIN: f64 = 16.0;
const OWNER_SUBCLASS: usize = 0xDC02;
const POPUP_SUBCLASS: usize = 0xDC03;

#[derive(Clone, Copy, Deserialize)]
pub(crate) struct AnchorRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Default)]
struct PopupState {
    batch_id: Option<String>,
    keyboard: bool,
    pending_close: bool,
    origin_label: Option<String>,
}
#[derive(Default)]
pub(crate) struct ManageItemsState(Mutex<PopupState>);

impl ManageItemsState {
    fn lock(&self) -> Result<MutexGuard<'_, PopupState>, String> {
        self.0
            .lock()
            .map_err(|_| "management state lock is poisoned".to_owned())
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn placement(anchor: Rect, size: (f64, f64), work: Rect) -> (f64, f64) {
    let (width, height) = size;
    let margin_x = MARGIN.min(((work.width - width) / 2.0).max(0.0));
    let margin_y = MARGIN.min(((work.height - height) / 2.0).max(0.0));
    let min_x = work.x + margin_x;
    let max_x = (work.x + work.width - width - margin_x).max(min_x);
    let min_y = work.y + margin_y;
    let max_y = (work.y + work.height - height - margin_y).max(min_y);
    let candidates = [
        (anchor.x + anchor.width, anchor.y),
        (anchor.x - width, anchor.y),
        (anchor.x, anchor.y + anchor.height),
        (anchor.x, anchor.y - height),
    ];
    let (x, y) = candidates
        .into_iter()
        .find(|(x, y)| *x >= min_x && *x <= max_x && *y >= min_y && *y <= max_y)
        .unwrap_or(candidates[0]);
    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

pub(crate) fn register_owner(owner: &WebviewWindow) -> Result<(), String> {
    let hwnd = owner.hwnd().map_err(|error| error.to_string())?;
    let context = Box::into_raw(Box::new(owner.app_handle().clone()));
    // SAFETY: Setup runs on the native UI thread. The subclass owns this AppHandle allocation
    // until WM_NCDESTROY removes the subclass and frees it; failure frees it immediately.
    let installed =
        unsafe { SetWindowSubclass(hwnd, Some(owner_message), OWNER_SUBCLASS, context as usize) };
    if !installed.as_bool() {
        // SAFETY: Installation failed, so no callback can access the allocation.
        unsafe {
            drop(Box::from_raw(context));
        }
        return Err("could not observe Drop Shelf window lifetime".to_owned());
    }
    Ok(())
}

/// # Safety
/// `data` is the live boxed AppHandle installed by register_owner, owned until WM_NCDESTROY.
unsafe extern "system" fn owner_message(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: Clone the AppHandle before calling into resident functions to prevent use-after-free
    // in case a reentrant message (e.g. WM_NCDESTROY) deallocates data.
    let app = unsafe { (*(data as *const AppHandle)).clone() };
    if (message == WM_SHOWWINDOW && wparam.0 == 0) || message == WM_NCDESTROY {
        if let Err(error) = close(&app) {
            eprintln!("management owner cleanup failed: {error}");
        }
    }
    if message == WM_NCDESTROY {
        // SAFETY: This is the owning UI thread's final window callback. Remove our observer before
        // delegating destruction; keep its context alive through all nested destruction messages.
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(owner_message), subclass);
            let result = DefSubclassProc(hwnd, message, wparam, lparam);
            drop(Box::from_raw(data as *mut AppHandle));
            return result;
        }
    }
    // SAFETY: Unhandled messages retain the original native window procedure and arguments.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

/// # Safety
/// Installed on the Manage Items popup HWND when opened from Edge Rail to prevent activation.
/// Forwards all other native window messages to the subclass chain.
unsafe extern "system" fn popup_message(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    subclass: usize,
    _: usize,
) -> LRESULT {
    if message == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    if message == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(popup_message), subclass);
        }
    }
    // SAFETY: Forward unhandled messages to default subclass procedure.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

pub(crate) fn active_batch(app: &AppHandle) -> Result<Option<String>, String> {
    Ok(app.state::<ManageItemsState>().lock()?.batch_id.clone())
}

pub(crate) fn is_keyboard_activated(app: &AppHandle) -> Result<bool, String> {
    Ok(app.state::<ManageItemsState>().lock()?.keyboard)
}

pub(crate) fn open(
    app: &AppHandle,
    batch: &ShelfBatch,
    anchor: AnchorRect,
    keyboard: bool,
    origin: &str,
) -> Result<(), String> {
    if ![anchor.x, anchor.y, anchor.width, anchor.height]
        .into_iter()
        .all(f64::is_finite)
        || anchor.width <= 0.0
        || anchor.height <= 0.0
    {
        return Err("invalid management anchor bounds".to_owned());
    }
    if app.state::<DragSession>().is_active() {
        return Err("cannot replace management during native drag".to_owned());
    }
    if origin != "main" && origin != "edge-rail" {
        return Err("unsupported Manage Items origin window".to_owned());
    }
    let owner_label = origin;
    let owner = app
        .get_webview_window(owner_label)
        .ok_or_else(|| format!("{owner_label} window is missing"))?;
    let scale = owner.scale_factor().map_err(|error| error.to_string())?;
    let position = owner
        .inner_position()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(scale);
    let monitor = owner
        .current_monitor()
        .map_err(|error| error.to_string())?
        .ok_or("Drop Shelf monitor is missing")?;
    let bounds = monitor.work_area();
    let work_position = bounds.position.to_logical::<f64>(scale);
    let work_size = bounds.size.to_logical::<f64>(scale);
    if work_size.width < WIDTH || work_size.height <= 0.0 {
        return Err("monitor work area cannot contain Manage Items".to_owned());
    }
    let work = Rect {
        x: work_position.x,
        y: work_position.y,
        width: work_size.width,
        height: work_size.height,
    };
    let height = (64.0 + batch.items.len() as f64 * 76.0)
        .min(MAX_HEIGHT)
        .min(work.height);
    let screen_anchor = Rect {
        x: position.x + anchor.x,
        y: position.y + anchor.y,
        width: anchor.width,
        height: anchor.height,
    };
    let (x, y) = placement(screen_anchor, (WIDTH, height), work);

    let popup = if let Some(popup) = app.get_webview_window(LABEL) {
        popup
            .set_size(LogicalSize::new(WIDTH, height))
            .map_err(|error| error.to_string())?;
        // Final placement uses the recalculated native outer frame below.
        popup
    } else {
        let popup = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
            .title("Manage Items — DropCove")
            .inner_size(WIDTH, height)
            .position(x, y)
            .resizable(false)
            .decorations(false)
            .skip_taskbar(true)
            .disable_drag_drop_handler()
            .visible(false)
            .focused(false)
            .always_on_top(true)
            .owner(&owner)
            .map_err(|error| error.to_string())?
            .build()
            .map_err(|error| error.to_string())?;
        let (sender, receiver) = std::sync::mpsc::channel();
        popup
            .with_webview(move |webview| {
                // SAFETY: This callback executes on the owning WebView thread; the COM controller
                // stays live through the synchronous call. Management is not a storage intake surface.
                let result = unsafe {
                    webview
                        .controller()
                        .cast::<ICoreWebView2Controller4>()
                        .and_then(|controller| controller.SetAllowExternalDrop(false))
                }
                .map_err(|error| error.to_string());
                let _ = sender.send(result);
            })
            .map_err(|error| error.to_string())?;
        receiver.recv().map_err(|error| error.to_string())??;
        popup
    };
    {
        let state = app.state::<ManageItemsState>();
        let mut state = state.lock()?;
        state.batch_id = Some(batch.id.clone());
        state.keyboard = keyboard;
        state.pending_close = false;
        state.origin_label = Some(owner_label.to_owned());
    }
    if owner_label == "edge-rail" {
        app.state::<crate::resident::ResidentState>()
            .set_rail_transient_reason(crate::resident::RailTransientReason::ManageItems, true)?;
    }
    let (setup_sender, setup_receiver) = std::sync::mpsc::channel();
    let setup_popup = popup.clone();
    let rail_origin = owner_label == "edge-rail";
    app.run_on_main_thread(move || {
        let setup_result = (|| {
            let hwnd = setup_popup.hwnd().map_err(|error| error.to_string())?;
            // SAFETY: This HWND belongs to the live managed popup. Hidden Tao creation leaves the
            // nonclient frame unrecalculated; FRAMECHANGED applies its undecorated layout before showing.
            // NOACTIVATE preserves keyboard focus while keeping normal pointer activation available.
            unsafe {
                if rail_origin {
                    let extended = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, extended | WS_EX_NOACTIVATE.0 as isize);
                    let _ = RemoveWindowSubclass(hwnd, Some(popup_message), POPUP_SUBCLASS);
                    if !SetWindowSubclass(hwnd, Some(popup_message), POPUP_SUBCLASS, 0).as_bool() {
                        return Err(
                            "could not install no-activate guard on Manage Items popup".to_owned()
                        );
                    }
                } else {
                    let extended = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, extended & !(WS_EX_NOACTIVATE.0 as isize));
                    let _ = RemoveWindowSubclass(hwnd, Some(popup_message), POPUP_SUBCLASS);
                }
                SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                )
                .map_err(|error| error.to_string())?;
            }
            Ok(hwnd)
        })();
        let _ = setup_sender.send(setup_result.map(|hwnd| hwnd.0 as isize));
    })
    .map_err(|error| error.to_string())?;
    let setup_result = setup_receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map(|hwnd| HWND(hwnd as *mut _));
    let hwnd = match setup_result {
        Ok(hwnd) => hwnd,
        Err(error) => {
            if owner_label == "edge-rail" {
                let _ = app
                    .state::<crate::resident::ResidentState>()
                    .set_rail_transient_reason(
                        crate::resident::RailTransientReason::ManageItems,
                        false,
                    );
            }
            return Err(error);
        }
    };
    let show_result = (|| {
        let outer = popup
            .outer_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(scale);
        let inner = popup
            .inner_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(scale);
        if outer.height > work.height {
            popup
                .set_size(LogicalSize::new(
                    WIDTH,
                    (work.height - (outer.height - inner.height)).max(1.0),
                ))
                .map_err(|error| error.to_string())?;
        }
        let outer = popup
            .outer_size()
            .map_err(|error| error.to_string())?
            .to_logical::<f64>(scale);
        let (x, y) = placement(screen_anchor, (outer.width, outer.height), work);
        // SAFETY: The live owned HWND is moved/shown in physical screen coordinates without
        // activation. Placement bounds include its native border, not just the 280-pixel client.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                (x * scale).round() as i32,
                (y * scale).round() as i32,
                0,
                0,
                if keyboard {
                    SWP_NOSIZE | SWP_SHOWWINDOW
                } else {
                    SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW
                },
            )
        }
        .map_err(|error| error.to_string())?;
        if keyboard {
            popup.set_focus().map_err(|error| error.to_string())?;
        }
        popup
            .emit("manage-items-opened", keyboard)
            .map_err(|error| error.to_string())
    })();
    match show_result {
        Ok(()) => Ok(()),
        Err(error) => {
            if owner_label == "edge-rail" {
                let _ = app
                    .state::<crate::resident::ResidentState>()
                    .set_rail_transient_reason(
                        crate::resident::RailTransientReason::ManageItems,
                        false,
                    );
            }
            Err(error)
        }
    }
}

pub(crate) fn close_explicit(app: &AppHandle) -> Result<(), String> {
    if app.state::<DragSession>().is_active() {
        return Ok(());
    }
    let (batch_id, origin_label) = {
        let state = app.state::<ManageItemsState>();
        let state = state.lock()?;
        (state.batch_id.clone(), state.origin_label.clone())
    };
    let Some(batch_id) = batch_id else {
        return Ok(());
    };
    close(app)?;
    let origin = origin_label.as_deref().unwrap_or("main");
    if origin == "main" {
        if let Some(owner) = app.get_webview_window("main") {
            owner.set_focus().map_err(|error| error.to_string())?;
            owner
                .emit("manage-items-closed", batch_id)
                .map_err(|error| error.to_string())?;
        }
    } else if let Some(owner) = app.get_webview_window("edge-rail") {
        owner
            .emit("manage-items-closed", batch_id)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn close(app: &AppHandle) -> Result<(), String> {
    if app.state::<DragSession>().is_active() {
        app.state::<ManageItemsState>().lock()?.pending_close = true;
        return Ok(());
    }
    if let Some(popup) = app.get_webview_window(LABEL) {
        popup.hide().map_err(|error| error.to_string())?;
        popup.close().map_err(|error| error.to_string())?;
    }
    {
        let state = app.state::<ManageItemsState>();
        let mut state = state.lock()?;
        state.batch_id = None;
        state.keyboard = false;
        state.pending_close = false;
        state.origin_label = None;
    }
    if let Ok(Some(remaining)) = app
        .state::<crate::resident::ResidentState>()
        .set_rail_transient_reason(crate::resident::RailTransientReason::ManageItems, false)
    {
        let _ = crate::edge_rail::arm_rail_timer(app, remaining as u32);
    }
    app.emit("shelf-changed", ())
        .map_err(|error| error.to_string())
}

pub(crate) fn synchronize(app: &AppHandle) -> Result<(), String> {
    let (batch_id, pending) = {
        let state = app.state::<ManageItemsState>();
        let state = state.lock()?;
        (state.batch_id.clone(), state.pending_close)
    };
    if pending {
        close(app)?;
    } else if let Some(batch_id) = batch_id {
        let snapshot = app
            .state::<ShelfState>()
            .snapshot()
            .map_err(|error| error.to_string())?;
        if !snapshot.batches.iter().any(|batch| batch.id == batch_id) {
            close(app)?;
        }
    }
    app.emit("shelf-changed", ())
        .map_err(|error| error.to_string())
}

pub(crate) fn on_window_event(window: &Window, event: &WindowEvent) {
    let app = window.app_handle();
    let result = if (window.label() == "main" || window.label() == "edge-rail")
        && matches!(
            event,
            WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
        ) {
        close(app)
    } else if window.label() == LABEL {
        match event {
            WindowEvent::CloseRequested { api, .. } if app.state::<DragSession>().is_active() => {
                api.prevent_close();
                Ok(())
            }
            WindowEvent::CloseRequested { .. } => close_explicit(app),
            WindowEvent::Destroyed => match app.state::<ManageItemsState>().lock() {
                Ok(mut state) => {
                    state.batch_id = None;
                    state.keyboard = false;
                    state.pending_close = false;
                    let origin = state.origin_label.take();
                    if origin.as_deref() == Some("edge-rail") {
                        let _ = app
                            .state::<crate::resident::ResidentState>()
                            .set_rail_transient_reason(
                                crate::resident::RailTransientReason::ManageItems,
                                false,
                            );
                    }
                    Ok(())
                }
                Err(error) => Err(error),
            },
            _ => Ok(()),
        }
    } else {
        Ok(())
    };
    if let Err(error) = result {
        eprintln!("management window cleanup failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{placement, Rect};

    #[test]
    fn management_placement_uses_complete_candidate_bounds_then_clamps() {
        let work = Rect {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 700.0,
        };
        let size = (280.0, 250.0);
        assert_eq!(
            placement(
                Rect {
                    x: 100.0,
                    y: 100.0,
                    width: 180.0,
                    height: 80.0
                },
                size,
                work
            ),
            (280.0, 100.0)
        );
        assert_eq!(
            placement(
                Rect {
                    x: 720.0,
                    y: 100.0,
                    width: 160.0,
                    height: 80.0
                },
                size,
                work
            ),
            (440.0, 100.0)
        );
        let narrow = Rect {
            width: 500.0,
            ..work
        };
        assert_eq!(
            placement(
                Rect {
                    x: 120.0,
                    y: 100.0,
                    width: 160.0,
                    height: 80.0
                },
                size,
                narrow
            ),
            (120.0, 180.0)
        );
        assert_eq!(
            placement(
                Rect {
                    x: 120.0,
                    y: 450.0,
                    width: 160.0,
                    height: 80.0
                },
                size,
                narrow
            ),
            (120.0, 200.0)
        );
        assert_eq!(
            placement(
                Rect {
                    x: 890.0,
                    y: 690.0,
                    width: 180.0,
                    height: 80.0
                },
                size,
                work
            ),
            (604.0, 434.0)
        );
    }
}
