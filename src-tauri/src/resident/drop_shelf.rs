//! Show, hide, dock, and dismiss the Drop Shelf.

use tauri::{AppHandle, Emitter, Manager};
use crate::shelf::ShelfState;
use std::sync::atomic::{AtomicBool, Ordering};
use super::{ResidentState, SHELF_WINDOW_LABEL, Visibility};
use super::window_ops::{NativeWindowOps, WindowOps, calculate_placement};
use super::rail::{rail_placement, restore_rail};
use super::fullscreen::{evaluate_fullscreen_coverage};

pub(super) fn place_shelf(
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

/// Public API: show shelf on cursor monitor using durable preferred sizing.
/// Always shows rather than toggles; repeated calls are idempotent and reposition to current cursor.
pub(crate) fn show_shelf(app: &AppHandle) -> Result<(), String> {
    show_shelf_at(app, None)
}

pub(super) fn show_shelf_at(
    app: &AppHandle,
    monitor: Option<&crate::surfaces::edge_rail::DisplayMonitor>,
) -> Result<(), String> {
    if crate::surfaces::clear_confirmation::is_modal_active(app) {
        crate::surfaces::clear_confirmation::request(app, "main")?;
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
        crate::surfaces::manage_items::close(app)?;
        let _ = app.emit("rail-transients-closed", ());
        Ok(())
    })?;
    crate::surfaces::edge_rail::hide(app)?;
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
        let visual_state = app.state::<crate::surfaces::shell_visuals::ShellVisualState>();
        let new_gen = visual_state.0.advance_generation(crate::surfaces::shell_visuals::SurfaceId::Shelf);
        use tauri::Emitter;
        let _ = app.emit(
            "visual-generation-changed",
            crate::surfaces::shell_visuals::GenerationChangedPayload {
                surface: crate::surfaces::shell_visuals::SurfaceId::Shelf,
                generation: new_gen,
            },
        );
    }
    Ok(())
}

pub(super) fn coordinate_dock(
    ops: &dyn WindowOps,
    state: &ResidentState,
    shelf: &ShelfState,
    placement: &crate::surfaces::edge_rail::Placement,
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
        let mut presentation = state.lock_presentation()?;
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
    crate::surfaces::clear_confirmation::guard_interaction(app)?;
    let window = app
        .get_webview_window(SHELF_WINDOW_LABEL)
        .ok_or("Drop Shelf window is missing")?;
    let shelf = app.state::<ShelfState>();
    let snapshot = shelf.snapshot().map_err(|e| e.to_string())?;
    state
        .had_content
        .store(!snapshot.batches.is_empty(), Ordering::Release);
    crate::surfaces::manage_items::close(app)?;
    {
        let visual_state = app.state::<crate::surfaces::shell_visuals::ShellVisualState>();
        let new_gen = visual_state.0.advance_generation(crate::surfaces::shell_visuals::SurfaceId::Shelf);
        use tauri::Emitter;
        let _ = app.emit(
            "visual-generation-changed",
            crate::surfaces::shell_visuals::GenerationChangedPayload {
                surface: crate::surfaces::shell_visuals::SurfaceId::Shelf,
                generation: new_gen,
            },
        );
    }
    if snapshot.batches.is_empty() {
        crate::surfaces::edge_rail::hide(app)?;
        return coordinate_hide(&NativeWindowOps::new(&window), &state, || Ok(()));
    }
    let placement = crate::surfaces::edge_rail::placement_for_dismissal(app, &crate::settings::active(app)?)?;
    let rail = state.rail_presentation()?;
    coordinate_dock(
        &NativeWindowOps::new(&window),
        &state,
        &shelf,
        &placement,
        || crate::surfaces::edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded),
        || crate::surfaces::edge_rail::hide(app),
    )?;
    evaluate_fullscreen_coverage(app)
}
