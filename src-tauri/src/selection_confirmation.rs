use std::{collections::HashMap, sync::Mutex};
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub(crate) struct SelectionConfirmations(Mutex<HashMap<String, String>>);

pub(crate) fn begin(app: &AppHandle, caller: &str) -> Result<(), String> {
    let owner = if caller == "manage-items" {
        crate::manage_items::origin(app)?.unwrap_or_else(|| caller.to_owned())
    } else {
        caller.to_owned()
    };
    app.state::<SelectionConfirmations>().0.lock()
        .map_err(|_| "selection confirmation lock is poisoned".to_owned())?
        .insert(caller.to_owned(), owner);
    Ok(())
}

pub(crate) fn cancel(app: &AppHandle, caller: &str) -> Result<(), String> {
    app.state::<SelectionConfirmations>().0.lock()
        .map_err(|_| "selection confirmation lock is poisoned".to_owned())?
        .remove(caller);
    Ok(())
}

pub(crate) fn blocks(app: &AppHandle, caller: &str) -> bool {
    app.state::<SelectionConfirmations>().0.lock()
        .map(|owners| owners.contains_key(caller) || owners.values().any(|owner| owner == caller))
        .unwrap_or(true)
}
