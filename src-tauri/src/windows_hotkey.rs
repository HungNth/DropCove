//! Resident global hotkey registration. Focused Settings input is handled by its control.

use crate::settings_store::HotKey;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};
use tauri::{AppHandle, Manager, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Input::KeyboardAndMouse::{
            RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
            MOD_WIN,
        },
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{GetWindowThreadProcessId, WM_HOTKEY, WM_NCDESTROY},
    },
};

const SUBCLASS_ID: usize = 0xDC10;
const MIN_HOTKEY_ID: i32 = 0x1000;
const MAX_HOTKEY_ID: i32 = 0x7FFF;
struct NativeHotkeyState {
    app: AppHandle,
    hwnd: isize,
    next_id: i32,
    registrations: HashMap<i32, HotKey>,
}
static STATE: LazyLock<Mutex<Option<NativeHotkeyState>>> = LazyLock::new(|| Mutex::new(None));

fn with_state<R>(
    operation: impl FnOnce(&mut NativeHotkeyState) -> Result<R, String>,
) -> Result<R, String> {
    let mut guard = STATE
        .lock()
        .map_err(|_| "global hotkey state lock is poisoned")?;
    let state = guard
        .as_mut()
        .ok_or("global hotkey receiver is not initialized")?;
    // SAFETY: The managed HWND belongs to this process; only its owning UI thread registers keys.
    if unsafe { GetWindowThreadProcessId(HWND(state.hwnd as *mut _), None) != GetCurrentThreadId() }
    {
        return Err("global hotkey operation must run on the window thread".into());
    }
    operation(state)
}

pub(crate) fn initialize(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    // SAFETY: HWND is owned and live; both APIs inspect scalar thread identities.
    if unsafe { GetWindowThreadProcessId(hwnd, None) != GetCurrentThreadId() } {
        return Err("global hotkey receiver must be initialized on the window thread".into());
    }
    let mut state = STATE
        .lock()
        .map_err(|_| "global hotkey state lock is poisoned")?;
    if let Some(existing) = state.as_ref() {
        return if existing.hwnd == hwnd.0 as isize {
            Ok(())
        } else {
            Err("global hotkey receiver is already initialized".into())
        };
    }
    // SAFETY: Installing our callback on the owning window thread; no raw context allocation.
    if !unsafe { SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) }.as_bool() {
        return Err("could not install global hotkey receiver".into());
    }
    *state = Some(NativeHotkeyState {
        app: window.app_handle().clone(),
        hwnd: hwnd.0 as isize,
        next_id: MIN_HOTKEY_ID,
        registrations: HashMap::new(),
    });
    Ok(())
}

pub(crate) fn register(_app: &AppHandle, key: &HotKey) -> Result<i32, String> {
    key.validate()?;
    with_state(|state| {
        if state
            .registrations
            .values()
            .any(|registered| registered == key)
        {
            return Err("shortcut is already registered".into());
        }
        if state.next_id > MAX_HOTKEY_ID {
            return Err("global hotkey identifiers exhausted".into());
        }
        let id = state.next_id;
        state.next_id += 1;
        let mut modifiers = MOD_NOREPEAT;
        if key.control {
            modifiers |= MOD_CONTROL;
        }
        if key.alt {
            modifiers |= MOD_ALT;
        }
        if key.shift {
            modifiers |= MOD_SHIFT;
        }
        if key.windows {
            modifiers |= MOD_WIN;
        }
        // SAFETY: with_state verifies the live window's owning UI thread.
        unsafe {
            RegisterHotKey(
                Some(HWND(state.hwnd as *mut _)),
                id,
                modifiers,
                u32::from(key.virtual_key),
            )
        }
        .map_err(|error| format!("shortcut registration failed (it may be occupied): {error}"))?;
        state.registrations.insert(id, *key);
        Ok(id)
    })
}

pub(crate) fn unregister(_app: &AppHandle, id: i32) -> Result<(), String> {
    with_state(|state| {
        if state.registrations.contains_key(&id) {
            // SAFETY: with_state verifies ownership; keep registry state if the OS rejects removal.
            unsafe { UnregisterHotKey(Some(HWND(state.hwnd as *mut _)), id) }
                .map_err(|error| error.to_string())?;
            state.registrations.remove(&id);
        }
        Ok(())
    })
}

pub(crate) fn is_registered(_app: &AppHandle, id: i32) -> bool {
    STATE
        .lock()
        .ok()
        .and_then(|state| {
            state
                .as_ref()
                .map(|state| state.registrations.contains_key(&id))
        })
        .unwrap_or(false)
}

pub(crate) fn shutdown(_app: &AppHandle) -> Result<(), String> {
    let mut guard = STATE.lock().map_err(|_| "global hotkey state lock is poisoned")?;
    if let Some(state) = guard.as_mut() {
        let hwnd = HWND(state.hwnd as *mut _);
        // SAFETY: The managed HWND is live; only its owning UI thread performs teardown.
        if unsafe { GetWindowThreadProcessId(hwnd, None) != GetCurrentThreadId() } {
            return Err("global hotkey teardown must run on the window thread".into());
        }
        while let Some(id) = state.registrations.keys().next().copied() {
            // SAFETY: Owner-thread check above; preserve each ID if removal fails.
            unsafe { UnregisterHotKey(Some(hwnd), id) }.map_err(|error| error.to_string())?;
            state.registrations.remove(&id);
        }
        // SAFETY: Removing the subclass on the thread that installed it.
        let _ = unsafe { RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID) };
    }
    guard.take();
    Ok(())
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    if message == WM_HOTKEY {
        let app = STATE
            .lock()
            .ok()
            .and_then(|state| state.as_ref().map(|state| state.app.clone()));
        if let Some(app) = app {
            if let Err(error) = crate::settings::handle_hotkey(&app, wparam.0 as i32) {
                if let Err(warning) = crate::resident::publish_warning(&app, &error) {
                    eprintln!("{error}; warning delivery failed: {warning}");
                }
            }
        }
        return LRESULT(0);
    }
    if message == WM_NCDESTROY {
        // SAFETY: Removing our subclass during its owner's final destruction.
        let _ = unsafe { RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID) };
    }
    // SAFETY: Forward the original message to the remaining subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}
