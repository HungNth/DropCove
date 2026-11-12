use super::*;
use crate::resident::window_ops::{WorkArea};
use crate::resident::drop_shelf::{coordinate_dock, coordinate_show};
use crate::resident::rail::{RAIL_COLLAPSE_DELAY_MS, RailPointerRegion, RailTransientReason, calculate_rail_height, classify_rail_pointer, coordinate_apply_rail_settings};

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
    let placement = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "selected".into(),
            label: "selected".into(),
            device: "display".into(),
            monitor_rect: work,
            work,
            dpi: 96,
            primary: true,
        },
        edge: crate::settings::store::RailEdge::Right,
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
    let placement_a = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "monitor_a".into(),
            label: "monitor_a".into(),
            device: "display_a".into(),
            monitor_rect: work,
            work,
            dpi: 96,
            primary: true,
        },
        edge: crate::settings::store::RailEdge::Right,
    };
    let placement_b = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "monitor_b".into(),
            label: "monitor_b".into(),
            device: "display_b".into(),
            monitor_rect: work,
            work,
            dpi: 96,
            primary: false,
        },
        edge: crate::settings::store::RailEdge::Left,
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
    let placement = crate::surfaces::edge_rail::Placement {
        monitor: crate::surfaces::edge_rail::DisplayMonitor {
            id: "test".into(),
            label: "test".into(),
            device: "display".into(),
            monitor_rect: work,
            work,
            dpi: 96,
            primary: true,
        },
        edge: crate::settings::store::RailEdge::Right,
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
