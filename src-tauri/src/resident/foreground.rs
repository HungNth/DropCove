//! WinEvent callbacks are delivered to the installing UI thread; no cross-thread raw context.

use std::cell::RefCell;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            GetWindowThreadProcessId, EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND,
            INDEXID_CONTAINER, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
            WM_DISPLAYCHANGE, WM_NCDESTROY,
        },
    },
};

const SUBCLASS: usize = 0xDC08;
struct Observer {
    app: AppHandle,
    hwnd: HWND,
    foreground_hook: HWINEVENTHOOK,
    location_hook: HWINEVENTHOOK,
    current_foreground: HWND,
}
// WinEventProc has no userdata parameter. OUTOFCONTEXT delivers on the installing
// thread; release every RefCell borrow before native calls that could re-enter it.
thread_local! { static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) }; }

pub(crate) fn register(window: &WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    // SAFETY: Both APIs inspect scalar thread/window identities; HWND is live and owned.
    if unsafe { GetWindowThreadProcessId(hwnd, None) != GetCurrentThreadId() } {
        return Err("foreground observer must be installed on the Shelf UI thread".into());
    }
    if OBSERVER.with(|observer| observer.borrow().is_some()) {
        return Err("foreground observer is already registered".into());
    }
    // SAFETY: Callback matches WINEVENTPROC. OUTOFCONTEXT queues on this UI thread,
    // so it cannot race the thread-local owner or explicit teardown.
    let foreground_hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(foreground_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if foreground_hook.0.is_null() {
        return Err("could not install foreground observer".into());
    }
    let location_hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_LOCATIONCHANGE,
            None,
            Some(location_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    if location_hook.0.is_null() {
        let _ = unsafe { UnhookWinEvent(foreground_hook) };
        return Err("could not install location change observer".into());
    }
    // SAFETY: Setup runs on the window thread; no raw allocation is handed to this subclass.
    if !unsafe { SetWindowSubclass(hwnd, Some(window_event), SUBCLASS, 0) }.as_bool() {
        // SAFETY: The just-created hooks belong to this installing UI thread.
        let _ = unsafe { UnhookWinEvent(location_hook) };
        let _ = unsafe { UnhookWinEvent(foreground_hook) };
        return Err("could not observe Shelf display events".into());
    }
    OBSERVER.with(|observer| {
        *observer.borrow_mut() = Some(Observer {
            app: window.app_handle().clone(),
            hwnd,
            foreground_hook,
            location_hook,
            current_foreground: HWND::default(),
        })
    });
    Ok(())
}

pub(crate) fn shutdown(_app: &AppHandle) -> Result<(), String> {
    let observer = OBSERVER.with(|observer| observer.borrow_mut().take());
    let Some(observer) = observer else {
        return Ok(());
    };
    // SAFETY: Teardown is called from the same UI thread that installed the hooks.
    let unhook_fg = unsafe { UnhookWinEvent(observer.foreground_hook) }.as_bool();
    let unhook_loc = unsafe { UnhookWinEvent(observer.location_hook) }.as_bool();
    if !unhook_fg || !unhook_loc {
        OBSERVER.with(|slot| *slot.borrow_mut() = Some(observer));
        return Err("could not remove foreground observer".into());
    }
    // SAFETY: Remove our subclass before releasing its owning AppHandle. WM_NCDESTROY
    // can also call this path; removal is harmless when the window is already tearing down.
    let _ = unsafe { RemoveWindowSubclass(observer.hwnd, Some(window_event), SUBCLASS) };
    Ok(())
}

fn report(app: &AppHandle, error: String) {
    if let Err(warning) = crate::resident::publish_warning(app, &error) {
        eprintln!("{error}; warning delivery failed: {warning}");
    }
}

unsafe extern "system" fn foreground_changed(
    hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND {
        return;
    }
    let app = OBSERVER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let observer = slot.as_mut().filter(|observer| observer.foreground_hook == hook)?;
        observer.current_foreground = hwnd;
        Some(observer.app.clone())
    });
    if let Some(app) = app {
        if let Err(error) = crate::resident::repair_topmost(&app) {
            report(&app, error);
        }
        if let Err(error) = crate::resident::evaluate_fullscreen_coverage(&app) {
            report(&app, error);
        }
    }
}

unsafe extern "system" fn location_changed(
    hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object: i32,
    child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_OBJECT_LOCATIONCHANGE
        || object != OBJID_WINDOW.0
        || child != INDEXID_CONTAINER as i32
    {
        return;
    }
    let app = OBSERVER.with(|slot| {
        let slot = slot.borrow();
        let observer = slot.as_ref().filter(|observer| observer.location_hook == hook)?;
        if observer.current_foreground != hwnd {
            return None;
        }
        Some(observer.app.clone())
    });
    if let Some(app) = app {
        if let Err(error) = crate::resident::evaluate_fullscreen_coverage(&app) {
            report(&app, error);
        }
    }
}

unsafe extern "system" fn window_event(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    if message == WM_DISPLAYCHANGE || message == WM_NCDESTROY {
        let app = OBSERVER.with(|slot| slot.borrow().as_ref().map(|observer| observer.app.clone()));
        if let Some(app) = app {
            let result = if message == WM_NCDESTROY {
                shutdown(&app)
            } else {
                // Existing frontend reflow uses current monitor/frame bounds, without showing or activating.
                app.emit("shelf-changed", ())
                    .map_err(|error| error.to_string())
                    .and_then(|()| crate::resident::reflow_rail(&app))
            };
            if let Err(error) = result {
                report(&app, error);
            }
        }
    }
    // SAFETY: Forward the original message to the remaining native subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}
