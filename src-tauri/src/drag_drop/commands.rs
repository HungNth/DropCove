//! Tauri command wrappers for native Batch and Item drag-out.

use crate::shelf::{ShelfSnapshot, ShelfState};
use tauri::{Manager, State};

pub(crate) fn storage_drag_on_window_thread(
    app: tauri::AppHandle,
    operation: impl FnOnce(
            &super::drag::DragSession,
        ) -> Result<crate::shelf::DragOutcome, super::drag::StorageDragError>
        + Send
        + 'static,
) -> Result<crate::shelf::DragOutcome, String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let drag_app = app.clone();
    app.run_on_main_thread(move || {
        struct RailDragGuard<'a>(&'a tauri::AppHandle);
        impl Drop for RailDragGuard<'_> {
            fn drop(&mut self) {
                match self
                    .0
                    .state::<crate::resident::ResidentState>()
                    .set_rail_drag_active(false)
                {
                    Ok(Some(remaining)) => {
                        if let Err(error) = crate::surfaces::edge_rail::arm_rail_timer(self.0, remaining as u32) {
                            let _ = crate::resident::publish_warning(self.0, &error);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let _ = crate::resident::publish_warning(self.0, &error);
                    }
                }
            }
        }
        if let Err(error) = drag_app
            .state::<crate::resident::ResidentState>()
            .set_rail_drag_active(true)
        {
            let _ = sender.send(Err(error));
            return;
        }
        let _guard = RailDragGuard(&drag_app);
        let session = drag_app.state::<super::drag::DragSession>();
        let result = operation(&session).map_err(|error| error.to_string());
        // A disappearing caller leaves its references retained.
        let _ = sender.send(result);
    })
    .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())?
}

#[derive(serde::Serialize)]
pub(crate) struct ItemDragResult {
    outcome: crate::shelf::DragOutcome,
    error: Option<String>,
}

#[derive(serde::Serialize)]
pub(crate) struct BatchDragResult {
    outcome: crate::shelf::DragOutcome,
    error: Option<String>,
}

#[tauri::command]
pub(crate) async fn prepare_item_drag(
    item_ids: Vec<String>,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<crate::shelf::ItemPreparation, String> {
    if window.label() != "manage-items" {
        return Err("prepare_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let active_batch = crate::surfaces::manage_items::active_batch(&app)?
            .ok_or_else(|| "Manage Items is not active".to_owned())?;
        let preparation = state
            .prepare_item_drag(&active_batch, &item_ids)
            .map_err(|error| error.to_string())?;
        if let Err(error) = crate::shelf::commands::publish_shelf(&app, preparation.snapshot.clone()) {
            let mut message = error;
            if let Some(token) = &preparation.token {
                if let Err(cleanup) = state.cancel_item_drag(token) {
                    message.push_str(&format!("; prepared drag cancellation failed: {cleanup}"));
                }
            }
            if let Err(cleanup) = crate::surfaces::manage_items::synchronize(&app) {
                message.push_str(&format!("; Manage Items synchronization failed: {cleanup}"));
            }
            return Err(message);
        }
        Ok(preparation)
    })
    .await
}

#[tauri::command]
pub(crate) async fn cancel_item_drag(
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ShelfSnapshot, String> {
    if window.label() != "manage-items" {
        return Err("cancel_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .cancel_item_drag(&token)
            .map_err(|error| error.to_string())?;
        crate::shelf::commands::publish_shelf(&app, snapshot)
    })
    .await
}

#[cfg(windows)]
#[tauri::command]
pub(crate) async fn start_item_drag(
    token: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<ItemDragResult, String> {
    if window.label() != "manage-items" {
        return Err("start_item_drag is only permitted from Manage Items".to_owned());
    }
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let active_batch = crate::surfaces::manage_items::active_batch(&app)?
            .ok_or_else(|| "Manage Items is not active".to_owned())?;
        let transfer = state
            .begin_item_drag(&token, Some(&active_batch))
            .map_err(|error| error.to_string())?;
        let single_item_text = if transfer.participants.len() == 1 {
            state.read_managed_text_content(&transfer.participants[0].id).map_err(|error| error.to_string())?
        } else {
            None
        };
        let native_transfer = std::sync::Arc::clone(&transfer);
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            super::drag::perform_storage_drag(
                session,
                native_transfer.paths.iter().map(String::as_str),
                single_item_text.as_deref(),
            )
        });

        match outcome {
            Ok(outcome) => {
                if outcome == crate::shelf::DragOutcome::Canceled {
                    use tauri::Emitter;
                    let _ = app.emit_to("manage-items", "item-drag-canceled", ());
                }
                let completion = state
                    .complete_item_drag(&transfer, outcome)
                    .map_err(|error| error.to_string())
                    .and_then(|snapshot| crate::shelf::commands::publish_shelf(&app, snapshot));
                if let Err(sync_error) = crate::surfaces::manage_items::synchronize(&app) {
                    let error_message = match completion.err() {
                        Some(err) => format!("{err}; Manage Items synchronization failed: {sync_error}"),
                        None => format!("Manage Items synchronization failed: {sync_error}"),
                    };
                    return Ok(ItemDragResult { outcome, error: Some(error_message) });
                }
                Ok(ItemDragResult { outcome, error: completion.err() })
            }
            Err(error) => {
                match crate::surfaces::manage_items::synchronize(&app) {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(format!("{error}; Manage Items synchronization failed: {cleanup}")),
                }
            }
        }
    })
    .await
}

#[tauri::command]
pub(crate) async fn prepare_batch_drag(
    batch_ids: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<crate::shelf::BatchPreparation, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        crate::surfaces::manage_items::close(&app)?;
        let preparation = state
            .prepare_batch_drag(&batch_ids)
            .map_err(|error| error.to_string())?;
        crate::shelf::commands::publish_shelf(&app, preparation.snapshot.clone())?;
        Ok::<_, String>(preparation)
    })
    .await
}

#[tauri::command]
pub(crate) async fn cancel_batch_drag(
    token: String,
    state: State<'_, ShelfState>,
    app: tauri::AppHandle,
) -> Result<ShelfSnapshot, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        let snapshot = state
            .cancel_batch_drag(&token)
            .map_err(|error| error.to_string())?;
        crate::shelf::commands::publish_shelf(&app, snapshot)
    })
    .await
}

#[tauri::command]
pub(crate) async fn start_batch_drag(
    token: String,
    app: tauri::AppHandle,
    state: State<'_, ShelfState>,
) -> Result<BatchDragResult, String> {
    let state = state.inner().clone();
    crate::run_blocking(move || {
        if let Err(error) = crate::surfaces::manage_items::close(&app) {
            return match state.cancel_batch_drag(&token) {
                Ok(_) => Err(error),
                Err(cleanup) => Err(format!("{error}; prepared drag cancellation failed: {cleanup}")),
            };
        }
        let transfer = state
            .begin_batch_drag(&token)
            .map_err(|error| error.to_string())?;
        let single_item_text = if transfer.participants.len() == 1 {
            state.read_managed_text_content(&transfer.participants[0].id).map_err(|error| error.to_string())?
        } else {
            None
        };
        let native_transfer = std::sync::Arc::clone(&transfer);
        let outcome = storage_drag_on_window_thread(app.clone(), move |session| {
            super::drag::perform_storage_drag(
                session,
                native_transfer.paths.iter().map(String::as_str),
                single_item_text.as_deref(),
            )
        });
        match outcome {
            Ok(outcome) => {
                let completion = state
                    .complete_batch_drag(&transfer, outcome)
                    .map_err(|error| error.to_string())
                    .and_then(|snapshot| crate::shelf::commands::publish_shelf(&app, snapshot));
                Ok(BatchDragResult { outcome, error: completion.err() })
            }
            Err(error) => match crate::surfaces::manage_items::synchronize(&app) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!("{error}; Manage Items synchronization failed: {cleanup}")),
            },
        }
    })
    .await
}
