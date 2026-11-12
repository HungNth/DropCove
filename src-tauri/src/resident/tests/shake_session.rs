use super::*;
use crate::resident::window_ops::{WorkArea};
use crate::resident::rail::{RailPresentation};
use crate::resident::shake_session::{ShakeDropResolution, coordinate_shake_begin, coordinate_shake_promote, coordinate_shake_restore};

#[test]
fn shake_session_restores_hidden_or_rail_and_promotes_only_accepted_batches() {
    let work = WorkArea {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    let placement = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "rail-display".into(),
            label: "rail".into(),
            device: "display".into(),
            monitor_rect: work,
            work,
            dpi: 96,
            primary: true,
        },
        edge: crate::settings::store::RailEdge::Left,
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
