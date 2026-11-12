use super::*;
use crate::resident::window_ops::{WorkArea};
use crate::resident::drop_shelf::{coordinate_dock};
use crate::resident::rail::RailTransientReason;
use crate::resident::fullscreen::{coordinate_fullscreen};

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
    let placement = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "test".into(),
            label: "test".into(),
            device: "display".into(),
            monitor_rect,
            work,
            dpi: 96,
            primary: true,
        },
        edge: crate::settings::store::RailEdge::Right,
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
    let occluded_other = crate::surfaces::edge_rail::rect_covers_monitor(&other_monitor_rect, &monitor_rect);
    assert!(!occluded_other);

    // Geometric coverage on the rail's monitor
    let occluded_same = crate::surfaces::edge_rail::rect_covers_monitor(&monitor_rect, &monitor_rect);
    assert!(occluded_same);

    // Standard maximized window covering only work area does NOT cover monitor_rect
    assert!(!crate::surfaces::edge_rail::rect_covers_monitor(&work, &monitor_rect));

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
