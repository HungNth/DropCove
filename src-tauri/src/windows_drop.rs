use std::{cell::Cell, sync::Mutex};

use tauri::{Emitter, Manager, WebviewWindow};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller4;
use windows::{
    core::{implement, Error, Interface, Ref, Result, BOOL},
    Win32::{
        Foundation::{DRAGDROP_E_NOTREGISTERED, E_FAIL, E_POINTER, HWND, LPARAM, POINTL},
        System::{
            Com::{
                CoTaskMemFree, IDataObject, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, TYMED_HGLOBAL,
            },
            Ole::{
                IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
                CF_HDROP, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
            },
            SystemServices::MODIFIERKEYS_FLAGS,
        },
        UI::{
            Shell::{
                DragQueryFileW, IShellItemArray, SHCreateShellItemArrayFromDataObject, HDROP,
                SIGDN_FILESYSPATH,
            },
            WindowsAndMessaging::EnumChildWindows,
        },
    },
};

use crate::shelf::ShelfState;
use crate::windows_drag::DragSession;

#[implement(IDropTarget)]
struct StorageDropTarget {
    window: WebviewWindow,
    can_copy: Cell<bool>,
}

#[derive(Default)]
struct RegisteredDropWindows(Mutex<Vec<isize>>);

/// # Safety
/// `context` must point to the caller's live, exclusively borrowed `Vec<HWND>` during enumeration.
unsafe extern "system" fn collect_child(hwnd: HWND, context: LPARAM) -> BOOL {
    // SAFETY: EnumChildWindows calls synchronously while the caller's Vec is alive and unique.
    unsafe {
        (&mut *(context.0 as *mut Vec<HWND>)).push(hwnd);
    }
    true.into()
}
fn storage_paths(data: &IDataObject) -> Result<Vec<Option<String>>> {
    // SAFETY: The callback owns a live IDataObject for this synchronous read. COM interface
    // wrappers retain their references; each display-name allocation is freed below.
    unsafe {
        let items: IShellItemArray = match SHCreateShellItemArrayFromDataObject(data) {
            Ok(items) => items,
            Err(_) => return hdrop_paths(data),
        };
        let count = items.GetCount()?;
        let mut paths = Vec::with_capacity(count as usize);
        for index in 0..count {
            let path = match items
                .GetItemAt(index)
                .and_then(|item| item.GetDisplayName(SIGDN_FILESYSPATH))
            {
                Ok(value) => {
                    let owned = value.to_string().ok().filter(|value| !value.is_empty());
                    CoTaskMemFree(Some(value.0.cast()));
                    owned
                }
                Err(_) => None,
            };
            paths.push(path);
        }
        Ok(paths)
    }
}

struct StorageMedium(STGMEDIUM);

impl Drop for StorageMedium {
    fn drop(&mut self) {
        // SAFETY: GetData transferred one owned STGMEDIUM, released exactly once here.
        unsafe {
            ReleaseStgMedium(&mut self.0);
        }
    }
}

fn hdrop_paths(data: &IDataObject) -> Result<Vec<Option<String>>> {
    let format = FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    // SAFETY: GetData supplies the owned HGLOBAL; DragQueryFileW reads it while its medium lives.
    unsafe {
        let medium = StorageMedium(data.GetData(&format)?);
        if medium.0.tymed != TYMED_HGLOBAL.0 as u32 {
            return Err(Error::from_hresult(E_FAIL));
        }
        let drop = HDROP(medium.0.u.hGlobal.0);
        let count = DragQueryFileW(drop, u32::MAX, None);
        let mut paths = Vec::with_capacity(count as usize);
        for index in 0..count {
            let length = DragQueryFileW(drop, index, None) as usize;
            let mut buffer = vec![0u16; length + 1];
            let read = DragQueryFileW(drop, index, Some(&mut buffer)) as usize;
            paths.push(
                String::from_utf16(&buffer[..read])
                    .ok()
                    .filter(|path| !path.is_empty()),
            );
        }
        Ok(paths)
    }
}

/// # Safety
/// `effect` must be uniquely writable for the returned borrow, bounded by the OLE callback.
unsafe fn requested_effect<'a>(effect: *mut DROPEFFECT) -> Result<&'a mut DROPEFFECT> {
    // SAFETY: OLE passes a writable effect pointer for the duration of each callback. A null
    // pointer is rejected; callers do not retain the returned borrow beyond that callback.
    unsafe {
        effect
            .as_mut()
            .ok_or_else(|| Error::from_hresult(E_POINTER))
    }
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for StorageDropTarget_Impl {
    fn DragEnter(
        &self,
        data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        _point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        // SAFETY: OLE provides this output pointer for this callback only.
        let effect = unsafe { requested_effect(effect)? };
        let own_drag = self.window.state::<DragSession>().is_active();
        let supported = !own_drag
            && data
                .as_ref()
                .and_then(|data| storage_paths(data).ok())
                .is_some_and(|paths| paths.iter().any(Option::is_some));
        let can_copy = effect.contains(DROPEFFECT_COPY) && supported;
        self.can_copy.set(can_copy);
        *effect = if can_copy {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        };
        // A disappearing presentation listener does not affect the storage operation.
        let _ = self.window.emit("shelf-drag-over", can_copy);
        Ok(())
    }

    fn DragOver(
        &self,
        _keys: MODIFIERKEYS_FLAGS,
        _point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        // SAFETY: OLE provides this output pointer for this callback only.
        let effect = unsafe { requested_effect(effect)? };
        *effect = if !self.window.state::<DragSession>().is_active()
            && self.can_copy.get()
            && effect.contains(DROPEFFECT_COPY)
        {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        };
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        self.can_copy.set(false);
        let _ = self.window.emit("shelf-drag-over", false);
        Ok(())
    }

    fn Drop(
        &self,
        data: Ref<'_, IDataObject>,
        _keys: MODIFIERKEYS_FLAGS,
        _point: &POINTL,
        effect: *mut DROPEFFECT,
    ) -> Result<()> {
        // SAFETY: OLE provides this output pointer for this callback only.
        let effect = unsafe { requested_effect(effect)? };
        let allowed =
            !self.window.state::<DragSession>().is_active() && effect.contains(DROPEFFECT_COPY);
        *effect = DROPEFFECT_NONE;
        self.can_copy.set(false);
        let _ = self.window.emit("shelf-drag-over", false);
        if !allowed {
            return Ok(());
        }
        let Some(data) = data.as_ref() else {
            return Ok(());
        };
        let paths = storage_paths(data)?;
        if !paths.iter().any(Option::is_some) {
            return Ok(());
        }
        let window = self.window.clone();
        let state = window.state::<ShelfState>().inner().clone();
        // The IDataObject never crosses threads. Only its owned path projection does.
        tauri::async_runtime::spawn_blocking(move || match state.accept_drop(paths) {
            Ok(result) => {
                if result.snapshot.is_some() {
                    if let Err(error) = crate::manage_items::synchronize(window.app_handle()) {
                        let _ = window.emit("shelf-error", error);
                    }
                }
                let _ = window.emit("shelf-drop", result);
            }
            Err(error) => {
                let _ = window.emit("shelf-error", error.to_string());
            }
        });
        *effect = DROPEFFECT_COPY;
        Ok(())
    }
}

pub(crate) fn register(window: &WebviewWindow) -> tauri::Result<()> {
    window.app_handle().manage(RegisteredDropWindows::default());
    let window = window.clone();
    let target_window = window.clone();
    window.with_webview(move |webview| {
        let result = (|| -> std::result::Result<(), Box<dyn std::error::Error>> {
            let mut children = Vec::<HWND>::new();
            // SAFETY: Tauri executes this callback on the owning WebView thread. Each enumerated
            // HWND is a child of this app's surface; no foreign top-level window is touched.
            unsafe {
                webview
                    .controller()
                    .cast::<ICoreWebView2Controller4>()?
                    .SetAllowExternalDrop(false)?;
                let context = LPARAM((&mut children as *mut Vec<HWND>) as isize);
                let _ = EnumChildWindows(Some(target_window.hwnd()?), Some(collect_child), context);
                let mut registered = Vec::with_capacity(children.len());
                for hwnd in children {
                    let target: IDropTarget = StorageDropTarget {
                        window: target_window.clone(),
                        can_copy: Cell::new(false),
                    }
                    .into();
                    // This is Wry's established integration pattern; the payload parser differs
                    // because CF_HDROP cannot count virtual Shell entries in mixed drops.
                    let _ = RevokeDragDrop(hwnd);
                    if RegisterDragDrop(hwnd, &target).is_ok() {
                        registered.push(hwnd.0 as isize);
                    }
                }
                if registered.is_empty() {
                    return Err(Error::from_hresult(E_FAIL).into());
                }
                *target_window
                    .state::<RegisteredDropWindows>()
                    .0
                    .lock()
                    .map_err(|_| std::io::Error::other("drop registration lock is poisoned"))? =
                    registered;
            }
            Ok(())
        })();
        if let Err(error) = result {
            crate::fatal_startup_error(format!(
                "storage drop target initialization failed: {error}"
            ));
        }
    })
}

pub(crate) fn unregister(window: &tauri::Window) {
    if let Some(registration) = window.try_state::<RegisteredDropWindows>() {
        if let Ok(mut handles) = registration.0.lock() {
            for handle in handles.drain(..) {
                // SAFETY: CloseRequested runs on the owning thread before child destruction.
                if let Err(error) = unsafe { RevokeDragDrop(HWND(handle as *mut _)) } {
                    if error.code() != DRAGDROP_E_NOTREGISTERED {
                        eprintln!("storage drop target cleanup failed: {error}");
                    }
                }
            }
        }
    }
}
