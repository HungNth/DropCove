use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use tauri::{AppHandle, Manager, WebviewWindow};

use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::{
        HiDpi::{GetDpiForMonitor, GetDpiForWindow},
        WindowsAndMessaging::{
            GetClientRect, GetCursorPos, GetWindowRect, SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        },
    },
};

use crate::{manage_items, shelf::ShelfState};

const SHELF_WINDOW_LABEL: &str = "main";

/// Transient presentation state of the resident coordinator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Visibility {
    Hidden,
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Hidden
    }
}

/// Thread-safe coordinator state for the resident instance.
#[derive(Default)]
pub(crate) struct ResidentState {
    visibility: Mutex<Visibility>,
    showing: AtomicBool,
    exiting: AtomicBool,
}

impl ResidentState {
    pub(crate) fn visibility(&self) -> Result<Visibility, String> {
        self.visibility
            .lock()
            .map(|v| *v)
            .map_err(|_| "resident coordinator state lock is poisoned".to_owned())
    }

    fn set_visibility(&self, next: Visibility) -> Result<(), String> {
        let mut guard = self
            .visibility
            .lock()
            .map_err(|_| "resident coordinator state lock is poisoned".to_owned())?;
        *guard = next;
        Ok(())
    }
}

/// Native geometry calculation inputs and outcome for placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorkArea {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

impl WorkArea {
    pub(crate) fn width(&self) -> i32 {
        (self.right - self.left).max(0)
    }

    pub(crate) fn height(&self) -> i32 {
        (self.bottom - self.top).max(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PhysicalPlacement {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// Compute physical placement on a monitor work area given cursor position, monitor work area,
/// DPI scaling, and durable preferred logical dimensions.
///
/// Rules:
/// 1. Convert preferred logical dimensions to physical pixels using target monitor DPI:
///    physical = round(logical * dpi / 96.0).
/// 2. Clamp target physical dimensions to the monitor work area extents.
/// 3. Center the shelf at or near the cursor, clamped completely inside the work area bounds:
///    X in [work.left, work.right - physical_width]
///    Y in [work.top, work.bottom - physical_height]
/// 4. Temporary work-area clamping NEVER modifies or overwrites durable ShelfState sizing.
pub(crate) fn calculate_placement(
    cursor: (i32, i32),
    work_area: WorkArea,
    dpi: u32,
    preferred_logical_width: f64,
    preferred_logical_height: f64,
) -> PhysicalPlacement {
    let scale = (dpi as f64) / 96.0;
    let raw_phys_width = (preferred_logical_width * scale).round() as i32;
    let raw_phys_height = (preferred_logical_height * scale).round() as i32;

    let work_w = work_area.width();
    let work_h = work_area.height();

    let clamped_w = raw_phys_width.clamp(1, work_w);
    let clamped_h = raw_phys_height.clamp(1, work_h);

    // Center on cursor horizontally and vertically, then clamp to work area
    let target_x = cursor.0 - (clamped_w / 2);
    let target_y = cursor.1 - (clamped_h / 2);

    let max_x = work_area.right - clamped_w;
    let min_x = work_area.left;
    let clamped_x = if max_x >= min_x {
        target_x.clamp(min_x, max_x)
    } else {
        min_x
    };

    let max_y = work_area.bottom - clamped_h;
    let min_y = work_area.top;
    let clamped_y = if max_y >= min_y {
        target_y.clamp(min_y, max_y)
    } else {
        min_y
    };

    PhysicalPlacement {
        x: clamped_x,
        y: clamped_y,
        width: clamped_w as u32,
        height: clamped_h as u32,
    }
}

/// Trait abstracting native window operations for deterministic testing.
pub(crate) trait WindowOps {
    fn get_cursor_and_monitor_work_area(&self) -> Result<((i32, i32), WorkArea, u32), String>;
    fn frame_size(&self, dpi: u32) -> Result<(f64, f64), String>;
    fn apply_placement(&self, placement: PhysicalPlacement) -> Result<(), String>;
    fn show_and_activate(&self) -> Result<(), String>;
    fn hide_window(&self) -> Result<(), String>;
    fn reassert_topmost(&self) -> Result<(), String>;
}

/// Production implementation of WindowOps using Win32 API and Tauri window handle.
pub(crate) struct NativeWindowOps<'a> {
    window: &'a WebviewWindow,
}

impl<'a> NativeWindowOps<'a> {
    pub(crate) fn new(window: &'a WebviewWindow) -> Self {
        Self { window }
    }
}

impl<'a> WindowOps for NativeWindowOps<'a> {
    fn get_cursor_and_monitor_work_area(&self) -> Result<((i32, i32), WorkArea, u32), String> {
        let mut point = POINT::default();
        // SAFETY: Point is a valid stack pointer.
        unsafe {
            GetCursorPos(&mut point).map_err(|e| format!("failed to get cursor position: {e}"))?;
        }

        // SAFETY: Point contains valid coordinates.
        let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
        if monitor.0.is_null() {
            return Err("failed to determine monitor from cursor point".to_owned());
        }

        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        // SAFETY: mi is appropriately sized and initialized.
        let success = unsafe { GetMonitorInfoW(monitor, &mut mi).as_bool() };
        if !success {
            return Err("failed to query monitor info".to_owned());
        }

        let mut dpi_x = 96u32;
        let mut dpi_y = 96u32;
        // SAFETY: monitor is valid; query effective DPI.
        unsafe {
            GetDpiForMonitor(
                monitor,
                windows::Win32::UI::HiDpi::MDT_EFFECTIVE_DPI,
                &mut dpi_x,
                &mut dpi_y,
            )
        }
        .map_err(|error| format!("failed to query cursor monitor DPI: {error}"))?;
        if dpi_x == 0 {
            return Err("cursor monitor returned zero DPI".to_owned());
        }
        let dpi = dpi_x;

        let work = WorkArea {
            left: mi.rcWork.left,
            top: mi.rcWork.top,
            right: mi.rcWork.right,
            bottom: mi.rcWork.bottom,
        };

        Ok(((point.x, point.y), work, dpi))
    }

    fn frame_size(&self, target_dpi: u32) -> Result<(f64, f64), String> {
        let hwnd = self.window.hwnd().map_err(|error| error.to_string())?;
        let mut outer = RECT::default();
        let mut client = RECT::default();
        // SAFETY: HWND is owned and live; both rectangles are valid writable outputs.
        let dpi = unsafe {
            GetWindowRect(hwnd, &mut outer).map_err(|error| error.to_string())?;
            GetClientRect(hwnd, &mut client).map_err(|error| error.to_string())?;
            GetDpiForWindow(hwnd)
        };
        if dpi == 0 {
            return Err("could not read Shelf frame DPI".to_owned());
        }
        let ratio = f64::from(target_dpi) / f64::from(dpi);
        Ok((
            f64::from((outer.right - outer.left - client.right + client.left).max(0)) * ratio,
            f64::from((outer.bottom - outer.top - client.bottom + client.top).max(0)) * ratio,
        ))
    }

    fn apply_placement(&self, placement: PhysicalPlacement) -> Result<(), String> {
        let hwnd = self.window.hwnd().map_err(|error| error.to_string())?;
        // SAFETY: SetWindowPos applies the complete outer bounds on the owning UI thread.
        // Using native bounds avoids adding the non-client frame to an already-clamped size.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                placement.x,
                placement.y,
                placement.width as i32,
                placement.height as i32,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
        }
        .map_err(|error| format!("could not place Drop Shelf: {error}"))
    }

    fn show_and_activate(&self) -> Result<(), String> {
        self.window
            .show()
            .map_err(|e| format!("failed to show window: {e}"))?;
        self.window
            .set_focus()
            .map_err(|e| format!("failed to focus window: {e}"))?;
        Ok(())
    }

    fn hide_window(&self) -> Result<(), String> {
        self.window
            .hide()
            .map_err(|e| format!("failed to hide window: {e}"))?;
        Ok(())
    }

    fn reassert_topmost(&self) -> Result<(), String> {
        let hwnd_raw = self
            .window
            .hwnd()
            .map_err(|e| format!("failed to get HWND: {e}"))?;
        let hwnd = HWND(hwnd_raw.0 as _);
        // SAFETY: SetWindowPos with HWND_TOPMOST and SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE
        // brings window to top of topmost band without stealing user focus.
        unsafe {
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
            .map_err(|e| format!("failed to reassert topmost: {e}"))?;
        }
        Ok(())
    }
}

/// Pure coordinator show logic over abstract WindowOps, ShelfState dimensions, and ResidentState.
pub(crate) fn coordinate_show(
    ops: &dyn WindowOps,
    state: &ResidentState,
    preferred_width: f64,
    preferred_height: f64,
) -> Result<(), String> {
    // Native calls can re-enter the message pump; do not acknowledge unfinished activation.
    if state.showing.swap(true, Ordering::SeqCst) {
        return Err("Shelf activation is already in progress".to_owned());
    }

    struct Guard<'a>(&'a AtomicBool);
    impl<'a> Drop for Guard<'a> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }
    let _guard = Guard(&state.showing);

    let (cursor, work_area, dpi) = ops.get_cursor_and_monitor_work_area()?;
    if work_area.width() == 0 || work_area.height() == 0 || dpi == 0 {
        return Err("cursor monitor has invalid work area or DPI".to_owned());
    }
    let (frame_width, frame_height) = ops.frame_size(dpi)?;
    let scale = f64::from(dpi) / 96.0;
    let placement = calculate_placement(
        cursor,
        work_area,
        dpi,
        preferred_width + frame_width / scale,
        preferred_height + frame_height / scale,
    );

    ops.apply_placement(placement)?;
    ops.show_and_activate()?;
    state.set_visibility(Visibility::Visible)?;
    ops.reassert_topmost()
}

/// Pure coordinator hide logic over abstract WindowOps, dismiss popup, and ResidentState.
pub(crate) fn coordinate_hide(
    ops: &dyn WindowOps,
    state: &ResidentState,
    dismiss_popup: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    // Dismiss transient popups such as manage-items
    dismiss_popup()?;

    ops.hide_window()?;
    state.set_visibility(Visibility::Hidden)?;
    Ok(())
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
    if state.visibility()? == Visibility::Hidden {
        return Ok(());
    }
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    coordinate_repair(&NativeWindowOps::new(&window), state.inner())
}

pub(crate) fn publish_warning(app: &AppHandle, message: &str) -> Result<(), String> {
    crate::resident_tray::warning(app, message)
}

pub(crate) fn exit(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    let busy = app.state::<crate::windows_drag::DragSession>().is_active()
        || crate::shelf_resize::is_resizing(app)?
        || app.state::<ShelfState>().has_pending_operations();
    coordinate_exit(state.inner(), busy, || {
        manage_items::close(app)?;
        crate::windows_foreground::shutdown(app)?;
        crate::windows_drop::unregister(app);
        app.state::<ShelfState>()
            .close()
            .map_err(|error| error.to_string())?;
        crate::settings::shutdown(app)?;
        crate::resident_tray::shutdown(app)?;
        app.exit(0);
        Ok(())
    })
}

/// Public API: show shelf on cursor monitor using durable preferred sizing.
/// Always shows rather than toggles; repeated calls are idempotent and reposition to current cursor.
pub(crate) fn show_shelf(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or_else(|| "Drop Shelf window is missing".to_owned())?;

    let shelf_state = app.state::<ShelfState>();
    let snapshot = shelf_state
        .snapshot()
        .map_err(|e| format!("failed to get shelf snapshot: {e}"))?;

    let resident_state = app.state::<ResidentState>();
    let ops = NativeWindowOps::new(&window);

    coordinate_show(
        &ops,
        resident_state.inner(),
        snapshot.preferred_width,
        snapshot.height,
    )
}

/// Public API: hide shelf without terminating residency or modifying durable references.
/// Transitions `Visible` to `Hidden`.
pub(crate) fn hide_shelf(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or_else(|| "Drop Shelf window is missing".to_owned())?;

    let resident_state = app.state::<ResidentState>();
    let ops = NativeWindowOps::new(&window);

    let app_handle = app.clone();
    coordinate_hide(&ops, resident_state.inner(), move || {
        manage_items::close(&app_handle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockWindowOps {
        cursor: (i32, i32),
        work_area: WorkArea,
        dpi: u32,
        frame: (f64, f64),
        placement: RefCell<Option<PhysicalPlacement>>,
        is_visible: RefCell<bool>,
        has_focus: RefCell<bool>,
        topmost_reasserted: RefCell<bool>,
        fail_placement: bool,
        fail_show: bool,
        fail_hide: bool,
    }

    impl MockWindowOps {
        pub(crate) fn new(cursor: (i32, i32), work_area: WorkArea, dpi: u32) -> Self {
            Self {
                cursor,
                work_area,
                dpi,
                frame: (0.0, 0.0),
                placement: RefCell::new(None),
                is_visible: RefCell::new(false),
                has_focus: RefCell::new(false),
                topmost_reasserted: RefCell::new(false),
                fail_placement: false,
                fail_show: false,
                fail_hide: false,
            }
        }
    }
    impl WindowOps for MockWindowOps {
        fn get_cursor_and_monitor_work_area(&self) -> Result<((i32, i32), WorkArea, u32), String> {
            Ok((self.cursor, self.work_area, self.dpi))
        }

        fn frame_size(&self, _dpi: u32) -> Result<(f64, f64), String> {
            Ok(self.frame)
        }

        fn apply_placement(&self, placement: PhysicalPlacement) -> Result<(), String> {
            if self.fail_placement {
                return Err("mock placement failure".to_owned());
            }
            *self.placement.borrow_mut() = Some(placement);
            Ok(())
        }

        fn show_and_activate(&self) -> Result<(), String> {
            if self.fail_show {
                return Err("mock show failure".to_owned());
            }
            *self.is_visible.borrow_mut() = true;
            *self.has_focus.borrow_mut() = true;
            Ok(())
        }

        fn hide_window(&self) -> Result<(), String> {
            if self.fail_hide {
                return Err("mock hide failure".to_owned());
            }
            *self.is_visible.borrow_mut() = false;
            *self.has_focus.borrow_mut() = false;
            Ok(())
        }

        fn reassert_topmost(&self) -> Result<(), String> {
            *self.topmost_reasserted.borrow_mut() = true;
            Ok(())
        }
    }

    #[test]
    fn foreground_changes_never_show_hidden_shelf_or_reactivate_visible_shelf() {
        let state = ResidentState::default();
        let ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );
        coordinate_repair(&ops, &state).unwrap();
        assert!(!*ops.is_visible.borrow());
        assert!(!*ops.topmost_reasserted.borrow());
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        *ops.has_focus.borrow_mut() = false;
        coordinate_repair(&ops, &state).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert!(!*ops.has_focus.borrow());
        coordinate_hide(&ops, &state, || Ok(())).unwrap();
        *ops.topmost_reasserted.borrow_mut() = false;
        coordinate_repair(&ops, &state).unwrap();
        assert!(!*ops.topmost_reasserted.borrow());
    }

    #[test]
    fn exit_waits_for_work_and_failed_teardown_can_be_retried() {
        let state = ResidentState::default();
        state.set_visibility(Visibility::Visible).unwrap();
        assert!(coordinate_exit(&state, true, || panic!("busy exit disposed state")).is_err());
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert!(coordinate_exit(&state, false, || Err("teardown failed".to_owned())).is_err());
        assert!(!state.exiting.load(Ordering::Acquire));
        coordinate_exit(&state, false, || Ok(())).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        coordinate_exit(&state, false, || panic!("successful exit was repeated")).unwrap();
    }

    #[test]
    fn activation_clamps_outer_frame_without_shrinking_preferred_client_size() {
        let state = ResidentState::default();
        let mut ops = MockWindowOps::new(
            (1919, 1031),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1032,
            },
            96,
        );
        ops.frame = (16.0, 9.0);
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        assert_eq!(
            *ops.placement.borrow(),
            Some(PhysicalPlacement {
                x: 1724,
                y: 843,
                width: 196,
                height: 189,
            })
        );
    }

    #[test]
    fn placement_scales_dpi_and_centers_on_cursor() {
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        // 96 DPI: scale = 1.0. Logical (200, 200) -> Physical (200, 200)
        let placement = calculate_placement((500, 500), work_area, 96, 200.0, 200.0);
        assert_eq!(
            placement,
            PhysicalPlacement {
                x: 400, // 500 - 100
                y: 400, // 500 - 100
                width: 200,
                height: 200,
            }
        );

        // 144 DPI (150%): Logical (200, 200) -> Physical (300, 300)
        let placement_hi = calculate_placement((500, 500), work_area, 144, 200.0, 200.0);
        assert_eq!(
            placement_hi,
            PhysicalPlacement {
                x: 350, // 500 - 150
                y: 350, // 500 - 150
                width: 300,
                height: 300,
            }
        );
    }

    #[test]
    fn placement_clamps_to_work_area_boundaries() {
        let work_area = WorkArea {
            left: 100,
            top: 50,
            right: 1000,
            bottom: 800,
        };

        // Cursor at far top-left (0, 0): clamped to work area top-left (100, 50)
        let placement_min = calculate_placement((0, 0), work_area, 96, 200.0, 200.0);
        assert_eq!(
            placement_min,
            PhysicalPlacement {
                x: 100,
                y: 50,
                width: 200,
                height: 200,
            }
        );

        // Cursor at far bottom-right (2000, 2000): clamped so window is fully inside (800, 600)
        let placement_max = calculate_placement((2000, 2000), work_area, 96, 200.0, 200.0);
        assert_eq!(
            placement_max,
            PhysicalPlacement {
                x: 800, // right (1000) - width (200)
                y: 600, // bottom (800) - height (200)
                width: 200,
                height: 200,
            }
        );
    }

    #[test]
    fn placement_clamps_oversized_preferred_dimension_to_work_area() {
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 400,
            bottom: 300,
        };
        // Preferred dimensions 600x500 exceed work area 400x300
        let placement = calculate_placement((200, 150), work_area, 96, 600.0, 500.0);
        assert_eq!(
            placement,
            PhysicalPlacement {
                x: 0,
                y: 0,
                width: 400,
                height: 300,
            }
        );
    }

    #[test]
    fn repeat_show_is_idempotent_and_repositions_to_new_cursor() {
        let state = ResidentState::default();
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);

        let mut ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );

        // First show
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert_eq!(
            *ops.placement.borrow(),
            Some(PhysicalPlacement {
                x: 310,
                y: 310,
                width: 180,
                height: 180,
            })
        );
        assert!(*ops.is_visible.borrow());
        assert!(*ops.topmost_reasserted.borrow());

        // Repeat show with moved cursor (e.g. secondary launch or tray click)
        ops.cursor = (800, 600);
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert_eq!(
            *ops.placement.borrow(),
            Some(PhysicalPlacement {
                x: 710,
                y: 510,
                width: 180,
                height: 180,
            })
        );
    }

    #[test]
    fn hide_retains_coordinator_and_dismisses_popup() {
        let state = ResidentState::default();
        let ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );

        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);

        let popup_dismissed = RefCell::new(false);
        coordinate_hide(&ops, &state, || {
            *popup_dismissed.borrow_mut() = true;
            Ok(())
        })
        .unwrap();

        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        assert!(!*ops.is_visible.borrow());
        assert!(*popup_dismissed.borrow());
    }

    #[test]
    fn failed_window_placement_returns_error_and_does_not_mark_visible() {
        let state = ResidentState::default();
        let mut ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );
        ops.fail_placement = true;

        let result = coordinate_show(&ops, &state, 180.0, 180.0);
        assert!(result.is_err());
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        assert!(!*ops.is_visible.borrow());
    }

    #[test]
    fn failed_window_show_returns_error_and_does_not_mark_visible() {
        let state = ResidentState::default();
        let mut ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );
        ops.fail_show = true;

        let result = coordinate_show(&ops, &state, 180.0, 180.0);
        assert!(result.is_err());
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
    }

    #[test]
    fn failed_window_hide_returns_error_and_does_not_mark_hidden() {
        let state = ResidentState::default();
        let mut ops = MockWindowOps::new(
            (400, 400),
            WorkArea {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            96,
        );

        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);

        ops.fail_hide = true;
        let result = coordinate_hide(&ops, &state, || Ok(()));
        assert!(result.is_err());
        // Coordinator visibility remains Visible because native window hide failed
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
    }
}
