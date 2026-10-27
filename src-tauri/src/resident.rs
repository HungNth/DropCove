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

use crate::{edge_rail, manage_items, settings_store::Settings, shelf::ShelfState};

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

pub(crate) const RAIL_WIDTH: f64 = 280.0;
pub(crate) const RAIL_HANDLE_WIDTH: f64 = 16.0;
pub(crate) const RAIL_MIN_HEIGHT: f64 = 280.0;
pub(crate) const RAIL_MAX_HEIGHT: f64 = 642.0;
pub(crate) const RAIL_EXPAND_DELAY_MS: u64 = 200;
pub(crate) const RAIL_COLLAPSE_DELAY_MS: u64 = 300;
static MONOTONIC_START: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);

pub(crate) fn now_monotonic_ms() -> u64 {
    std::time::Instant::now()
        .duration_since(*MONOTONIC_START)
        .as_millis() as u64
}
pub(crate) fn calculate_rail_height(batch_count: usize) -> f64 {
    let height = 52.0 * (batch_count as f64) + 70.0;
    height.clamp(RAIL_MIN_HEIGHT, RAIL_MAX_HEIGHT)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RailPresentation {
    pub(crate) expanded: bool,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) expand_due_ms: Option<u64>,
    pub(crate) collapse_due_ms: Option<u64>,
}

impl Default for RailPresentation {
    fn default() -> Self {
        Self {
            expanded: false,
            width: RAIL_HANDLE_WIDTH,
            height: RAIL_MIN_HEIGHT,
            expand_due_ms: None,
            collapse_due_ms: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RailTransientReason {
    ManageItems = 1 << 0,
    ClearConfirmation = 1 << 1,
    BatchDragConfirmation = 1 << 2,
    ActiveDrag = 1 << 3,
    ItemActions = 1 << 4,
    SelectionActions = 1 << 5,
}

#[derive(Clone, Debug)]
struct ShakeSession {
    id: u64,
    previous_visibility: Visibility,
    previous_placement: Option<edge_rail::Placement>,
    previous_rail: RailPresentation,
    drag_inside: bool,
    drop_pending: bool,
    released: bool,
}

enum ShakeDropResolution {
    None,
    Restore(ShakeSession),
    Promote,
}

#[derive(Default)]
struct Presentation {
    visibility: Visibility,
    rail_placement: Option<edge_rail::Placement>,
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

impl ResidentState {
    pub(crate) fn visibility(&self) -> Result<Visibility, String> {
        self.presentation
            .lock()
            .map(|v| v.visibility)
            .map_err(|_| "resident coordinator state lock is poisoned".to_owned())
    }

    fn placement(&self) -> Result<Option<edge_rail::Placement>, String> {
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

    fn shake_candidate(&self) -> Result<Option<ShakeSession>, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        if guard.shake_session.is_some()
            || guard.visibility == Visibility::Visible
            || self.exiting.load(Ordering::Acquire)
        {
            return Ok(None);
        }
        let id = guard.next_shake_id;
        guard.next_shake_id = guard.next_shake_id.wrapping_add(1);
        Ok(Some(ShakeSession {
            id,
            previous_visibility: guard.visibility,
            previous_placement: guard.rail_placement.clone(),
            previous_rail: guard.rail,
            drag_inside: false,
            drop_pending: false,
            released: false,
        }))
    }

    fn commit_shake(&self, session: ShakeSession) -> Result<(), String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        if guard.shake_session.is_some() || guard.visibility != session.previous_visibility {
            return Err("Shelf presentation changed while Shake-to-open was starting".into());
        }
        guard.visibility = Visibility::Visible;
        guard.rail_placement = None;
        guard.rail = RailPresentation::default();
        guard.transient_holds &= RailTransientReason::ActiveDrag as u8;
        guard.fullscreen_occluded = false;
        guard.shake_session = Some(session);
        Ok(())
    }

    fn mark_shake_drag_inside(&self, inside: bool) -> Result<bool, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        let Some(session) = guard.shake_session.as_mut() else {
            return Ok(false);
        };
        session.drag_inside = inside;
        Ok(session.released && !session.drag_inside && !session.drop_pending)
    }

    fn mark_shake_drop_pending(&self) -> Result<Option<u64>, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        let Some(session) = guard.shake_session.as_mut() else {
            return Ok(None);
        };
        session.drag_inside = false;
        session.drop_pending = true;
        Ok(Some(session.id))
    }

    fn mark_shake_released(&self) -> Result<bool, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        let Some(session) = guard.shake_session.as_mut() else {
            return Ok(false);
        };
        session.released = true;
        Ok(!session.drag_inside && !session.drop_pending)
    }

    fn resolve_shake_drop(
        &self,
        session_id: Option<u64>,
        accepted: bool,
    ) -> Result<ShakeDropResolution, String> {
        let guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        let Some(session) = guard
            .shake_session
            .as_ref()
            .filter(|session| Some(session.id) == session_id)
        else {
            return Ok(ShakeDropResolution::None);
        };
        if accepted {
            return Ok(ShakeDropResolution::Promote);
        }
        Ok(ShakeDropResolution::Restore(session.clone()))
    }

    fn finish_shake_restore(&self, session: &ShakeSession) -> Result<(), String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        if !guard
            .shake_session
            .as_ref()
            .is_some_and(|current| current.id == session.id)
        {
            return Ok(());
        }
        guard.visibility = session.previous_visibility;
        guard.rail_placement = session.previous_placement.clone();
        guard.rail = session.previous_rail;
        guard.fullscreen_occluded = false;
        guard.shake_session = None;
        Ok(())
    }

    fn finish_shake_promote(&self, session_id: Option<u64>) -> Result<(), String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        if !guard
            .shake_session
            .as_ref()
            .is_some_and(|current| Some(current.id) == session_id)
        {
            return Ok(());
        }
        guard.visibility = Visibility::Visible;
        guard.rail_placement = None;
        guard.shake_session = None;
        Ok(())
    }

    fn active_shake_session(&self) -> Result<Option<ShakeSession>, String> {
        self.presentation
            .lock()
            .map(|guard| guard.shake_session.clone())
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }

    pub(crate) fn set_rail_transient_reason(
        &self,
        reason: RailTransientReason,
        active: bool,
    ) -> Result<Option<u64>, String> {
        let now = now_monotonic_ms();
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned".to_owned())?;
        let mask = reason as u8;
        if active {
            guard.transient_holds |= mask;
            guard.rail.collapse_due_ms = None;
            Ok(None)
        } else {
            guard.transient_holds &= !mask;
            if guard.visibility == Visibility::EdgeDocked
                && guard.rail.expanded
                && guard.transient_holds == 0
                && !guard.pointer_inside
                && guard.rail.collapse_due_ms.is_none()
            {
                guard.rail.collapse_due_ms = Some(now + RAIL_COLLAPSE_DELAY_MS);
                Ok(Some(RAIL_COLLAPSE_DELAY_MS))
            } else {
                Ok(None)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn has_rail_transient_hold(&self) -> Result<bool, String> {
        self.presentation
            .lock()
            .map(|p| p.transient_holds != 0)
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }

    pub(crate) fn set_rail_drag_active(&self, active: bool) -> Result<Option<u64>, String> {
        self.set_rail_transient_reason(RailTransientReason::ActiveDrag, active)
    }

    #[cfg(test)]
    pub(crate) fn rail_drag_active(&self) -> Result<bool, String> {
        self.presentation
            .lock()
            .map(|p| (p.transient_holds & (RailTransientReason::ActiveDrag as u8)) != 0)
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }

    #[cfg(test)]
    pub(crate) fn is_fullscreen_occluded(&self) -> Result<bool, String> {
        self.presentation
            .lock()
            .map(|p| p.fullscreen_occluded)
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }

    #[cfg(test)]
    pub(crate) fn has_transient_reason(&self, reason: RailTransientReason) -> Result<bool, String> {
        self.presentation
            .lock()
            .map(|p| (p.transient_holds & (reason as u8)) != 0)
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }
    pub(crate) fn rail_presentation(&self) -> Result<RailPresentation, String> {
        self.presentation
            .lock()
            .map(|p| p.rail)
            .map_err(|_| String::from("resident presentation lock is poisoned"))
    }

    pub(crate) fn set_rail_presentation(&self, rail: RailPresentation) -> Result<(), String> {
        self.presentation
            .lock()
            .map(|mut p| {
                p.rail = rail;
            })
            .map_err(|_| String::from("resident presentation lock is poisoned"))
    }

    pub(crate) fn on_rail_pointer_enter(&self, now_ms: u64) -> Result<RailPresentation, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| String::from("resident presentation lock is poisoned"))?;
        guard.pointer_inside = true;
        if guard.visibility != Visibility::EdgeDocked {
            return Ok(guard.rail);
        }
        guard.rail.collapse_due_ms = None;
        if !guard.rail.expanded && guard.rail.expand_due_ms.is_none() {
            guard.rail.expand_due_ms = Some(now_ms + RAIL_EXPAND_DELAY_MS);
        }
        Ok(guard.rail)
    }

    pub(crate) fn on_rail_button_pointer_enter(&self) -> Result<RailPresentation, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| String::from("resident presentation lock is poisoned"))?;
        guard.pointer_inside = true;
        if guard.visibility != Visibility::EdgeDocked {
            return Ok(guard.rail);
        }
        guard.rail.collapse_due_ms = None;
        if !guard.rail.expanded {
            guard.rail.expand_due_ms = None;
        }
        Ok(guard.rail)
    }

    pub(crate) fn on_rail_body_pointer_enter(&self, now_ms: u64) -> Result<RailPresentation, String> {
        self.on_rail_pointer_enter(now_ms)
    }

    #[cfg(test)]
    pub(crate) fn on_rail_drag_enter(&self, now_ms: u64) -> Result<RailPresentation, String> {
        self.on_rail_pointer_enter(now_ms)
    }

    pub(crate) fn reconcile_rail_pointer(
        &self,
        now_ms: u64,
        region: RailPointerRegion,
    ) -> Result<RailPresentation, String> {
        match region {
            RailPointerRegion::Outside => self.on_rail_pointer_leave(now_ms),
            RailPointerRegion::Button => self.on_rail_button_pointer_enter(),
            RailPointerRegion::Body => self.on_rail_body_pointer_enter(now_ms),
        }
    }

    pub(crate) fn on_rail_pointer_leave(&self, now_ms: u64) -> Result<RailPresentation, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| String::from("resident presentation lock is poisoned"))?;
        guard.pointer_inside = false;
        if guard.visibility != Visibility::EdgeDocked {
            return Ok(guard.rail);
        }
        guard.rail.expand_due_ms = None;
        if guard.rail.expanded && guard.transient_holds == 0 && guard.rail.collapse_due_ms.is_none()
        {
            guard.rail.collapse_due_ms = Some(now_ms + RAIL_COLLAPSE_DELAY_MS);
        }
        Ok(guard.rail)
    }

    pub(crate) fn plan_tick_rail(
        &self,
        now_ms: u64,
        batch_count: usize,
    ) -> Result<Option<RailPresentation>, String> {
        let guard = self
            .presentation
            .lock()
            .map_err(|_| String::from("resident presentation lock is poisoned"))?;
        if guard.visibility != Visibility::EdgeDocked || batch_count == 0 {
            return Ok(None);
        }
        let mut next = guard.rail;
        let mut changed = false;
        if let Some(due) = next.expand_due_ms {
            if now_ms >= due {
                next.expand_due_ms = None;
                next.expanded = true;
                next.width = RAIL_WIDTH;
                next.height = calculate_rail_height(batch_count);
                changed = true;
            }
        }
        if let Some(due) = next.collapse_due_ms {
            if guard.transient_holds != 0 {
                next.collapse_due_ms = None;
                changed = true;
            } else if now_ms >= due {
                next.collapse_due_ms = None;
                next.expanded = false;
                next.width = RAIL_HANDLE_WIDTH;
                next.height = RAIL_MIN_HEIGHT;
                changed = true;
            }
        }
        if changed {
            Ok(Some(next))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn tick_rail(
        &self,
        now_ms: u64,
        batch_count: usize,
    ) -> Result<Option<RailPresentation>, String> {
        let planned = self.plan_tick_rail(now_ms, batch_count)?;
        if let Some(next) = planned {
            let mut guard = self
                .presentation
                .lock()
                .map_err(|_| String::from("resident presentation lock is poisoned"))?;
            guard.rail = next;
            Ok(Some(next))
        } else {
            let mut guard = self
                .presentation
                .lock()
                .map_err(|_| String::from("resident presentation lock is poisoned"))?;
            if guard.visibility != Visibility::EdgeDocked || batch_count == 0 {
                guard.rail.expand_due_ms = None;
                guard.rail.collapse_due_ms = None;
            }
            Ok(None)
        }
    }

    pub(crate) fn update_rail_content(
        &self,
        batch_count: usize,
    ) -> Result<Option<RailPresentation>, String> {
        let mut guard = self
            .presentation
            .lock()
            .map_err(|_| String::from("resident presentation lock is poisoned"))?;
        if guard.visibility != Visibility::EdgeDocked {
            return Ok(None);
        }
        if batch_count == 0 {
            guard.rail.expand_due_ms = None;
            guard.rail.collapse_due_ms = None;
            return Ok(None);
        }
        if guard.rail.expanded {
            let next_height = calculate_rail_height(batch_count);
            if next_height > guard.rail.height {
                guard.rail.height = next_height;
                return Ok(Some(guard.rail));
            }
        }
        Ok(None)
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
    fn show_without_activate(&self) -> Result<(), String>;
    fn hide_without_activate(&self) -> Result<(), String>;
    fn reassert_topmost(&self) -> Result<(), String>;
}

/// Production implementation of WindowOps using Win32 API and Tauri window handle.
pub(crate) struct NativeWindowOps<'a> {
    window: &'a WebviewWindow,
    target: Option<&'a edge_rail::DisplayMonitor>,
}

impl<'a> NativeWindowOps<'a> {
    pub(crate) fn new(window: &'a WebviewWindow) -> Self {
        Self {
            window,
            target: None,
        }
    }

    fn on_monitor(window: &'a WebviewWindow, target: &'a edge_rail::DisplayMonitor) -> Self {
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

fn place_shelf(
    ops: &dyn WindowOps,
    preferred_width: f64,
    preferred_height: f64,
) -> Result<(), String> {
    let (cursor, work_area, dpi) = ops.get_cursor_and_monitor_work_area()?;
    if work_area.width() == 0 || work_area.height() == 0 || dpi == 0 {
        return Err("cursor monitor has invalid work area or DPI".to_owned());
    }
    let (frame_width, frame_height) = ops.frame_size(dpi)?;
    let scale = f64::from(dpi) / 96.0;
    ops.apply_placement(calculate_placement(
        cursor,
        work_area,
        dpi,
        preferred_width + frame_width / scale,
        preferred_height + frame_height / scale,
    ))
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

    place_shelf(ops, preferred_width, preferred_height)?;
    ops.show_and_activate()?;
    state.set_visibility(Visibility::Visible)?;
    ops.reassert_topmost()
}

/// Coordinator transition when showing shelf from a previous visibility state.
/// If transitioning from EdgeDocked, closes rail transients and manages popup.
pub(crate) fn coordinate_docked_to_shelf(
    previous: Visibility,
    mut close_rail_transients: impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    if previous == Visibility::EdgeDocked {
        close_rail_transients()?;
    }
    Ok(())
}

fn coordinate_shake_begin(
    ops: &dyn WindowOps,
    state: &ResidentState,
    preferred_width: f64,
    preferred_height: f64,
    hide_rail: impl FnOnce() -> Result<(), String>,
    restore_rail: impl FnOnce(&ShakeSession) -> Result<(), String>,
) -> Result<bool, String> {
    if state.showing.swap(true, Ordering::SeqCst) {
        return Err("Shelf activation is already in progress".to_owned());
    }
    struct Guard<'a>(&'a AtomicBool);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }
    let _guard = Guard(&state.showing);
    let Some(session) = state.shake_candidate()? else {
        return Ok(false);
    };
    if session.previous_visibility == Visibility::EdgeDocked {
        hide_rail()?;
    }
    let result = place_shelf(ops, preferred_width, preferred_height)
        .and_then(|()| ops.show_without_activate())
        .and_then(|()| ops.reassert_topmost())
        .and_then(|()| state.commit_shake(session.clone()));
    if let Err(error) = result {
        let hide = ops.hide_without_activate();
        let restore = if session.previous_visibility == Visibility::EdgeDocked {
            restore_rail(&session)
        } else {
            Ok(())
        };
        if hide.is_err() || restore.is_err() {
            return Err(format!(
                "Shake-to-open failed: {error}; restore incomplete: shelf={hide:?}, rail={restore:?}"
            ));
        }
        return Err(error);
    }
    Ok(true)
}

fn coordinate_shake_restore(
    ops: &dyn WindowOps,
    state: &ResidentState,
    session: &ShakeSession,
    show_rail: impl FnOnce(&ShakeSession) -> Result<(), String>,
) -> Result<(), String> {
    ops.hide_without_activate()?;
    if session.previous_visibility == Visibility::EdgeDocked {
        if let Err(error) = show_rail(session) {
            let show = ops.show_without_activate();
            return Err(format!(
                "could not restore Edge Rail after Shake-to-open: {error}; transient Shelf restore={show:?}"
            ));
        }
    }
    state.finish_shake_restore(session)
}

fn coordinate_shake_promote(
    ops: &dyn WindowOps,
    state: &ResidentState,
    session_id: Option<u64>,
) -> Result<(), String> {
    let show = ops.show_and_activate();
    state.finish_shake_promote(session_id)?;
    show
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

pub(crate) fn coordinate_fullscreen(
    state: &ResidentState,
    occluded: bool,
    show_over_fullscreen: bool,
    mut hide_or_show_rail: impl FnMut() -> Result<(), String>,
    close_transients: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let mut guard = state
        .presentation
        .lock()
        .map_err(|_| "resident presentation lock is poisoned".to_owned())?;

    if show_over_fullscreen {
        return Ok(());
    }

    if occluded {
        if !guard.fullscreen_occluded {
            guard.fullscreen_occluded = true;
            // Clear non-drag transient holds (keep active drag)
            guard.transient_holds &= RailTransientReason::ActiveDrag as u8;
            guard.rail = RailPresentation::default();
            drop(guard);
            hide_or_show_rail()?;
            close_transients()?;
        }
    } else if guard.fullscreen_occluded {
        guard.fullscreen_occluded = false;
        guard.rail = RailPresentation::default();
        drop(guard);
        hide_or_show_rail()?;
    }
    Ok(())
}

pub(crate) fn evaluate_fullscreen_coverage(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::EdgeDocked {
        return Ok(());
    }
    let placement = match rail_placement(app)? {
        Some(p) => p,
        None => return Ok(()),
    };
    let settings = crate::settings::active(app)?;
    let occluded = is_monitor_occluded_by_foreground(&placement.monitor.monitor_rect);
    coordinate_fullscreen(
        state.inner(),
        occluded,
        settings.show_rail_over_fullscreen,
        || {
            if occluded && !settings.show_rail_over_fullscreen {
                edge_rail::hide(app)
            } else {
                let rail = state.rail_presentation()?;
                edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded)
            }
        },
        || {
            manage_items::close(app)?;
            let _ = app.emit("rail-transients-closed", ());
            Ok(())
        },
    )
}

fn is_monitor_occluded_by_foreground(monitor_rect: &WorkArea) -> bool {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    };

    // SAFETY: GetForegroundWindow returns the current foreground HWND or NULL.
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.0.is_null() {
        return false;
    }
    let mut pid = 0;
    // SAFETY: GetWindowThreadProcessId inspects foreground HWND safely.
    unsafe { GetWindowThreadProcessId(foreground, Some(&mut pid)) };
    if pid == unsafe { GetCurrentProcessId() } {
        return false;
    }
    // SAFETY: Standard window visibility and minimized state inspection.
    if !unsafe { IsWindowVisible(foreground) }.as_bool()
        || unsafe { IsIconic(foreground) }.as_bool()
    {
        return false;
    }
    let mut rect = RECT::default();
    // SAFETY: GetWindowRect fills the RECT buffer safely.
    if unsafe { GetWindowRect(foreground, &mut rect) }.is_err() {
        return false;
    }
    let window_rect = WorkArea {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    edge_rail::rect_covers_monitor(&window_rect, monitor_rect)
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
    crate::resident_tray::warning(app, message)
}

pub(crate) fn exit(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    let busy = app.state::<crate::windows_drag::DragSession>().is_active()
        || crate::shelf_resize::is_resizing(app)?
        || app.state::<ShelfState>().has_pending_operations();
    coordinate_exit(state.inner(), busy, || {
        crate::clear_confirmation::shutdown(app)?;
        crate::about::shutdown(app)?;
        manage_items::close(app)?;
        edge_rail::hide(app)?;
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
    show_shelf_at(app, None)
}

fn show_shelf_at(
    app: &AppHandle,
    monitor: Option<&edge_rail::DisplayMonitor>,
) -> Result<(), String> {
    if crate::clear_confirmation::is_modal_active(app) {
        crate::clear_confirmation::request(app, "main")?;
        return Ok(());
    }
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let snapshot = app
        .state::<ShelfState>()
        .snapshot()
        .map_err(|e| e.to_string())?;
    let state = app.state::<ResidentState>();
    let previous = state.visibility()?;
    let previous_rail = rail_placement(app)?;
    state
        .had_content
        .store(!snapshot.batches.is_empty(), Ordering::Release);
    coordinate_docked_to_shelf(previous, || {
        manage_items::close(app)?;
        let _ = app.emit("rail-transients-closed", ());
        Ok(())
    })?;
    edge_rail::hide(app)?;
    let ops = match monitor {
        Some(monitor) => NativeWindowOps::on_monitor(&window, monitor),
        None => NativeWindowOps::new(&window),
    };
    if let Err(error) = coordinate_show(&ops, &state, snapshot.preferred_width, snapshot.height) {
        if previous == Visibility::EdgeDocked {
            ops.hide_window()?;
            state.set_visibility(previous)?;
            restore_rail(app, previous_rail.as_ref())?;
        }
        return Err(error);
    }
    if previous != Visibility::Visible {
        let visual_state = app.state::<crate::windows_visuals::ShellVisualState>();
        let new_gen = visual_state.0.advance_generation(crate::windows_visuals::SurfaceId::Shelf);
        use tauri::Emitter;
        let _ = app.emit(
            "visual-generation-changed",
            crate::windows_visuals::GenerationChangedPayload {
                surface: crate::windows_visuals::SurfaceId::Shelf,
                generation: new_gen,
            },
        );
    }
    Ok(())
}

fn show_saved_shake_rail(app: &AppHandle, session: &ShakeSession) -> Result<(), String> {
    let placement = session
        .previous_placement
        .as_ref()
        .ok_or("saved EdgeDocked Shake session has no rail placement")?;
    edge_rail::show(
        app,
        placement,
        session.previous_rail.width,
        session.previous_rail.height,
        session.previous_rail.expanded,
    )
}

fn restore_shake_on_window_thread(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    let Some(session) = state.active_shake_session()? else {
        return Ok(());
    };
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    coordinate_shake_restore(
        &NativeWindowOps::new(&window),
        state.inner(),
        &session,
        |session| show_saved_shake_rail(app, session),
    )?;
    evaluate_fullscreen_coverage(app)
}

pub(crate) fn begin_shake(app: &AppHandle) -> Result<(), String> {
    if app.state::<crate::windows_drag::DragSession>().is_active() {
        return Ok(());
    }
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let snapshot = app
        .state::<ShelfState>()
        .snapshot()
        .map_err(|error| error.to_string())?;
    let state = app.state::<ResidentState>();
    let started = coordinate_shake_begin(
        &NativeWindowOps::new(&window),
        state.inner(),
        snapshot.preferred_width,
        snapshot.height,
        || {
            manage_items::close(app)?;
            let _ = app.emit("rail-transients-closed", ());
            edge_rail::hide(app)
        },
        |session| show_saved_shake_rail(app, session),
    )?;
    if started {
        state
            .had_content
            .store(!snapshot.batches.is_empty(), Ordering::Release);
    }
    Ok(())
}

pub(crate) fn release_shake(app: &AppHandle) -> Result<(), String> {
    if app.state::<ResidentState>().mark_shake_released()? {
        restore_shake_on_window_thread(app)?;
    }
    Ok(())
}

pub(crate) fn shake_drag_inside(app: &AppHandle, inside: bool) -> Result<(), String> {
    if app
        .state::<ResidentState>()
        .mark_shake_drag_inside(inside)?
    {
        restore_shake_on_window_thread(app)?;
    }
    Ok(())
}

pub(crate) fn shake_drop_started(app: &AppHandle) -> Result<Option<u64>, String> {
    app.state::<ResidentState>().mark_shake_drop_pending()
}

fn complete_shake_drop_on_window_thread(
    app: &AppHandle,
    session_id: Option<u64>,
    accepted: bool,
) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    match state.resolve_shake_drop(session_id, accepted)? {
        ShakeDropResolution::None => Ok(()),
        ShakeDropResolution::Restore(session) => {
            let window = app
                .get_webview_window(SHELF_WINDOW_LABEL)
                .ok_or("Drop Shelf window is missing")?;
            coordinate_shake_restore(
                &NativeWindowOps::new(&window),
                state.inner(),
                &session,
                |session| show_saved_shake_rail(app, session),
            )?;
            evaluate_fullscreen_coverage(app)
        }
        ShakeDropResolution::Promote => {
            let window = app
                .get_webview_window(SHELF_WINDOW_LABEL)
                .ok_or("Drop Shelf window is missing")?;
            coordinate_shake_promote(&NativeWindowOps::new(&window), state.inner(), session_id)
        }
    }
}

pub(crate) fn complete_shake_drop(
    app: &AppHandle,
    session_id: Option<u64>,
    accepted: bool,
) -> Result<(), String> {
    use windows::Win32::{
        System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    // SAFETY: The live Shelf HWND identifies the UI thread that owns presentation changes.
    if unsafe { GetWindowThreadProcessId(hwnd, None) == GetCurrentThreadId() } {
        return complete_shake_drop_on_window_thread(app, session_id, accepted);
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let app_handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = sender.send(complete_shake_drop_on_window_thread(
            &app_handle,
            session_id,
            accepted,
        ));
    })
    .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())?
}

pub(crate) fn cancel_shake(app: &AppHandle) -> Result<(), String> {
    restore_shake_on_window_thread(app)
}

pub(crate) fn rail_placement(app: &AppHandle) -> Result<Option<edge_rail::Placement>, String> {
    app.state::<ResidentState>().placement()
}
fn set_rail_placement(
    app: &AppHandle,
    placement: Option<edge_rail::Placement>,
) -> Result<(), String> {
    app.state::<ResidentState>()
        .presentation
        .lock()
        .map_err(|_| "resident presentation lock is poisoned")?
        .rail_placement = placement;
    Ok(())
}
pub(crate) fn restore_rail(
    app: &AppHandle,
    placement: Option<&edge_rail::Placement>,
) -> Result<(), String> {
    if app.state::<ResidentState>().visibility()? == Visibility::EdgeDocked {
        let rail = app.state::<ResidentState>().rail_presentation()?;
        match placement {
            Some(placement) => {
                edge_rail::show(app, placement, rail.width, rail.height, rail.expanded)?
            }
            None => edge_rail::hide(app)?,
        }
    }
    set_rail_placement(app, placement.cloned())
}
pub(crate) fn coordinate_apply_rail_settings(
    state: &ResidentState,
    shelf: &ShelfState,
    placement: &edge_rail::Placement,
    rail_monitor_id_setting: &str,
    show_native: impl FnOnce(&edge_rail::Placement) -> Result<(), String>,
    restore_native: impl FnOnce(Option<&edge_rail::Placement>) -> Result<(), String>,
) -> Result<(), String> {
    let prior_placement = state.placement()?;
    let prior_remembered = shelf.rail_monitor().map_err(|e| e.to_string())?;
    let should_persist = rail_monitor_id_setting.is_empty()
        && (prior_placement
            .as_ref()
            .map(|prev| prev.monitor.id != placement.monitor.id)
            .unwrap_or(true)
            || prior_remembered.as_deref() != Some(&placement.monitor.id));

    let mut persisted_changed = false;
    let result = (|| {
        show_native(placement)?;
        if should_persist {
            shelf
                .remember_rail_monitor(Some(&placement.monitor.id))
                .map_err(|e| e.to_string())?;
            persisted_changed = true;
        }
        let mut presentation = state
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned")?;
        presentation.rail_placement = Some(placement.clone());
        Ok::<(), String>(())
    })();

    if let Err(error) = result {
        let rollback_native = restore_native(prior_placement.as_ref());
        let rollback_persist = if persisted_changed {
            shelf
                .remember_rail_monitor(prior_remembered.as_deref())
                .map_err(|e| e.to_string())
        } else {
            Ok(())
        };
        let rollback_live = {
            match state.presentation.lock() {
                Ok(mut pres) => {
                    pres.rail_placement = prior_placement.clone();
                    Ok(())
                }
                Err(_) => Err("resident presentation lock is poisoned".to_string()),
            }
        };
        let mut rollback_errors = Vec::new();
        if let Err(e) = rollback_native {
            rollback_errors.push(format!("native restore failed: {e}"));
        }
        if let Err(e) = rollback_persist {
            rollback_errors.push(format!("persistence restore failed: {e}"));
        }
        if let Err(e) = rollback_live {
            rollback_errors.push(format!("placement restore failed: {e}"));
        }
        if !rollback_errors.is_empty() {
            return Err(format!(
                "{error}; rollback incomplete: {}",
                rollback_errors.join(", ")
            ));
        }
        return Err(error);
    }
    Ok(())
}
pub(crate) fn apply_rail_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::EdgeDocked {
        return Ok(());
    }
    let placement = edge_rail::placement(app, settings)?;
    let shelf = app.state::<ShelfState>();
    let rail = state.rail_presentation()?;
    coordinate_apply_rail_settings(
        &state,
        &shelf,
        &placement,
        &settings.rail_monitor_id,
        |p| edge_rail::show(app, p, rail.width, rail.height, rail.expanded),
        |prev| match prev {
            Some(prev_p) => edge_rail::show(app, prev_p, rail.width, rail.height, rail.expanded),
            None => edge_rail::hide(app),
        },
    )
}
pub(crate) fn update_rail_content(app: &AppHandle, batch_count: usize) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::EdgeDocked {
        return Ok(());
    }
    let prior_rail = state.rail_presentation()?;
    if let Some(rail) = state.update_rail_content(batch_count)? {
        let placement = match rail_placement(app)? {
            Some(p) => p,
            None => {
                state.set_rail_presentation(prior_rail)?;
                return Err(String::from("docked rail has no placement"));
            }
        };
        if let Err(error) = edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded)
        {
            state.set_rail_presentation(prior_rail)?;
            return Err(error);
        }
    }
    Ok(())
}
pub(crate) fn tick_rail(app: &AppHandle) -> Result<(), String> {
    let now = now_monotonic_ms();
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::EdgeDocked {
        return Ok(());
    }
    let shelf = app.state::<ShelfState>();
    let batch_count = shelf
        .snapshot()
        .map_err(|e| format!("cannot tick rail without shelf state: {e}"))?
        .batches
        .len();
    if let Some(next) = state.plan_tick_rail(now, batch_count)? {
        let placement = match rail_placement(app)? {
            Some(p) => p,
            None => {
                return Err(String::from("docked rail has no placement"));
            }
        };
        edge_rail::show(app, &placement, next.width, next.height, next.expanded)?;
        state.set_rail_presentation(next)?;
        edge_rail::cancel_rail_timer(app);
    } else {
        state.tick_rail(now, batch_count)?;
        let rail = state.rail_presentation()?;
        if let Some(due) = rail.expand_due_ms.or(rail.collapse_due_ms) {
            let remaining = (due.saturating_sub(now)).max(1) as u32;
            edge_rail::arm_rail_timer(app, remaining)?;
        } else {
        }
    }
    Ok(())
}
pub(crate) fn reflow_rail(app: &AppHandle) -> Result<(), String> {
    apply_rail_settings(app, &crate::settings::active(app)?)
}
pub(crate) fn open_shelf_from_rail(app: &AppHandle) -> Result<(), String> {
    if app.state::<ResidentState>().visibility()? != Visibility::EdgeDocked {
        return Err("Rail Handle is not docked".into());
    }
    let placement = match rail_placement(app) {
        Ok(Some(placement)) => placement,
        other => {
            coordinate_docked_to_shelf(Visibility::EdgeDocked, || {
                manage_items::close(app)?;
                let _ = app.emit("rail-transients-closed", ());
                Ok(())
            })?;
            return match other {
                Ok(None) => Err("docked rail has no placement".into()),
                Err(err) => Err(err),
                Ok(Some(_)) => unreachable!(),
            };
        }
    };
    show_shelf_at(app, Some(&placement.monitor))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RailPointerRegion {
    Outside,
    Button,
    Body,
}

pub(crate) fn classify_rail_pointer(
    pt_x: i32,
    pt_y: i32,
    rect_left: i32,
    rect_top: i32,
    rect_right: i32,
    rect_bottom: i32,
    scale: f64,
) -> RailPointerRegion {
    if pt_x < rect_left || pt_x >= rect_right || pt_y < rect_top || pt_y >= rect_bottom {
        return RailPointerRegion::Outside;
    }
    let button_bottom = rect_top + (16.0 * scale).round() as i32;
    if pt_y < button_bottom {
        RailPointerRegion::Button
    } else {
        RailPointerRegion::Body
    }
}

pub(crate) fn reconcile_rail_pointer_after_drag(app: &AppHandle) -> Result<(), String> {
    let window = match app.get_webview_window(edge_rail::LABEL) {
        Some(w) => w,
        None => return Ok(()),
    };
    let hwnd = match window.hwnd() {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };
    let mut pt = POINT::default();
    let cursor_ok = unsafe { GetCursorPos(&mut pt) }.is_ok();
    let mut rect = RECT::default();
    let rect_ok = unsafe { GetWindowRect(hwnd, &mut rect) }.is_ok();
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let scale = if dpi == 0 { 1.0 } else { f64::from(dpi) / 96.0 };

    let region = if cursor_ok && rect_ok {
        classify_rail_pointer(pt.x, pt.y, rect.left, rect.top, rect.right, rect.bottom, scale)
    } else {
        RailPointerRegion::Outside
    };

    let now = now_monotonic_ms();
    let state = app.state::<ResidentState>();
    let rail = state.reconcile_rail_pointer(now, region)?;
    if let Some(due) = rail.expand_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        let _ = edge_rail::arm_rail_timer(app, remaining);
    } else if let Some(due) = rail.collapse_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        let _ = edge_rail::arm_rail_timer(app, remaining);
    } else {
        edge_rail::cancel_rail_timer(app);
    }
    Ok(())
}

fn coordinate_dock(
    ops: &dyn WindowOps,
    state: &ResidentState,
    shelf: &ShelfState,
    placement: &edge_rail::Placement,
    show_rail: impl FnOnce() -> Result<(), String>,
    hide_rail: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let previous = shelf.rail_monitor().map_err(|e| e.to_string())?;
    let mut shelf_hidden = false;
    let result = (|| {
        show_rail()?;
        shelf
            .remember_rail_monitor(Some(&placement.monitor.id))
            .map_err(|e| e.to_string())?;
        ops.hide_window()?;
        shelf_hidden = true;
        let mut presentation = state
            .presentation
            .lock()
            .map_err(|_| "resident presentation lock is poisoned")?;
        presentation.visibility = Visibility::EdgeDocked;
        presentation.rail_placement = Some(placement.clone());
        presentation.shake_session = None;
        presentation.fullscreen_occluded = false;
        Ok::<(), String>(())
    })();
    if let Err(error) = result {
        let hide = hide_rail();
        let show = if shelf_hidden {
            ops.show_and_activate()
        } else {
            Ok(())
        };
        let restore = if shelf.rail_monitor().map_err(|e| e.to_string())? != previous {
            shelf
                .remember_rail_monitor(previous.as_deref())
                .map_err(|e| e.to_string())
        } else {
            Ok(())
        };
        if hide.is_err() || restore.is_err() || show.is_err() {
            return Err(format!("rail dismissal failed: {error}; restore incomplete: rail={hide:?}, monitor={restore:?}, shelf={show:?}"));
        }
        return Err(error);
    }
    Ok(())
}
pub(crate) fn dismiss_shelf(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ResidentState>();
    if state.visibility()? != Visibility::Visible {
        return Ok(());
    }
    crate::clear_confirmation::guard_interaction(app)?;
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let shelf = app.state::<ShelfState>();
    let snapshot = shelf.snapshot().map_err(|e| e.to_string())?;
    state
        .had_content
        .store(!snapshot.batches.is_empty(), Ordering::Release);
    manage_items::close(app)?;
    {
        let visual_state = app.state::<crate::windows_visuals::ShellVisualState>();
        let new_gen = visual_state.0.advance_generation(crate::windows_visuals::SurfaceId::Shelf);
        use tauri::Emitter;
        let _ = app.emit(
            "visual-generation-changed",
            crate::windows_visuals::GenerationChangedPayload {
                surface: crate::windows_visuals::SurfaceId::Shelf,
                generation: new_gen,
            },
        );
    }
    if snapshot.batches.is_empty() {
        edge_rail::hide(app)?;
        return coordinate_hide(&NativeWindowOps::new(&window), &state, || Ok(()));
    }
    let placement = edge_rail::placement_for_dismissal(app, &crate::settings::active(app)?)?;
    let rail = state.rail_presentation()?;
    coordinate_dock(
        &NativeWindowOps::new(&window),
        &state,
        &shelf,
        &placement,
        || edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded),
        || edge_rail::hide(app),
    )?;
    evaluate_fullscreen_coverage(app)
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
        manage_items::close(app)?;
        let _ = app.emit("rail-transients-closed", ());
        edge_rail::hide(app)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn docking_native_failure_restores_visible_shelf_and_remembered_monitor() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };
        let path = std::env::temp_dir().join(format!(
            "dc-dock-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let shelf = ShelfState::open(path.join("shelf.db")).unwrap();
        shelf.remember_rail_monitor(Some("original")).unwrap();
        let state = ResidentState::default();
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1032,
        };
        let placement = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "selected".into(),
                label: "selected".into(),
                device: "display".into(),
                monitor_rect: work,
                work,
                dpi: 96,
                primary: true,
            },
            edge: crate::settings_store::RailEdge::Right,
        };
        let mut ops = MockWindowOps::new((500, 500), work, 96);
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        ops.fail_hide = true;
        let rail_visible = RefCell::new(false);
        assert!(coordinate_dock(
            &ops,
            &state,
            &shelf,
            &placement,
            || {
                *rail_visible.borrow_mut() = true;
                Ok(())
            },
            || {
                *rail_visible.borrow_mut() = false;
                Ok(())
            }
        )
        .is_err());
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert!(state.placement().unwrap().is_none());
        assert!(!*rail_visible.borrow());
        assert_eq!(shelf.rail_monitor().unwrap().as_deref(), Some("original"));
        ops.fail_hide = false;
        coordinate_dock(&ops, &state, &shelf, &placement, || Ok(()), || Ok(())).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::EdgeDocked);
        assert_eq!(state.placement().unwrap().as_ref(), Some(&placement));
        assert_eq!(shelf.rail_monitor().unwrap().as_deref(), Some("selected"));
        coordinate_content(&state, true, || panic!("nonempty update hid the rail")).unwrap();
        assert!(coordinate_content(&state, false, || Err("native hide rejected".into())).is_err());
        assert_eq!(state.visibility().unwrap(), Visibility::EdgeDocked);
        coordinate_content(&state, false, || Ok(())).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        assert!(state.placement().unwrap().is_none());
        coordinate_content(&state, false, || panic!("empty no-op hid an intake window")).unwrap();
        shelf.close().unwrap();
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn rail_settings_failures_restore_native_and_persisted_placement() {
        use std::{
            fs,
            time::{SystemTime, UNIX_EPOCH},
        };
        let path = std::env::temp_dir().join(format!(
            "dc-dock-fail-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let shelf = ShelfState::open(path.join("shelf.db")).unwrap();
        shelf.remember_rail_monitor(Some("monitor_a")).unwrap();
        let state = ResidentState::default();
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1032,
        };
        let placement_a = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "monitor_a".into(),
                label: "monitor_a".into(),
                device: "display_a".into(),
                monitor_rect: work,
                work,
                dpi: 96,
                primary: true,
            },
            edge: crate::settings_store::RailEdge::Right,
        };
        let placement_b = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "monitor_b".into(),
                label: "monitor_b".into(),
                device: "display_b".into(),
                monitor_rect: work,
                work,
                dpi: 96,
                primary: false,
            },
            edge: crate::settings_store::RailEdge::Left,
        };

        // Setup docked state with placement_a
        let mut presentation = state.presentation.lock().unwrap();
        presentation.visibility = Visibility::EdgeDocked;
        presentation.rail_placement = Some(placement_a.clone());
        drop(presentation);

        let prev_placement = state.placement().unwrap();
        assert_eq!(prev_placement.as_ref(), Some(&placement_a));
        let prev_remembered = shelf.rail_monitor().unwrap();
        assert_eq!(prev_remembered.as_deref(), Some("monitor_a"));

        // Install trigger causing actual ShelfState write to fail during persistence
        shelf.install_test_rail_monitor_failure().unwrap();
        let shown = RefCell::new(Vec::new());
        let restored = RefCell::new(Vec::new());
        let err = coordinate_apply_rail_settings(
            &state,
            &shelf,
            &placement_b,
            "",
            |p| {
                shown.borrow_mut().push(p.monitor.id.clone());
                Ok(())
            },
            |prev| {
                restored.borrow_mut().push(prev.map(|p| p.monitor.id.clone()));
                Ok(())
            },
        );

        assert!(err.is_err(), "SQL trigger must reject monitor persistence");
        assert_eq!(shown.into_inner(), vec!["monitor_b"]);
        assert_eq!(restored.into_inner(), vec![Some("monitor_a".to_string())]);
        assert_eq!(state.placement().unwrap().as_ref(), Some(&placement_a));

        let native_monitor = RefCell::new(Some("monitor_a".to_owned()));
        let error = coordinate_apply_rail_settings(
            &state,
            &shelf,
            &placement_b,
            "",
            |p| {
                *native_monitor.borrow_mut() = Some(p.monitor.id.clone());
                Err("projection failed after native move".to_owned())
            },
            |previous| {
                *native_monitor.borrow_mut() = previous.map(|p| p.monitor.id.clone());
                Ok(())
            },
        )
        .unwrap_err();
        assert!(error.contains("projection failed after native move"));
        assert_eq!(native_monitor.into_inner().as_deref(), Some("monitor_a"));
        assert_eq!(state.placement().unwrap().as_ref(), Some(&placement_a));

        // Reopen database to verify durable persistence retention
        let reopened = ShelfState::open(path.join("shelf.db")).unwrap();
        assert_eq!(reopened.rail_monitor().unwrap().as_deref(), Some("monitor_a"));
        reopened.close().unwrap();
        shelf.close().unwrap();
        fs::remove_dir_all(path).unwrap();
    }

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

        fn show_without_activate(&self) -> Result<(), String> {
            if self.fail_show {
                return Err("mock show failure".to_owned());
            }
            *self.is_visible.borrow_mut() = true;
            *self.has_focus.borrow_mut() = false;
            Ok(())
        }

        fn hide_without_activate(&self) -> Result<(), String> {
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
    fn docked_to_shelf_closes_rail_transients_only_when_edge_docked() {
        let closed = RefCell::new(0);
        let mut on_close = || {
            *closed.borrow_mut() += 1;
            Ok(())
        };

        coordinate_docked_to_shelf(Visibility::Visible, &mut on_close).unwrap();
        assert_eq!(*closed.borrow(), 0, "visible shelf must not close rail transients");

        coordinate_docked_to_shelf(Visibility::Hidden, &mut on_close).unwrap();
        assert_eq!(*closed.borrow(), 0, "hidden shelf must not close rail transients");

        coordinate_docked_to_shelf(Visibility::EdgeDocked, &mut on_close).unwrap();
        assert_eq!(*closed.borrow(), 1, "edge docked shelf must close rail transients");
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

    #[test]
    fn rail_expansion_timers_height_and_empty_content_cancellation() {
        let state = ResidentState::default();
        state.set_visibility(Visibility::EdgeDocked).unwrap();

        let mut now = 1000u64;

        // Pointer hover requests expansion after 200 ms; leaving before deadline prevents expansion
        state.on_rail_pointer_enter(now).unwrap();
        now += 150;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(!state.rail_presentation().unwrap().expanded);

        state.on_rail_pointer_leave(now).unwrap();
        now += 100;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(!state.rail_presentation().unwrap().expanded);

        // Hovering through 200 ms expands the rail
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        let tick = state.tick_rail(now, 1).unwrap();
        assert!(tick.is_some());
        let current = state.rail_presentation().unwrap();
        assert!(current.expanded);
        assert_eq!(current.width, 280.0);
        assert_eq!(current.height, 280.0);

        // Pointer leave collapses after 300 ms; re-entry before deadline cancels collapse
        state.on_rail_pointer_leave(now).unwrap();
        now += 150;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(state.rail_presentation().unwrap().expanded);

        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(state.rail_presentation().unwrap().expanded);

        // Leaving through 300 ms collapses the rail
        state.on_rail_pointer_leave(now).unwrap();
        now += 300;
        let tick = state.tick_rail(now, 1).unwrap();
        assert!(tick.is_some());
        let current = state.rail_presentation().unwrap();
        assert!(!current.expanded);
        assert_eq!(current.width, 16.0);

        // Button enter never starts expansion and cancels pending body expansion
        state.on_rail_button_pointer_enter().unwrap();
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, None);
        now += 250;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(!state.rail_presentation().unwrap().expanded);

        // Moving from button into body starts 200 ms expansion delay
        state.on_rail_pointer_enter(now).unwrap();
        assert!(state.rail_presentation().unwrap().expand_due_ms.is_some());
        // Moving back to button before deadline cancels it
        now += 100;
        state.on_rail_button_pointer_enter().unwrap();
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, None);
        now += 200;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(!state.rail_presentation().unwrap().expanded);

        // Moving over button while already expanded does NOT collapse
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 1).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);
        state.on_rail_button_pointer_enter().unwrap();
        assert!(state.rail_presentation().unwrap().expanded);
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
        now += 350;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(state.rail_presentation().unwrap().expanded);
        // Leaving expanded rail arms 300 ms collapse delay
        state.on_rail_pointer_leave(now).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        // Re-entering directly via Open Shelf button cancels the pending collapse
        now += 150;
        state.on_rail_button_pointer_enter().unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
        now += 200;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(state.rail_presentation().unwrap().expanded);

        // Leaving again arms 300 ms collapse delay
        state.on_rail_pointer_leave(now).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        // Re-entering via expanded body/section also cancels the pending collapse
        now += 150;
        state.on_rail_body_pointer_enter(now).unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
        now += 200;
        let tick = state.tick_rail(now, 1).unwrap();
        assert_eq!(tick, None);
        assert!(state.rail_presentation().unwrap().expanded);
        // Finally collapse after leaving again
        state.on_rail_pointer_leave(now).unwrap();
        now += 300;
        state.tick_rail(now, 1).unwrap();
        assert!(!state.rail_presentation().unwrap().expanded);
        assert_eq!(state.rail_presentation().unwrap().width, 16.0);
        // Reconcile pointer after drag
        // 1. Cursor over button: cancels pending expansion
        state.on_rail_drag_enter(now).unwrap();
        assert!(state.rail_presentation().unwrap().expand_due_ms.is_some());
        state.reconcile_rail_pointer(now, RailPointerRegion::Button).unwrap();
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, None);
        now += 250;
        assert_eq!(state.tick_rail(now, 1).unwrap(), None);
        assert!(!state.rail_presentation().unwrap().expanded);

        // 2. Cursor over body: preserves/starts expansion
        state.reconcile_rail_pointer(now, RailPointerRegion::Body).unwrap();
        assert!(state.rail_presentation().unwrap().expand_due_ms.is_some());
        now += 200;
        assert!(state.tick_rail(now, 1).unwrap().is_some());
        assert!(state.rail_presentation().unwrap().expanded);

        // 3. Cursor outside: collapses
        state.reconcile_rail_pointer(now, RailPointerRegion::Outside).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        now += 300;
        assert!(state.tick_rail(now, 1).unwrap().is_some());
        assert!(!state.rail_presentation().unwrap().expanded);

        // Test geometry classifier
        // Rect: left=100, top=200, right=116, bottom=480 (16x280 at 1.0 scale)
        // Top button is 16px tall: top=200..216
        assert_eq!(classify_rail_pointer(99, 210, 100, 200, 116, 480, 1.0), RailPointerRegion::Outside);
        assert_eq!(classify_rail_pointer(116, 210, 100, 200, 116, 480, 1.0), RailPointerRegion::Outside);
        assert_eq!(classify_rail_pointer(110, 199, 100, 200, 116, 480, 1.0), RailPointerRegion::Outside);
        assert_eq!(classify_rail_pointer(110, 480, 100, 200, 116, 480, 1.0), RailPointerRegion::Outside);
        assert_eq!(classify_rail_pointer(110, 200, 100, 200, 116, 480, 1.0), RailPointerRegion::Button);
        assert_eq!(classify_rail_pointer(110, 215, 100, 200, 116, 480, 1.0), RailPointerRegion::Button);
        assert_eq!(classify_rail_pointer(110, 216, 100, 200, 116, 480, 1.0), RailPointerRegion::Body);
        assert_eq!(classify_rail_pointer(110, 479, 100, 200, 116, 480, 1.0), RailPointerRegion::Body);

        // Fractional DPI scaling (1.5x scale): top button is (16.0 * 1.5).round() = 24px: top=200..224
        // Rect: left=100, top=200, right=124, bottom=620 (24x420 at 1.5 scale)
        assert_eq!(classify_rail_pointer(110, 200, 100, 200, 124, 620, 1.5), RailPointerRegion::Button);
        assert_eq!(classify_rail_pointer(110, 223, 100, 200, 124, 620, 1.5), RailPointerRegion::Button);
        assert_eq!(classify_rail_pointer(110, 224, 100, 200, 124, 620, 1.5), RailPointerRegion::Body);
        assert_eq!(classify_rail_pointer(110, 619, 100, 200, 124, 620, 1.5), RailPointerRegion::Body);

        // Compact rows retain the minimum surface and clamp at the work-area ceiling.
        assert_eq!(calculate_rail_height(0), 280.0);
        assert_eq!(calculate_rail_height(1), 280.0);
        assert_eq!(calculate_rail_height(4), 280.0);
        assert_eq!(calculate_rail_height(5), 330.0);
        assert_eq!(calculate_rail_height(11), 642.0);
        assert_eq!(calculate_rail_height(12), 642.0);
        assert_eq!(calculate_rail_height(20), 642.0);

        // Removing content does not live-shrink expanded rail, next expansion recalculates
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 8).unwrap();
        let current = state.rail_presentation().unwrap();
        assert!(current.expanded);
        assert_eq!(current.height, 486.0);

        // Content updated while expanded
        let shrunk = state.update_rail_content(2).unwrap();
        assert_eq!(shrunk, None); // no live shrink
        assert_eq!(state.rail_presentation().unwrap().height, 486.0);

        // Collapse and re-expand recalculates height
        state.on_rail_pointer_leave(now).unwrap();
        now += 300;
        state.tick_rail(now, 2).unwrap();
        assert!(!state.rail_presentation().unwrap().expanded);

        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 2).unwrap();
        let current = state.rail_presentation().unwrap();
        assert!(current.expanded);
        assert_eq!(current.height, 280.0);

        // Removing final item cancels pending timers and hides rail immediately
        state.on_rail_pointer_leave(now).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        let hide_called = RefCell::new(false);
        coordinate_content(&state, false, || {
            *hide_called.borrow_mut() = true;
            Ok(())
        })
        .unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
        assert!(!state.rail_presentation().unwrap().expanded);
        assert!(*hide_called.borrow());
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
    }
    #[test]
    fn rail_sub_deadline_tick_does_not_expand_early_and_preserves_deadline_until_due() {
        let state = ResidentState::default();
        state.set_visibility(Visibility::EdgeDocked).unwrap();
        let now = 1000u64;
        state.on_rail_pointer_enter(now).unwrap();
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, Some(1200));

        // Sub-deadline callback (e.g. WM_TIMER arrives early at 1194ms < 1200ms)
        let early_tick = state.tick_rail(1194, 1).unwrap();
        assert_eq!(early_tick, None, "early callback before 200ms must NOT expand");
        assert!(!state.rail_presentation().unwrap().expanded);
        // The deadline remains pending so caller re-arms remaining duration
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, Some(1200));

        // Subsequent callback when deadline is reached (at 1200ms) DOES expand
        let due_tick = state.tick_rail(1200, 1).unwrap();
        assert!(due_tick.is_some(), "callback at or after deadline must expand");
        assert!(state.rail_presentation().unwrap().expanded);
        assert_eq!(state.rail_presentation().unwrap().expand_due_ms, None);

        // Collapse delay: pointer leave arms collapse for 1200 + 300 = 1500ms
        state.on_rail_pointer_leave(1200).unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, Some(1500));

        // Early callback before 300ms (e.g. at 1494ms < 1500ms) must NOT collapse
        let early_collapse = state.tick_rail(1494, 1).unwrap();
        assert_eq!(early_collapse, None, "early callback before 300ms must NOT collapse");
        assert!(state.rail_presentation().unwrap().expanded);
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, Some(1500));

        // Subsequent callback when collapse deadline is reached (at 1500ms) DOES collapse
        let due_collapse = state.tick_rail(1500, 1).unwrap();
        assert!(due_collapse.is_some(), "callback at or after deadline must collapse");
        assert!(!state.rail_presentation().unwrap().expanded);
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
    }

    #[test]
    fn rail_management_transient_hold_open_and_inflight_drag_ownership() {
        let state = ResidentState::default();
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1032,
        };
        let placement = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "test".into(),
                label: "test".into(),
                device: "display".into(),
                monitor_rect: work,
                work,
                dpi: 96,
                primary: true,
            },
            edge: crate::settings_store::RailEdge::Right,
        };
        let ops = MockWindowOps::new((500, 500), work, 96);
        coordinate_show(&ops, &state, 180.0, 180.0).unwrap();
        let path = std::env::temp_dir().join(format!("dc-hold-{}", std::process::id()));
        let shelf = ShelfState::open(path.join("shelf.db")).unwrap();
        coordinate_dock(&ops, &state, &shelf, &placement, || Ok(()), || Ok(())).unwrap();

        let mut now = 1000;
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 2).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);

        // Acquire multiple simultaneous transient holds (e.g. Manage Items + Clear confirmation)
        state
            .set_rail_transient_reason(RailTransientReason::ManageItems, true)
            .unwrap();
        state
            .set_rail_transient_reason(RailTransientReason::ClearConfirmation, true)
            .unwrap();
        state
            .set_rail_transient_reason(RailTransientReason::BatchDragConfirmation, true)
            .unwrap();
        state.set_rail_transient_reason(RailTransientReason::ItemActions, true).unwrap();
        assert!(state.has_rail_transient_hold().unwrap());

        // Releasing Clear confirmation while BatchDrag and Manage Items remain retains hold
        state
            .set_rail_transient_reason(RailTransientReason::ClearConfirmation, false)
            .unwrap();
        assert!(state.has_rail_transient_hold().unwrap());
        state
            .set_rail_transient_reason(RailTransientReason::BatchDragConfirmation, false)
            .unwrap();
        assert!(state.has_rail_transient_hold().unwrap());
        // Pointer leaves while Manage Items hold active: must NOT schedule collapse
        state.on_rail_pointer_leave(now).unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);

        assert_eq!(state.set_rail_transient_reason(RailTransientReason::ManageItems, false).unwrap(), None);
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);
        let remaining = state
            .set_rail_transient_reason(RailTransientReason::ItemActions, false)
            .unwrap();
        assert_eq!(remaining, Some(RAIL_COLLAPSE_DELAY_MS));
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());

        // Re-entering before collapse deadline cancels collapse
        state.on_rail_pointer_enter(now).unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);

        // Pointer leaves again: collapse scheduled
        state.on_rail_pointer_leave(now).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        let collapse_due = state.rail_presentation().unwrap().collapse_due_ms.unwrap();
        let tick = state.tick_rail(collapse_due, 2).unwrap();
        assert!(tick.is_some());
        assert!(!state.rail_presentation().unwrap().expanded);

        // Inflight OLE drag starts and finishes while pointer is outside
        state.on_rail_pointer_enter(now).unwrap();
        let expand_due = state.rail_presentation().unwrap().expand_due_ms.unwrap();
        state.tick_rail(expand_due, 2).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);
        state.set_rail_drag_active(true).unwrap();
        state.on_rail_pointer_leave(now).unwrap();
        assert_eq!(state.rail_presentation().unwrap().collapse_due_ms, None);

        // Drag finishes while pointer is outside (e.g. dropped in File Explorer): automatically arms collapse!
        let remaining = state.set_rail_drag_active(false).unwrap();
        assert_eq!(remaining, Some(RAIL_COLLAPSE_DELAY_MS));
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        let collapse_due = state.rail_presentation().unwrap().collapse_due_ms.unwrap();
        let tick = state.tick_rail(collapse_due, 2).unwrap();
        assert!(tick.is_some());
        assert!(!state.rail_presentation().unwrap().expanded);
        // Removing the final item hides the rail immediately regardless of holds
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 1).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);
        state
            .set_rail_transient_reason(RailTransientReason::ManageItems, true)
            .unwrap();

        let hide_called = RefCell::new(false);
        coordinate_content(&state, false, || {
            *hide_called.borrow_mut() = true;
            Ok(())
        })
        .unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        assert!(!state.rail_presentation().unwrap().expanded);
        assert!(!state.has_rail_transient_hold().unwrap());
        assert!(*hide_called.borrow());

        // Re-docking rail after hiding does not inherit stale holds; normal pointer leave collapses
        coordinate_dock(&ops, &state, &shelf, &placement, || Ok(()), || Ok(())).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::EdgeDocked);
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 2).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);
        state.on_rail_pointer_leave(now).unwrap();
        assert!(state.rail_presentation().unwrap().collapse_due_ms.is_some());
        now += 300;
        let tick = state.tick_rail(now, 2).unwrap();
        assert!(tick.is_some());
        assert!(!state.rail_presentation().unwrap().expanded);
        shelf.close().unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn fullscreen_coverage_evaluation_and_transient_dismissal_without_canceling_drag() {
        let state = ResidentState::default();
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1032,
        };
        let monitor_rect = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let other_monitor_rect = WorkArea {
            left: 1920,
            top: 0,
            right: 3840,
            bottom: 1080,
        };
        let placement = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "test".into(),
                label: "test".into(),
                device: "display".into(),
                monitor_rect,
                work,
                dpi: 96,
                primary: true,
            },
            edge: crate::settings_store::RailEdge::Right,
        };
        let ops = MockWindowOps::new((500, 500), work, 96);
        let path = std::env::temp_dir().join(format!("dc-fs-{}", std::process::id()));
        let shelf = ShelfState::open(path.join("shelf.db")).unwrap();
        coordinate_dock(&ops, &state, &shelf, &placement, || Ok(()), || Ok(())).unwrap();

        // Expand rail and set transients + active drag
        let mut now = 1000;
        state.on_rail_pointer_enter(now).unwrap();
        now += 200;
        state.tick_rail(now, 2).unwrap();
        assert!(state.rail_presentation().unwrap().expanded);

        state
            .set_rail_transient_reason(RailTransientReason::ManageItems, true)
            .unwrap();
        state
            .set_rail_transient_reason(RailTransientReason::ClearConfirmation, true)
            .unwrap();
        state.set_rail_drag_active(true).unwrap();

        // Fullscreen on a DIFFERENT monitor does NOT occlude or hide rail
        let occluded_other = edge_rail::rect_covers_monitor(&other_monitor_rect, &monitor_rect);
        assert!(!occluded_other);

        // Geometric coverage on the rail's monitor
        let occluded_same = edge_rail::rect_covers_monitor(&monitor_rect, &monitor_rect);
        assert!(occluded_same);

        // Standard maximized window covering only work area does NOT cover monitor_rect
        assert!(!edge_rail::rect_covers_monitor(&work, &monitor_rect));

        // Now coordinate fullscreen occlusion with show_over_fullscreen = false
        let rail_hidden = RefCell::new(false);
        let transients_closed = RefCell::new(false);
        coordinate_fullscreen(
            &state,
            true,  // is occluded
            false, // show_over_fullscreen = false
            || {
                *rail_hidden.borrow_mut() = true;
                Ok(())
            },
            || {
                *transients_closed.borrow_mut() = true;
                Ok(())
            },
        )
        .unwrap();

        assert!(*rail_hidden.borrow());
        assert!(*transients_closed.borrow());
        assert!(state.is_fullscreen_occluded().unwrap());
        // Visibility remains EdgeDocked
        assert_eq!(state.visibility().unwrap(), Visibility::EdgeDocked);
        // Rail presentation reset to collapsed
        assert!(!state.rail_presentation().unwrap().expanded);
        // Non-drag transient holds are cleared
        assert!(!state
            .has_transient_reason(RailTransientReason::ManageItems)
            .unwrap());
        assert!(!state
            .has_transient_reason(RailTransientReason::ClearConfirmation)
            .unwrap());
        // Active OLE drag remains preserved!
        assert!(state.rail_drag_active().unwrap());

        // Leaving fullscreen restores only collapsed rail handle
        let rail_shown = RefCell::new(false);
        coordinate_fullscreen(
            &state,
            false, // is un-occluded
            false, // show_over_fullscreen = false
            || {
                *rail_shown.borrow_mut() = true;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
        assert!(*rail_shown.borrow());
        assert!(!state.is_fullscreen_occluded().unwrap());
        assert!(!state.rail_presentation().unwrap().expanded);

        // If show_over_fullscreen is true, fullscreen coverage does NOT hide rail
        let rail_hidden_overlay = RefCell::new(false);
        coordinate_fullscreen(
            &state,
            true, // is occluded
            true, // show_over_fullscreen = true
            || {
                *rail_hidden_overlay.borrow_mut() = true;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
        assert!(!*rail_hidden_overlay.borrow());

        shelf.close().unwrap();
        std::fs::remove_dir_all(path).unwrap();
    }

    fn set_rail_presentation_for_test(
        state: &ResidentState,
        placement: &edge_rail::Placement,
        rail: RailPresentation,
    ) {
        state.set_visibility(Visibility::EdgeDocked).unwrap();
        state.set_rail_presentation(rail).unwrap();
        let Ok(mut guard) = state.presentation.lock() else {
            panic!("resident presentation lock is poisoned");
        };
        guard.rail_placement = Some(placement.clone());
    }

    #[test]
    fn shake_session_restores_hidden_or_rail_and_promotes_only_accepted_batches() {
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let placement = edge_rail::Placement {
            monitor: edge_rail::DisplayMonitor {
                id: "rail-display".into(),
                label: "rail".into(),
                device: "display".into(),
                monitor_rect: work,
                work,
                dpi: 96,
                primary: true,
            },
            edge: crate::settings_store::RailEdge::Left,
        };
        let state = ResidentState::default();
        let ops = MockWindowOps::new((600, 500), work, 96);
        assert!(coordinate_shake_begin(&ops, &state, 180.0, 180.0, || Ok(()), |_| Ok(())).unwrap());
        assert!(*ops.is_visible.borrow());
        assert!(!*ops.has_focus.borrow());
        assert!(state.mark_shake_released().unwrap());
        let session = state.active_shake_session().unwrap().unwrap();
        coordinate_shake_restore(&ops, &state, &session, |_| Ok(())).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Hidden);
        assert!(!*ops.is_visible.borrow());
        set_rail_presentation_for_test(
            &state,
            &placement,
            RailPresentation {
                expanded: true,
                width: 280.0,
                height: 340.0,
                ..Default::default()
            },
        );
        let rail_shown = RefCell::new(false);
        assert!(coordinate_shake_begin(&ops, &state, 180.0, 180.0, || Ok(()), |_| Ok(())).unwrap());
        assert!(!state.mark_shake_drag_inside(true).unwrap());
        let drop_id = state.mark_shake_drop_pending().unwrap();
        assert!(!state.mark_shake_released().unwrap());
        assert!(matches!(
            state.resolve_shake_drop(None, true).unwrap(),
            ShakeDropResolution::None
        ));
        let ShakeDropResolution::Restore(restored) =
            state.resolve_shake_drop(drop_id, false).unwrap()
        else {
            panic!("duplicate/rejected drop did not restore")
        };
        coordinate_shake_restore(&ops, &state, &restored, |saved| {
            assert_eq!(saved.previous_placement.as_ref(), Some(&placement));
            assert_eq!(
                (saved.previous_rail.expanded, saved.previous_rail.height),
                (true, 340.0)
            );
            *rail_shown.borrow_mut() = true;
            Ok(())
        })
        .unwrap();
        assert!(*rail_shown.borrow());
        assert_eq!(state.visibility().unwrap(), Visibility::EdgeDocked);
        assert_eq!(state.rail_presentation().unwrap().height, 340.0);

        assert!(coordinate_shake_begin(&ops, &state, 180.0, 180.0, || Ok(()), |_| Ok(())).unwrap());
        let accepted_id = state.mark_shake_drop_pending().unwrap();
        assert!(matches!(
            state.resolve_shake_drop(accepted_id, true).unwrap(),
            ShakeDropResolution::Promote
        ));
        coordinate_shake_promote(&ops, &state, accepted_id).unwrap();
        assert_eq!(state.visibility().unwrap(), Visibility::Visible);
        assert!(*ops.has_focus.borrow());
        assert!(state.active_shake_session().unwrap().is_none());
    }
}
