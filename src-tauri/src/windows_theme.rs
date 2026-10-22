//! Resident-owned Windows appearance projection and native frame synchronization.

use crate::settings_store::AppearanceMode;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Theme, WebviewWindow};
use windows::{
    core::w,
    Win32::{
        Foundation::{ERROR_SUCCESS, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
        System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Shell::{DefSubclassProc, GetWindowSubclass, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                IsWindow, SystemParametersInfoW, SPI_GETHIGHCONTRAST,
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_NCDESTROY, WM_SETTINGCHANGE,
                WM_THEMECHANGED,
            },
        },
    },
};

const SUBCLASS: usize = 0xDC07;
const APPEARANCE_EVENT: &str = "appearance-changed";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ResolvedTheme {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppearanceSnapshot {
    pub(crate) mode: AppearanceMode,
    pub(crate) theme: ResolvedTheme,
    pub(crate) high_contrast: bool,
    pub(crate) revision: u64,
}

struct AppearanceState(Mutex<AppearanceSnapshot>);

fn resolve(mode: AppearanceMode, revision: u64) -> Result<AppearanceSnapshot, String> {
    let mut light = 1u32;
    let mut size = 4u32;
    // SAFETY: The registry query writes one DWORD into a valid stack buffer.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut light as *mut u32).cast()),
            Some(&mut size),
        )
    };
    let mut contrast = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    // SAFETY: The correctly sized output remains valid throughout the synchronous query.
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            contrast.cbSize,
            Some((&mut contrast as *mut HIGHCONTRASTW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .map_err(|error| format!("could not read Windows Contrast Theme: {error}"))?;
    let high_contrast = contrast.dwFlags.0 & HCF_HIGHCONTRASTON.0 != 0;
    let system_dark = result == ERROR_SUCCESS && light == 0;
    let dark = if high_contrast || mode == AppearanceMode::System {
        system_dark
    } else {
        mode == AppearanceMode::Dark
    };
    Ok(AppearanceSnapshot {
        mode,
        theme: if dark {
            ResolvedTheme::Dark
        } else {
            ResolvedTheme::Light
        },
        high_contrast,
        revision,
    })
}

pub(crate) fn initialize(app: &AppHandle, mode: AppearanceMode) -> Result<(), String> {
    app.manage(AppearanceState(Mutex::new(resolve(mode, 1)?)));
    synchronize(app, snapshot(app)?)
}

pub(crate) fn snapshot(app: &AppHandle) -> Result<AppearanceSnapshot, String> {
    let state = app
        .try_state::<AppearanceState>()
        .ok_or("appearance is not initialized")?;
    let value = *state.0.lock().map_err(|_| "appearance lock is poisoned")?;
    Ok(value)
}

pub(crate) fn current_theme(app: &AppHandle) -> Result<Option<Theme>, String> {
    Ok(native_theme(snapshot(app)?))
}

fn native_theme(value: AppearanceSnapshot) -> Option<Theme> {
    if value.high_contrast {
        None
    } else {
        Some(match value.theme {
            ResolvedTheme::Dark => Theme::Dark,
            ResolvedTheme::Light => Theme::Light,
        })
    }
}

fn apply_window(window: &WebviewWindow, value: AppearanceSnapshot) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    // SAFETY: This only queries the captured owned HWND, which can have just been destroyed.
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return Ok(());
    }
    window
        .set_theme(native_theme(value))
        .map_err(|error| error.to_string())?;
    if !value.high_contrast {
        let dark = i32::from(value.theme == ResolvedTheme::Dark);
        // SAFETY: The live owned frame receives a four-byte BOOL on its UI thread.
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&dark as *const i32).cast(),
                4,
            )
        }
        .map_err(|error| format!("{} native appearance: {error}", window.label()))?;
    }
    Ok(())
}

pub(crate) fn apply_to_window(window: &WebviewWindow) -> Result<(), String> {
    apply_window(window, snapshot(window.app_handle())?)
}

fn synchronize(app: &AppHandle, value: AppearanceSnapshot) -> Result<(), String> {
    let mut errors = Vec::new();
    for label in [
        "main",
        "settings",
        "edge-rail",
        "manage-items",
        "about",
        "clear-temporary",
    ] {
        if let Some(window) = app.get_webview_window(label) {
            if let Err(error) = apply_window(&window, value) {
                errors.push(error);
            }
        }
    }
    if let Err(error) = app.emit(APPEARANCE_EVENT, value) {
        errors.push(error.to_string());
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub(crate) fn apply(app: &AppHandle, mode: AppearanceMode) -> Result<(), String> {
    let previous = snapshot(app)?;
    let next = resolve(mode, previous.revision + 1)?;
    let state = app.state::<AppearanceState>();
    *state.0.lock().map_err(|_| "appearance lock is poisoned")? = next;
    if let Err(error) = synchronize(app, next) {
        let restored = AppearanceSnapshot {
            revision: next.revision + 1,
            ..previous
        };
        *state.0.lock().map_err(|_| "appearance lock is poisoned")? = restored;
        return match synchronize(app, restored) {
            Ok(()) => Err(error),
            Err(restore) => Err(format!(
                "{error}; appearance rollback incomplete: {restore}"
            )),
        };
    }
    Ok(())
}

fn refresh(app: &AppHandle) -> Result<(), String> {
    let previous = snapshot(app)?;
    let next = resolve(previous.mode, previous.revision)?;
    if next.theme != previous.theme || next.high_contrast != previous.high_contrast {
        apply(app, previous.mode)?;
    }
    Ok(())
}

pub(crate) fn register(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let mut existing = 0usize;
    // SAFETY: This queries only our subclass on the live owned HWND.
    if unsafe { GetWindowSubclass(hwnd, Some(message), SUBCLASS, Some(&mut existing)) }.as_bool() {
        return apply_to_window(window);
    }
    let data = Box::into_raw(Box::new(window.app_handle().clone()));
    // SAFETY: Registration is on the owning UI thread; the subclass owns data until destruction.
    if !unsafe { SetWindowSubclass(hwnd, Some(message), SUBCLASS, data as usize) }.as_bool() {
        // SAFETY: Failed registration transferred no ownership.
        unsafe {
            drop(Box::from_raw(data));
        }
        return Err("could not observe native theme changes".into());
    }
    apply_to_window(window)
}

unsafe extern "system" fn message(
    hwnd: HWND,
    code: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    if code == WM_SETTINGCHANGE || code == WM_THEMECHANGED {
        // SAFETY: The subclass owns data; clone before a potentially reentrant window operation.
        let app = unsafe { (&*(data as *const AppHandle)).clone() };
        if let Err(error) = refresh(&app) {
            if let Err(warning) = crate::resident::publish_warning(&app, &error) {
                eprintln!("{error}; warning delivery failed: {warning}");
            }
        }
    }
    if code == WM_NCDESTROY {
        // SAFETY: Remove the observer and release its sole allocation before forwarding destruction.
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(message), id);
            drop(Box::from_raw(data as *mut AppHandle));
        }
    }
    // SAFETY: Preserve native arguments for the remaining subclass chain.
    unsafe { DefSubclassProc(hwnd, code, wparam, lparam) }
}
