use std::{cell::Cell, sync::LazyLock, sync::Mutex};

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
            DataExchange::RegisterClipboardFormatW,
            Memory::{GlobalLock, GlobalSize, GlobalUnlock},
            Ole::{
                IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
                CF_HDROP, CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
            },
            SystemServices::MODIFIERKEYS_FLAGS,
        },
        UI::{
            Shell::{
                DragQueryFileW, IShellItemArray, SHCreateShellItemArrayFromDataObject,
                CFSTR_FILEDESCRIPTORW, HDROP, SIGDN_FILESYSPATH,
            },
            WindowsAndMessaging::EnumChildWindows,
        },
    },
};

use crate::shelf::ShelfState;
use crate::windows_drag::DragSession;

pub(crate) const MAX_UNICODE_TEXT_BYTES: usize = 10 * 1024 * 1024;

static FILE_DESCRIPTOR_W_FORMAT: LazyLock<u32> = LazyLock::new(|| {
    // SAFETY: RegisterClipboardFormatW safely registers or returns the atom for FileGroupDescriptorW.
    unsafe { RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW) }
});

static FILE_DESCRIPTOR_A_FORMAT: LazyLock<u32> = LazyLock::new(|| {
    // SAFETY: RegisterClipboardFormatW safely registers or returns the atom for FileGroupDescriptor (ANSI).
    unsafe { RegisterClipboardFormatW(windows::core::w!("FileGroupDescriptor")) }
});

#[implement(IDropTarget)]
struct StorageDropTarget {
    window: WebviewWindow,
    can_copy: Cell<bool>,
}

#[derive(Default)]
struct RegisteredDropWindows(Mutex<Vec<isize>>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EvaluatedPayload {
    Filesystem(Vec<Option<String>>),
    UnicodeText(String),
    InvalidUnicodeText,
    Unsupported,
}

impl EvaluatedPayload {
    pub(crate) fn is_supported(&self) -> bool {
        match self {
            Self::Filesystem(paths) => paths.iter().any(Option::is_some),
            Self::UnicodeText(_) => true,
            Self::InvalidUnicodeText => false,
            Self::Unsupported => false,
        }
    }
}

/// # Safety
/// `context` must point to the caller's live, exclusively borrowed `Vec<HWND>` during enumeration.
unsafe extern "system" fn collect_child(hwnd: HWND, context: LPARAM) -> BOOL {
    // SAFETY: EnumChildWindows calls synchronously while the caller's Vec is alive and unique.
    unsafe {
        (&mut *(context.0 as *mut Vec<HWND>)).push(hwnd);
    }
    true.into()
}

fn has_non_url_file_group_descriptor(data: &IDataObject) -> bool {
    for (format_id, wide) in [(*FILE_DESCRIPTOR_W_FORMAT, true), (*FILE_DESCRIPTOR_A_FORMAT, false)] {
        if format_id == 0 { continue; }
        let format = FORMATETC {
            cfFormat: format_id as u16, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0,
            lindex: -1, tymed: TYMED_HGLOBAL.0 as u32,
        };
        // SAFETY: GetData transfers an owned STGMEDIUM. StorageMedium ensures ReleaseStgMedium runs.
        let medium = match unsafe { data.GetData(&format) } {
            Ok(medium) => StorageMedium(medium),
            Err(_) => continue,
        };
        let hglobal = unsafe { medium.0.u.hGlobal };
        if medium.0.tymed != TYMED_HGLOBAL.0 as u32 || hglobal.is_invalid() { continue; }
        // SAFETY: GlobalLock is safe on a valid HGLOBAL and balanced by GlobalUnlock.
        let ptr = unsafe { GlobalLock(hglobal) };
        if ptr.is_null() { continue; }
        struct Unlock(windows::Win32::Foundation::HGLOBAL);
        impl Drop for Unlock { fn drop(&mut self) { unsafe { let _ = GlobalUnlock(self.0); } } }
        let _unlock = Unlock(hglobal);
        let count = unsafe { *(ptr as *const u32) };
        if count == 0 { continue; }
        // Chromium and Edge synthesize a single .url Internet Shortcut when dragging links.
        // If every descriptor in the payload is a synthesized .url file, do not treat it as a
        // file-like payload that suppresses accompanying plain text.
        let is_url_descriptor = |name: &str| name.to_ascii_lowercase().ends_with(".url");
        let mut only_urls = true;
        if wide {
            #[repr(C)]
            struct FILEDESCRIPTORW_HEAD { flags: u32, clsid: [u8; 16], size: [u32; 2], point: [i32; 2], file_attribs: u32, ft: [u32; 6], size_high: u32, size_low: u32, name: [u16; 260] }
            let first = unsafe { &*((ptr as *const u8).add(4) as *const FILEDESCRIPTORW_HEAD) };
            let len = first.name.iter().position(|&c| c == 0).unwrap_or(first.name.len());
            if let Ok(name) = String::from_utf16(&first.name[..len]) {
                if !is_url_descriptor(&name) { only_urls = false; }
            } else { only_urls = false; }
        } else {
            #[repr(C)]
            struct FILEDESCRIPTORA_HEAD { flags: u32, clsid: [u8; 16], size: [u32; 2], point: [i32; 2], file_attribs: u32, ft: [u32; 6], size_high: u32, size_low: u32, name: [u8; 260] }
            let first = unsafe { &*((ptr as *const u8).add(4) as *const FILEDESCRIPTORA_HEAD) };
            let len = first.name.iter().position(|&c| c == 0).unwrap_or(first.name.len());
            if let Ok(name) = std::str::from_utf8(&first.name[..len]) {
                if !is_url_descriptor(name) { only_urls = false; }
            } else { only_urls = false; }
        }
        if !only_urls { return true; }
    }
    false
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

pub(crate) fn decode_bounded_unicode_text(units: &[u16]) -> Option<String> {
    let nul_index = units.iter().position(|&unit| unit == 0)?;
    if nul_index == 0 {
        return None;
    }
    let content_units = &units[..nul_index];
    let mut utf8_len = 0usize;
    for item in char::decode_utf16(content_units.iter().copied()) {
        let ch = item.ok()?;
        utf8_len = utf8_len.checked_add(ch.len_utf8())?;
        if utf8_len > MAX_UNICODE_TEXT_BYTES {
            return None;
        }
    }
    let mut text = String::with_capacity(utf8_len);
    for item in char::decode_utf16(content_units.iter().copied()) {
        let ch = match item {
            Ok(ch) => ch,
            Err(_) => return None,
        };
        text.push(ch);
    }
    Some(text)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum TextExtraction {
    Valid(String),
    Invalid,
    None,
}

fn extract_unicode_text(data: &IDataObject) -> TextExtraction {
    let format = FORMATETC {
        cfFormat: CF_UNICODETEXT.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    // SAFETY: GetData transfers an owned STGMEDIUM. StorageMedium ensures ReleaseStgMedium runs.
    unsafe {
        let medium = match data.GetData(&format) {
            Ok(medium) => StorageMedium(medium),
            Err(_) => return TextExtraction::None,
        };
        if medium.0.tymed != TYMED_HGLOBAL.0 as u32 {
            return TextExtraction::Invalid;
        }
        let hglobal = medium.0.u.hGlobal;
        if hglobal.is_invalid() {
            return TextExtraction::Invalid;
        }
        let raw_size = GlobalSize(hglobal);
        if raw_size < std::mem::size_of::<u16>() {
            return TextExtraction::Invalid;
        }
        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            return TextExtraction::Invalid;
        }
        struct GlobalUnlockGuard(windows::Win32::Foundation::HGLOBAL);
        impl Drop for GlobalUnlockGuard {
            fn drop(&mut self) {
                // SAFETY: Unlocks the HGLOBAL locked above.
                unsafe {
                    let _ = GlobalUnlock(self.0);
                }
            }
        }
        let _unlock = GlobalUnlockGuard(hglobal);
        let total_units = raw_size / std::mem::size_of::<u16>();
        let max_units_to_inspect = total_units.min(MAX_UNICODE_TEXT_BYTES + 1);
        let units = std::slice::from_raw_parts(ptr as *const u16, max_units_to_inspect);
        match decode_bounded_unicode_text(units) {
            Some(text) => TextExtraction::Valid(text),
            None => TextExtraction::Invalid,
        }
    }
}

fn has_hdrop_format(data: &IDataObject) -> bool {
    let format = FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    // SAFETY: QueryGetData inspects available formats without allocating or reading payload data.
    unsafe { data.QueryGetData(&format).is_ok() }
}

pub(crate) fn evaluate_payload_precedence(
    filesystem_paths: Option<Vec<Option<String>>>,
    has_virtual_file_format: bool,
    text_extraction: TextExtraction,
) -> EvaluatedPayload {
    if let Some(paths) = filesystem_paths {
        if paths.iter().any(Option::is_some) {
            return EvaluatedPayload::Filesystem(paths);
        }
        // Higher-priority filesystem format was offered but empty or invalid; reject without
        // falling through to companion text/URL.
        return EvaluatedPayload::Unsupported;
    }
    if has_virtual_file_format {
        // Higher-priority virtual file format was offered; unsupported in 0.0.23, reject without
        // falling through to companion text/URL.
        return EvaluatedPayload::Unsupported;
    }
    match text_extraction {
        TextExtraction::Valid(text) => EvaluatedPayload::UnicodeText(text),
        TextExtraction::Invalid => EvaluatedPayload::InvalidUnicodeText,
        TextExtraction::None => EvaluatedPayload::Unsupported,
    }
}

pub(crate) fn evaluate_data_payload(data: &IDataObject) -> EvaluatedPayload {
    let filesystem_paths = storage_paths(data).ok().or_else(|| {
        if has_hdrop_format(data) {
            Some(Vec::new())
        } else {
            None
        }
    });
    let has_virtual = has_non_url_file_group_descriptor(data);
    let text_extraction = if filesystem_paths.is_none() && !has_virtual {
        extract_unicode_text(data)
    } else {
        TextExtraction::None
    };
    evaluate_payload_precedence(filesystem_paths, has_virtual, text_extraction)
}

pub(crate) const INVALID_TEXT_DROP_MESSAGE: &str =
    "Text drop is empty, invalid, or exceeds 10 MiB.";

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

pub(crate) fn evaluate_storage_drop(
    own_drag_active: bool,
    offered_effect: DROPEFFECT,
    has_valid_storage_paths: bool,
) -> DROPEFFECT {
    if own_drag_active || !offered_effect.contains(DROPEFFECT_COPY) || !has_valid_storage_paths {
        DROPEFFECT_NONE
    } else {
        DROPEFFECT_COPY
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
        let payload = data.as_ref().map(evaluate_data_payload);
        let supported = payload.as_ref().is_some_and(EvaluatedPayload::is_supported);
        let modal_active = self.window.label() == "main"
            && crate::clear_confirmation::is_modal_active(self.window.app_handle());
        if !own_drag && !modal_active && matches!(payload, Some(EvaluatedPayload::InvalidUnicodeText)) {
            let _ = self.window.app_handle().emit("shelf-error", INVALID_TEXT_DROP_MESSAGE);
        }
        let resolved_effect = if modal_active {
            DROPEFFECT_NONE
        } else {
            evaluate_storage_drop(own_drag, *effect, supported)
        };
        let can_copy = resolved_effect == DROPEFFECT_COPY;
        self.can_copy.set(can_copy);
        *effect = resolved_effect;
        if self.window.label() == "main" {
            if let Err(error) = crate::resident::shake_drag_inside(self.window.app_handle(), true) {
                let _ = crate::resident::publish_warning(self.window.app_handle(), &error);
            }
        }
        if self.window.label() == crate::edge_rail::LABEL && can_copy {
            let now = crate::resident::now_monotonic_ms();
            match self
                .window
                .state::<crate::resident::ResidentState>()
                .on_rail_pointer_enter(now)
            {
                Ok(rail) => {
                    if let Some(due) = rail.expand_due_ms {
                        let remaining = (due.saturating_sub(now)).max(1) as u32;
                        if let Err(error) =
                            crate::edge_rail::arm_rail_timer(self.window.app_handle(), remaining)
                        {
                            let _ =
                                crate::resident::publish_warning(self.window.app_handle(), &error);
                        }
                    } else {
                        crate::edge_rail::cancel_rail_timer(self.window.app_handle());
                    }
                }
                Err(error) => {
                    let _ = crate::resident::publish_warning(self.window.app_handle(), &error);
                }
            }
        }
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
        let own_drag = self.window.state::<DragSession>().is_active();
        let resolved_effect = evaluate_storage_drop(own_drag, *effect, self.can_copy.get());
        *effect = resolved_effect;
        Ok(())
    }

    fn DragLeave(&self) -> Result<()> {
        self.can_copy.set(false);
        if self.window.label() == "main" {
            if let Err(error) = crate::resident::shake_drag_inside(self.window.app_handle(), false)
            {
                let _ = crate::resident::publish_warning(self.window.app_handle(), &error);
            }
        }
        if self.window.label() == crate::edge_rail::LABEL {
            if let Err(error) =
                crate::resident::reconcile_rail_pointer_after_drag(self.window.app_handle())
            {
                let _ = crate::resident::publish_warning(self.window.app_handle(), &error);
            }
        }
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
        if self.window.label() == "main"
            && crate::clear_confirmation::is_modal_active(self.window.app_handle())
        {
            *effect = DROPEFFECT_NONE;
            self.can_copy.set(false);
            let _ = self.window.emit("shelf-drag-over", false);
            return Ok(());
        }
        let own_drag = self.window.state::<DragSession>().is_active();
        let shake_id = if self.window.label() == "main" {
            crate::resident::shake_drop_started(self.window.app_handle())
                .map_err(|_| Error::from_hresult(E_FAIL))?
        } else {
            None
        };
        let payload = match data.as_ref() {
            Some(data) => evaluate_data_payload(data),
            None => EvaluatedPayload::Unsupported,
        };
        let supported = payload.is_supported();
        let resolved_effect = evaluate_storage_drop(own_drag, *effect, supported);
        *effect = DROPEFFECT_NONE;
        self.can_copy.set(false);
        let _ = self.window.emit("shelf-drag-over", false);
        if self.window.label() == crate::edge_rail::LABEL {
            if let Err(error) =
                crate::resident::reconcile_rail_pointer_after_drag(self.window.app_handle())
            {
                let _ = crate::resident::publish_warning(self.window.app_handle(), &error);
            }
        }
        if resolved_effect != DROPEFFECT_COPY {
            let _ =
                crate::resident::complete_shake_drop(self.window.app_handle(), shake_id, false);
            if matches!(payload, EvaluatedPayload::InvalidUnicodeText) {
                let _ = crate::resident::publish_warning(
                    self.window.app_handle(),
                    INVALID_TEXT_DROP_MESSAGE,
                );
                let _ = self
                    .window
                    .app_handle()
                    .emit("shelf-error", INVALID_TEXT_DROP_MESSAGE);
            }
            return Ok(());
        }
        let window = self.window.clone();
        let state = window.state::<ShelfState>().inner().clone();
        // The IDataObject never crosses threads. Only owned extracted data does.
        tauri::async_runtime::spawn_blocking(move || {
            let intake_result = match payload {
                EvaluatedPayload::Filesystem(paths) => state.accept_drop(paths),
                EvaluatedPayload::UnicodeText(text) => state.accept_text(text),
                EvaluatedPayload::InvalidUnicodeText | EvaluatedPayload::Unsupported => {
                    Ok(crate::shelf::DropResult {
                        snapshot: None,
                        skipped: 0,
                    })
                }
            };
            match intake_result {
                Ok(result) => {
                    if let Err(error) = crate::resident::complete_shake_drop(
                        window.app_handle(),
                        shake_id,
                        result.snapshot.is_some(),
                    ) {
                        let _ = crate::resident::publish_warning(window.app_handle(), &error);
                    }
                    if let Some(snapshot) = result.snapshot.as_ref() {
                        if let Err(error) = crate::resident::synchronize(
                            window.app_handle(),
                            !snapshot.batches.is_empty(),
                        )
                        .and_then(|()| crate::manage_items::synchronize(window.app_handle()))
                        .and_then(|()| {
                            crate::resident::update_rail_content(
                                window.app_handle(),
                                snapshot.batches.len(),
                            )
                        }) {
                            let _ = crate::resident::publish_warning(window.app_handle(), &error);
                        }
                        let _ = window.app_handle().emit("shelf-changed", ());
                    }
                    let _ = window.app_handle().emit("shelf-drop", result);
                }
                Err(error) => {
                    let _ =
                        crate::resident::complete_shake_drop(window.app_handle(), shake_id, false);
                    let _ =
                        crate::resident::publish_warning(window.app_handle(), &error.to_string());
                    let _ = window.app_handle().emit("shelf-error", error.to_string());
                }
            }
        });
        *effect = DROPEFFECT_COPY;
        Ok(())
    }
}

pub(crate) fn register(window: &WebviewWindow) -> tauri::Result<()> {
    if window
        .app_handle()
        .try_state::<RegisteredDropWindows>()
        .is_none()
    {
        window.app_handle().manage(RegisteredDropWindows::default());
    }
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
                let registration_state = target_window.state::<RegisteredDropWindows>();
                let mut guard = registration_state
                    .0
                    .lock()
                    .map_err(|_| std::io::Error::other("drop registration lock is poisoned"))?;
                guard.extend(registered);
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

pub(crate) fn unregister(app: &tauri::AppHandle) {
    if let Some(registration) = app.try_state::<RegisteredDropWindows>() {
        if let Ok(mut handles) = registration.0.lock() {
            for handle in handles.drain(..) {
                // SAFETY: Explicit resident exit runs on the owning UI thread before child destruction.
                if let Err(error) = unsafe { RevokeDragDrop(HWND(handle as *mut _)) } {
                    if error.code() != DRAGDROP_E_NOTREGISTERED {
                        eprintln!("storage drop target cleanup failed: {error}");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_bounded_unicode_text_rejects_empty_and_unterminated() {
        assert_eq!(decode_bounded_unicode_text(&[]), None);
        // Only NUL -> empty content -> rejected
        assert_eq!(decode_bounded_unicode_text(&[0]), None);
        // No NUL -> rejected
        let no_nul: Vec<u16> = "Hello".encode_utf16().collect();
        assert_eq!(decode_bounded_unicode_text(&no_nul), None);
    }

    #[test]
    fn decode_bounded_unicode_text_accepts_plain_and_multiline_text() {
        let text = "Line 1\r\nLine 2\nLine 3 \t with tabs";
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.push(0);
        units.push(0x1234); // extra trailing garbage after NUL
        assert_eq!(decode_bounded_unicode_text(&units).as_deref(), Some(text));
    }

    #[test]
    fn decode_bounded_unicode_text_preserves_unicode_and_whitespace() {
        // Whitespace-only text must be accepted
        let whitespace = "   \t\r\n  ";
        let mut units: Vec<u16> = whitespace.encode_utf16().collect();
        units.push(0);
        assert_eq!(
            decode_bounded_unicode_text(&units).as_deref(),
            Some(whitespace)
        );

        // Non-ASCII and emoji
        let non_ascii = "Chào bạn! 🦀 日本語 \u{1F600}";
        let mut units: Vec<u16> = non_ascii.encode_utf16().collect();
        units.push(0);
        assert_eq!(
            decode_bounded_unicode_text(&units).as_deref(),
            Some(non_ascii)
        );
    }

    #[test]
    fn decode_bounded_unicode_text_rejects_malformed_surrogates() {
        // Unpaired high surrogate followed by NUL
        let invalid_surrogate = [0xD800, 0x0000];
        assert_eq!(decode_bounded_unicode_text(&invalid_surrogate), None);
    }

    #[test]
    fn decode_bounded_unicode_text_bounds_at_10_mib() {
        // Exactly 10 MiB ASCII
        let mut exact: Vec<u16> = vec![b'a' as u16; MAX_UNICODE_TEXT_BYTES];
        exact.push(0);
        let decoded = decode_bounded_unicode_text(&exact);
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap().len(), MAX_UNICODE_TEXT_BYTES);

        // 10 MiB + 1 byte ASCII -> over limit -> rejected
        let mut over: Vec<u16> = vec![b'a' as u16; MAX_UNICODE_TEXT_BYTES + 1];
        over.push(0);
        assert_eq!(decode_bounded_unicode_text(&over), None);

        // Multi-byte UTF-8 boundary check: each character is 3 UTF-8 bytes (e.g., '€' is 3 bytes)
        let char_count = MAX_UNICODE_TEXT_BYTES / 3;
        let mut euro: Vec<u16> = vec![0x20AC; char_count];
        euro.push(0);
        let decoded = decode_bounded_unicode_text(&euro);
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap().len(), char_count * 3);

        // Adding enough euro chars to cross 10 MiB
        let mut over_euro: Vec<u16> = vec![0x20AC; char_count + 2];
        over_euro.push(0);
        assert_eq!(decode_bounded_unicode_text(&over_euro), None);
    }

    #[test]
    fn payload_evaluation_precedence_and_eligibility() {
        // Filesystem payload with valid paths is supported
        let fs = EvaluatedPayload::Filesystem(vec![Some("C:\\test.txt".into())]);
        assert!(fs.is_supported());

        // Filesystem payload with only None/empty paths is NOT supported
        let empty_fs = EvaluatedPayload::Filesystem(vec![None, None]);
        assert!(!empty_fs.is_supported());

        // Non-empty Unicode text is supported
        let text = EvaluatedPayload::UnicodeText("Sample dropped text".into());
        assert!(text.is_supported());

        // Unsupported is not supported
        let unsupported = EvaluatedPayload::Unsupported;
        assert!(!unsupported.is_supported());
    }

    #[test]
    fn evaluate_payload_precedence_rules() {
        // Filesystem valid beats virtual file and text
        assert_eq!(
            evaluate_payload_precedence(
                Some(vec![Some("C:\\test.txt".into())]),
                true,
                TextExtraction::Valid("companion text".into())
            ),
            EvaluatedPayload::Filesystem(vec![Some("C:\\test.txt".into())])
        );

        // Offered filesystem but invalid/empty does NOT fall through to text or virtual
        assert_eq!(
            evaluate_payload_precedence(
                Some(vec![None]),
                false,
                TextExtraction::Valid("companion text".into())
            ),
            EvaluatedPayload::Unsupported
        );

        // Virtual file format offered without filesystem does NOT fall through to companion text
        assert_eq!(
            evaluate_payload_precedence(
                None,
                true,
                TextExtraction::Valid("https://example.com/image.png".into())
            ),
            EvaluatedPayload::Unsupported
        );

        // Pure unicode text with no higher priority formats is accepted
        assert_eq!(
            evaluate_payload_precedence(
                None,
                false,
                TextExtraction::Valid("plain dropped text".into())
            ),
            EvaluatedPayload::UnicodeText("plain dropped text".into())
        );

        // Pure invalid/empty/over-limit unicode text returns InvalidUnicodeText
        assert_eq!(
            evaluate_payload_precedence(None, false, TextExtraction::Invalid),
            EvaluatedPayload::InvalidUnicodeText
        );

        // No supported payload
        assert_eq!(
            evaluate_payload_precedence(None, false, TextExtraction::None),
            EvaluatedPayload::Unsupported
        );
    }
}
