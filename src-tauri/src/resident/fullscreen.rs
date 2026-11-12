//! Edge Rail hiding and restoration under fullscreen coverage.

use super::*;
use super::window_ops::{WorkArea};
use super::rail::{RailPresentation, RailTransientReason, rail_placement};

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
                crate::surfaces::edge_rail::hide(app)
            } else {
                let rail = state.rail_presentation()?;
                crate::surfaces::edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded)
            }
        },
        || {
            crate::surfaces::manage_items::close(app)?;
            let _ = app.emit("rail-transients-closed", ());
            Ok(())
        },
    )
}

pub(super) fn is_monitor_occluded_by_foreground(monitor_rect: &WorkArea) -> bool {
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
    crate::surfaces::edge_rail::rect_covers_monitor(&window_rect, monitor_rect)
}
