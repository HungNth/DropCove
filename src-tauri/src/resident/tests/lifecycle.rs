use super::*;
use crate::resident::window_ops::{WorkArea};
use crate::resident::drop_shelf::{coordinate_hide, coordinate_show};

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
