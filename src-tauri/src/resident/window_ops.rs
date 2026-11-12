//! Drop Shelf placement geometry and the native window operations abstraction.

use tauri::{Emitter, WebviewWindow};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST, MonitorFromPoint};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, GetDpiForWindow};
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetCursorPos, GetWindowRect, HWND_TOPMOST, IsWindowVisible, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SetWindowPos};

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
    fn show_without_activate(&self) -> Result<(), String>;
    fn hide_without_activate(&self) -> Result<(), String>;
    fn reassert_topmost(&self) -> Result<(), String>;
}

/// Production implementation of WindowOps using Win32 API and Tauri window handle.
pub(crate) struct NativeWindowOps<'a> {
    window: &'a WebviewWindow,
    target: Option<&'a crate::surfaces::edge_rail::DisplayMonitor>,
}

impl<'a> NativeWindowOps<'a> {
    pub(crate) fn new(window: &'a WebviewWindow) -> Self {
        Self {
            window,
            target: None,
        }
    }

    pub(super) fn on_monitor(window: &'a WebviewWindow, target: &'a crate::surfaces::edge_rail::DisplayMonitor) -> Self {
        Self {
            window,
            target: Some(target),
        }
    }
}

impl<'a> WindowOps for NativeWindowOps<'a> {
    fn get_cursor_and_monitor_work_area(&self) -> Result<((i32, i32), WorkArea, u32), String> {
        if let Some(target) = self.target {
            return Ok((
                (
                    target.work.left + target.work.width() / 2,
                    target.work.top + target.work.height() / 2,
                ),
                target.work,
                target.dpi,
            ));
        }
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
        self.hide_without_activate()
    }

    fn show_without_activate(&self) -> Result<(), String> {
        let hwnd = self.window.hwnd().map_err(|error| error.to_string())?;
        // SAFETY: The live Shelf HWND is shown in the topmost band without activation.
        unsafe {
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            )
        }
        .map_err(|error| format!("failed to show Shake Shelf without activation: {error}"))
    }

    fn hide_without_activate(&self) -> Result<(), String> {
        let hwnd = self.window.hwnd().map_err(|error| error.to_string())?;
        // SAFETY: The live Shelf HWND is hidden without activation or z-order changes.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            )
        }
        .map_err(|error| format!("failed to hide transient Shake Shelf: {error}"))?;
        // SAFETY: Querying visibility of the same live HWND verifies the native postcondition.
        if unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return Err("transient Shake Shelf remained visible after hide".into());
        }
        self.window.emit("shelf-hidden", ()).map_err(|error| error.to_string())?;
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
