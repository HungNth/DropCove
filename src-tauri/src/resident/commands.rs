//! Tauri command wrappers for the Resident Instance and Edge Rail presentation.

use tauri::Manager;

#[tauri::command]
pub(crate) async fn dismiss_drop_shelf(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("dismiss_drop_shelf is only permitted from the main Drop Shelf window".to_owned());
    }
    crate::run_blocking(move || super::drop_shelf::dismiss_shelf(&app)).await
}

#[tauri::command]
pub(crate) fn get_rail(app: tauri::AppHandle) -> Result<crate::surfaces::edge_rail::Projection, String> {
    let edge = match super::rail::rail_placement(&app)? {
        Some(placement) => placement.edge,
        None => crate::settings::active(&app)?.rail_edge,
    };
    let rail = app.state::<super::ResidentState>().rail_presentation()?;
    Ok(crate::surfaces::edge_rail::Projection {
        edge,
        expanded: rail.expanded,
        width: rail.width,
        height: rail.height,
    })
}

#[tauri::command]
pub(crate) fn on_rail_body_pointer_enter(app: tauri::AppHandle) -> Result<(), String> {
    let now = super::now_monotonic_ms();
    let state = app.state::<super::ResidentState>();
    let rail = state.on_rail_body_pointer_enter(now)?;
    if let Some(due) = rail.expand_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        if let Err(error) = crate::surfaces::edge_rail::arm_rail_timer(&app, remaining) {
            let _ = super::publish_warning(&app, &error);
        }
    } else {
        crate::surfaces::edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn on_rail_button_pointer_enter(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<super::ResidentState>();
    let rail = state.on_rail_button_pointer_enter()?;
    if rail.expand_due_ms.is_none() && rail.collapse_due_ms.is_none() {
        crate::surfaces::edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn on_rail_pointer_leave(app: tauri::AppHandle) -> Result<(), String> {
    let now = super::now_monotonic_ms();
    let state = app.state::<super::ResidentState>();
    let rail = state.on_rail_pointer_leave(now)?;
    if let Some(due) = rail.collapse_due_ms {
        let remaining = (due.saturating_sub(now)).max(1) as u32;
        if let Err(error) = crate::surfaces::edge_rail::arm_rail_timer(&app, remaining) {
            let _ = super::publish_warning(&app, &error);
        }
    } else {
        crate::surfaces::edge_rail::cancel_rail_timer(&app);
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn tick_rail(app: tauri::AppHandle) -> Result<(), String> {
    super::rail::tick_rail(&app)
}

#[tauri::command]
pub(crate) fn open_shelf_from_rail(app: tauri::AppHandle) -> Result<(), String> {
    super::rail::open_shelf_from_rail(&app)
}

#[tauri::command]
pub(crate) fn set_rail_transient_hold(
    active: bool,
    reason: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let reason_enum = match reason.as_str() {
        "clear" => super::rail::RailTransientReason::ClearConfirmation,
        "batch-drag" => super::rail::RailTransientReason::BatchDragConfirmation,
        "item-actions" => super::rail::RailTransientReason::ItemActions,
        "selection-actions" => super::rail::RailTransientReason::SelectionActions,
        _ => return Err(format!("unknown rail transient hold reason '{reason}'")),
    };
    if let Some(remaining) = app
        .state::<super::ResidentState>()
        .set_rail_transient_reason(reason_enum, active)?
    {
        let _ = crate::surfaces::edge_rail::arm_rail_timer(&app, remaining as u32);
    }
    Ok(())
}
