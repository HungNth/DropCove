//! Shake-to-open presentation session: begin, restore, and promote on an accepted drop.

use super::*;
use super::window_ops::{NativeWindowOps, WindowOps};
use super::drop_shelf::place_shelf;
use super::rail::{RailPresentation, RailTransientReason};
use super::fullscreen::evaluate_fullscreen_coverage;

#[derive(Clone, Debug)]
pub(super) struct ShakeSession {
    pub(super) id: u64,
    pub(super) previous_visibility: Visibility,
    pub(super) previous_placement: Option<crate::surfaces::edge_rail::Placement>,
    pub(super) previous_rail: RailPresentation,
    pub(super) drag_inside: bool,
    pub(super) drop_pending: bool,
    pub(super) released: bool,
}

pub(super) enum ShakeDropResolution {
    None,
    Restore(ShakeSession),
    Promote,
}

pub(super) fn coordinate_shake_begin(
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

pub(super) fn coordinate_shake_restore(
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

pub(super) fn coordinate_shake_promote(
    ops: &dyn WindowOps,
    state: &ResidentState,
    session_id: Option<u64>,
) -> Result<(), String> {
    let show = ops.show_and_activate();
    state.finish_shake_promote(session_id)?;
    show
}

pub(super) fn show_saved_shake_rail(app: &AppHandle, session: &ShakeSession) -> Result<(), String> {
    let placement = session
        .previous_placement
        .as_ref()
        .ok_or("saved EdgeDocked Shake session has no rail placement")?;
    crate::surfaces::edge_rail::show(
        app,
        placement,
        session.previous_rail.width,
        session.previous_rail.height,
        session.previous_rail.expanded,
    )
}

pub(super) fn restore_shake_on_window_thread(app: &AppHandle) -> Result<(), String> {
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
    if app.state::<crate::drag_drop::drag::DragSession>().is_active() {
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
            crate::surfaces::manage_items::close(app)?;
            let _ = app.emit("rail-transients-closed", ());
            crate::surfaces::edge_rail::hide(app)
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

pub(super) fn complete_shake_drop_on_window_thread(
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

impl ResidentState {
    pub(super) fn shake_candidate(&self) -> Result<Option<ShakeSession>, String> {
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

    pub(super) fn commit_shake(&self, session: ShakeSession) -> Result<(), String> {
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

    pub(super) fn mark_shake_drag_inside(&self, inside: bool) -> Result<bool, String> {
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

    pub(super) fn mark_shake_drop_pending(&self) -> Result<Option<u64>, String> {
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

    pub(super) fn mark_shake_released(&self) -> Result<bool, String> {
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

    pub(super) fn resolve_shake_drop(
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

    pub(super) fn finish_shake_restore(&self, session: &ShakeSession) -> Result<(), String> {
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

    pub(super) fn finish_shake_promote(&self, session_id: Option<u64>) -> Result<(), String> {
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

    pub(super) fn active_shake_session(&self) -> Result<Option<ShakeSession>, String> {
        self.presentation
            .lock()
            .map(|guard| guard.shake_session.clone())
            .map_err(|_| "resident presentation lock is poisoned".to_owned())
    }
}
