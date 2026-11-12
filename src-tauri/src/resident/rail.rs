//! Edge Rail presentation state, expansion and collapse timing, settings application, and pointer classification.

use super::*;
use super::drop_shelf::{coordinate_docked_to_shelf, show_shelf_at};

pub(crate) const RAIL_WIDTH: f64 = 280.0;

pub(crate) const RAIL_HANDLE_WIDTH: f64 = 16.0;

pub(crate) const RAIL_MIN_HEIGHT: f64 = 280.0;

pub(crate) const RAIL_MAX_HEIGHT: f64 = 642.0;

pub(crate) const RAIL_EXPAND_DELAY_MS: u64 = 200;

pub(crate) const RAIL_COLLAPSE_DELAY_MS: u64 = 300;

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

pub(crate) fn rail_placement(app: &AppHandle) -> Result<Option<crate::surfaces::edge_rail::Placement>, String> {
    app.state::<ResidentState>().placement()
}

pub(super) fn set_rail_placement(
    app: &AppHandle,
    placement: Option<crate::surfaces::edge_rail::Placement>,
) -> Result<(), String> {
    app.state::<ResidentState>()
        .lock_presentation()?
        .rail_placement = placement;
    Ok(())
}

pub(crate) fn restore_rail(
    app: &AppHandle,
    placement: Option<&crate::surfaces::edge_rail::Placement>,
) -> Result<(), String> {
    if app.state::<ResidentState>().visibility()? == Visibility::EdgeDocked {
        let rail = app.state::<ResidentState>().rail_presentation()?;
        match placement {
            Some(placement) => {
                crate::surfaces::edge_rail::show(app, placement, rail.width, rail.height, rail.expanded)?
            }
            None => crate::surfaces::edge_rail::hide(app)?,
        }
    }
    set_rail_placement(app, placement.cloned())
}

pub(crate) fn coordinate_apply_rail_settings(
    state: &ResidentState,
    shelf: &ShelfState,
    placement: &crate::surfaces::edge_rail::Placement,
    rail_monitor_id_setting: &str,
    show_native: impl FnOnce(&crate::surfaces::edge_rail::Placement) -> Result<(), String>,
    restore_native: impl FnOnce(Option<&crate::surfaces::edge_rail::Placement>) -> Result<(), String>,
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
        let mut presentation = state.lock_presentation()?;
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
            state.lock_presentation().map(|mut pres| {
                pres.rail_placement = prior_placement.clone();
            })
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
    let placement = crate::surfaces::edge_rail::placement(app, settings)?;
    let shelf = app.state::<ShelfState>();
    let rail = state.rail_presentation()?;
    coordinate_apply_rail_settings(
        &state,
        &shelf,
        &placement,
        &settings.rail_monitor_id,
        |p| crate::surfaces::edge_rail::show(app, p, rail.width, rail.height, rail.expanded),
        |prev| match prev {
            Some(prev_p) => crate::surfaces::edge_rail::show(app, prev_p, rail.width, rail.height, rail.expanded),
            None => crate::surfaces::edge_rail::hide(app),
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
        if let Err(error) = crate::surfaces::edge_rail::show(app, &placement, rail.width, rail.height, rail.expanded)
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
        crate::surfaces::edge_rail::show(app, &placement, next.width, next.height, next.expanded)?;
        state.set_rail_presentation(next)?;
        crate::surfaces::edge_rail::cancel_rail_timer(app);
    } else {
        state.tick_rail(now, batch_count)?;
        let rail = state.rail_presentation()?;
        if let Some(due) = rail.expand_due_ms.or(rail.collapse_due_ms) {
            let remaining = (due.saturating_sub(now)).max(1) as u32;
            crate::surfaces::edge_rail::arm_rail_timer(app, remaining)?;
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
                crate::surfaces::manage_items::close(app)?;
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
    let window = match app.get_webview_window(crate::surfaces::edge_rail::LABEL) {
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
        let _ = crate::surfaces::edge_rail::arm_rail_timer(app, remaining);
    } else if let Some(due) = rail.collapse_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        let _ = crate::surfaces::edge_rail::arm_rail_timer(app, remaining);
    } else {
        crate::surfaces::edge_rail::cancel_rail_timer(app);
    }
    Ok(())
}

impl ResidentState {
    pub(crate) fn set_rail_transient_reason(
        &self,
        reason: RailTransientReason,
        active: bool,
    ) -> Result<Option<u64>, String> {
        let now = now_monotonic_ms();
        let mut guard = self.lock_presentation()?;
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
        self.lock_presentation()
            .map(|p| p.transient_holds != 0)
    }

    pub(crate) fn set_rail_drag_active(&self, active: bool) -> Result<Option<u64>, String> {
        self.set_rail_transient_reason(RailTransientReason::ActiveDrag, active)
    }

    #[cfg(test)]
    pub(crate) fn rail_drag_active(&self) -> Result<bool, String> {
        self.lock_presentation()
            .map(|p| (p.transient_holds & (RailTransientReason::ActiveDrag as u8)) != 0)
    }

    #[cfg(test)]
    pub(crate) fn is_fullscreen_occluded(&self) -> Result<bool, String> {
        self.lock_presentation()
            .map(|p| p.fullscreen_occluded)
    }

    #[cfg(test)]
    pub(crate) fn has_transient_reason(&self, reason: RailTransientReason) -> Result<bool, String> {
        self.lock_presentation()
            .map(|p| (p.transient_holds & (reason as u8)) != 0)
    }

    pub(crate) fn rail_presentation(&self) -> Result<RailPresentation, String> {
        self.lock_presentation()
            .map(|p| p.rail)
    }

    pub(crate) fn set_rail_presentation(&self, rail: RailPresentation) -> Result<(), String> {
        self.lock_presentation()
            .map(|mut p| {
                p.rail = rail;
            })
    }

    pub(crate) fn on_rail_pointer_enter(&self, now_ms: u64) -> Result<RailPresentation, String> {
        let mut guard = self.lock_presentation()?;
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
        let mut guard = self.lock_presentation()?;
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
        let mut guard = self.lock_presentation()?;
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
        let guard = self.lock_presentation()?;
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
            let mut guard = self.lock_presentation()?;
            guard.rail = next;
            Ok(Some(next))
        } else {
            let mut guard = self.lock_presentation()?;
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
        let mut guard = self.lock_presentation()?;
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
