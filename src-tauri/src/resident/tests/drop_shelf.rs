use super::*;
use crate::resident::window_ops::{PhysicalPlacement, WorkArea, calculate_placement};
use crate::resident::drop_shelf::{coordinate_docked_to_shelf, coordinate_hide, coordinate_show};

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
