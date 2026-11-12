use std::cell::RefCell;

use super::*;
use crate::resident::window_ops::{PhysicalPlacement, WindowOps, WorkArea};
use crate::resident::rail::RailPresentation;

mod drop_shelf;
mod fullscreen;
mod lifecycle;
mod rail;
mod shake_session;

pub(super) struct MockWindowOps {
    pub(super) cursor: (i32, i32),
    pub(super) work_area: WorkArea,
    pub(super) dpi: u32,
    pub(super) frame: (f64, f64),
    placement: RefCell<Option<PhysicalPlacement>>,
    pub(super) is_visible: RefCell<bool>,
    pub(super) has_focus: RefCell<bool>,
    pub(super) topmost_reasserted: RefCell<bool>,
    pub(super) fail_placement: bool,
    pub(super) fail_show: bool,
    pub(super) fail_hide: bool,
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

pub(super) fn set_rail_presentation_for_test(
    state: &ResidentState,
    placement: &crate::surfaces::edge_rail::Placement,
    rail: RailPresentation,
) {
    state.set_visibility(Visibility::EdgeDocked).unwrap();
    state.set_rail_presentation(rail).unwrap();
    let Ok(mut guard) = state.presentation.lock() else {
        panic!("resident presentation lock is poisoned");
    };
    guard.rail_placement = Some(placement.clone());
}
