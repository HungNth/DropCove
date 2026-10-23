use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use windows::{
    core::{implement, Error as WindowsError, BOOL, HRESULT, PCWSTR},
    Win32::{
        Foundation::{
            DATA_S_SAMEFORMATETC, DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS,
            DV_E_FORMATETC, E_NOTIMPL, E_POINTER, GlobalFree, HGLOBAL, S_FALSE, S_OK,
        },
        System::{
            Com::{
                DVASPECT_CONTENT, FORMATETC, IAdviseSink, IDataObject, IDataObject_Impl,
                IEnumFORMATETC, IEnumFORMATETC_Impl, IEnumSTATDATA, STGMEDIUM, STGMEDIUM_0,
                TYMED_HGLOBAL,
            },
            Ole::{
                DoDragDrop, IDropSource, IDropSource_Impl, OleInitialize, OleUninitialize,
                CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
            },
            SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS, SFGAO_FILESYSTEM, SFGAO_VALIDATE},
        },
        UI::Shell::{
            BHID_DataObject, ILCreateFromPathW, ILFree, SHCreateShellItemArrayFromIDLists,
            SIATTRIBFLAGS_AND,
        },
    },
};

use crate::shelf::DragOutcome;

#[derive(Debug, Error)]
pub(crate) enum StorageDragError {
    #[error("Windows could not resolve the storage path")]
    InvalidPath,
    #[error("Windows storage drag failed: {0}")]
    Windows(#[from] WindowsError),
    #[error("Windows storage drag returned unexpected status {0:#x}")]
    UnexpectedStatus(i32),
    #[error("another native drag is already active")]
    AlreadyDragging,
}

#[derive(Default)]
pub(crate) struct DragSession(AtomicBool);

impl DragSession {
    pub(crate) fn is_active(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    fn enter(&self) -> Result<DragGuard<'_>, StorageDragError> {
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| StorageDragError::AlreadyDragging)?;
        Ok(DragGuard(&self.0))
    }
}

struct DragGuard<'a>(&'a AtomicBool);

impl Drop for DragGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[implement(IDropSource)]
struct DropSource;

#[allow(non_snake_case)]
impl IDropSource_Impl for DropSource_Impl {
    fn QueryContinueDrag(&self, escape_pressed: BOOL, key_state: MODIFIERKEYS_FLAGS) -> HRESULT {
        if escape_pressed.as_bool() {
            DRAGDROP_S_CANCEL
        } else if (key_state & MK_LBUTTON) == MODIFIERKEYS_FLAGS(0) {
            DRAGDROP_S_DROP
        } else {
            S_OK
        }
    }

    fn GiveFeedback(&self, _effect: DROPEFFECT) -> HRESULT {
        DRAGDROP_S_USEDEFAULTCURSORS
    }
}
#[implement(IEnumFORMATETC)]
struct FormatEnumerator {
    formats: Vec<FORMATETC>,
    index: std::sync::atomic::AtomicUsize,
}

#[allow(non_snake_case)]
impl IEnumFORMATETC_Impl for FormatEnumerator_Impl {
    fn Next(&self, celt: u32, rgelt: *mut FORMATETC, pceltFetched: *mut u32) -> HRESULT {
        unsafe {
            if rgelt.is_null() || (celt > 1 && pceltFetched.is_null()) {
                return E_POINTER;
            }
            let mut count = 0u32;
            while count < celt {
                let current = self.index.fetch_add(1, Ordering::SeqCst);
                if current >= self.formats.len() {
                    break;
                }
                *rgelt.add(count as usize) = self.formats[current];
                count += 1;
            }
            if !pceltFetched.is_null() {
                *pceltFetched = count;
            }
            if count == celt {
                S_OK
            } else {
                S_FALSE
            }
        }
    }

    fn Skip(&self, celt: u32) -> windows::core::Result<()> {
        let new_idx = (self.index.load(Ordering::SeqCst) + celt as usize).min(self.formats.len());
        self.index.store(new_idx, Ordering::SeqCst);
        Ok(())
    }

    fn Reset(&self) -> windows::core::Result<()> {
        self.index.store(0, Ordering::SeqCst);
        Ok(())
    }
    fn Clone(&self) -> windows::core::Result<IEnumFORMATETC> {
        Ok(FormatEnumerator {
            formats: self.formats.clone(),
            index: std::sync::atomic::AtomicUsize::new(self.index.load(Ordering::SeqCst)),
        }
        .into())
    }
}

#[implement(IDataObject)]
struct UnicodeTextDataObject {
    wide: Vec<u16>,
}

#[allow(non_snake_case)]
impl IDataObject_Impl for UnicodeTextDataObject_Impl {
    fn GetData(&self, pformatetcIn: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
        unsafe {
            if pformatetcIn.is_null() {
                return Err(E_POINTER.into());
            }
            let format = &*pformatetcIn;
            if format.cfFormat != CF_UNICODETEXT.0 || (format.tymed & TYMED_HGLOBAL.0 as u32) == 0 {
                return Err(DV_E_FORMATETC.into());
            }
            let bytes_len = self.wide.len() * std::mem::size_of::<u16>();
            let hglobal = windows::Win32::System::Memory::GlobalAlloc(
                windows::Win32::System::Memory::GMEM_MOVEABLE,
                bytes_len,
            )?;
            let ptr = windows::Win32::System::Memory::GlobalLock(hglobal);
            if ptr.is_null() {
                let _ = GlobalFree(Some(HGLOBAL(hglobal.0)));
                return Err(windows::core::Error::from_hresult(windows::Win32::Foundation::E_OUTOFMEMORY));
            }
            std::ptr::copy_nonoverlapping(self.wide.as_ptr(), ptr as *mut u16, self.wide.len());
            let _ = windows::Win32::System::Memory::GlobalUnlock(hglobal);

            Ok(STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: STGMEDIUM_0 { hGlobal: hglobal },
                pUnkForRelease: std::mem::zeroed(),
            })
        }
    }

    fn GetDataHere(&self, _pformatetc: *const FORMATETC, _pmedium: *mut STGMEDIUM) -> windows::core::Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
        unsafe {
            if pformatetc.is_null() {
                return E_POINTER;
            }
            let format = &*pformatetc;
            if format.cfFormat == CF_UNICODETEXT.0 && (format.tymed & TYMED_HGLOBAL.0 as u32) != 0 {
                S_OK
            } else {
                DV_E_FORMATETC
            }
        }
    }

    fn GetCanonicalFormatEtc(&self, _pformatectIn: *const FORMATETC, pformatetcOut: *mut FORMATETC) -> HRESULT {
        unsafe {
            if !pformatetcOut.is_null() {
                (*pformatetcOut).ptd = std::ptr::null_mut();
            }
        }
        DATA_S_SAMEFORMATETC
    }

    fn SetData(&self, _pformatetc: *const FORMATETC, _pmedium: *const STGMEDIUM, _frelease: windows::core::BOOL) -> windows::core::Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn EnumFormatEtc(&self, dwdirection: u32) -> windows::core::Result<IEnumFORMATETC> {
        if dwdirection == windows::Win32::System::Com::DATADIR_GET.0 as u32 {
            let format = FORMATETC {
                cfFormat: CF_UNICODETEXT.0,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            };
            Ok(FormatEnumerator {
                formats: vec![format],
                index: std::sync::atomic::AtomicUsize::new(0),
            }
            .into())
        } else {
            Err(E_NOTIMPL.into())
        }
    }

    fn DAdvise(&self, _pformatetc: *const FORMATETC, _advf: u32, _padvsink: windows::core::Ref<'_, IAdviseSink>) -> windows::core::Result<u32> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }

    fn DUnadvise(&self, _dwconnection: u32) -> windows::core::Result<()> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }

    fn EnumDAdvise(&self) -> windows::core::Result<IEnumSTATDATA> {
        Err(windows::core::Error::from_hresult(E_NOTIMPL))
    }
}

struct OleGuard;

impl OleGuard {
    fn initialize() -> Result<Self, WindowsError> {
        // SAFETY: OLE uses this thread's STA; the guard balances either successful initialization result.
        unsafe { OleInitialize(None)? };
        Ok(Self)
    }
}

impl Drop for OleGuard {
    fn drop(&mut self) {
        // SAFETY: `OleGuard` exists only after a successful `OleInitialize` on this thread.
        unsafe { OleUninitialize() };
    }
}

struct ItemIdLists(Vec<*const windows::Win32::UI::Shell::Common::ITEMIDLIST>);

impl Drop for ItemIdLists {
    fn drop(&mut self) {
        for &item in &self.0 {
            // SAFETY: Each PIDL was allocated by ILCreateFromPathW and is owned by this guard.
            unsafe {
                ILFree(Some(item));
            }
        }
    }
}

pub(crate) fn perform_storage_drag<'a>(
    session: &DragSession,
    paths: impl ExactSizeIterator<Item = &'a str>,
    unicode_text: Option<&str>,
) -> Result<DragOutcome, StorageDragError> {
    if paths.len() == 0 {
        return Err(StorageDragError::InvalidPath);
    }
    let _session = session.enter()?;
    let _ole = OleGuard::initialize()?;
    unsafe {
        let data_object: IDataObject = if let Some(text) = unicode_text {
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            UnicodeTextDataObject { wide }.into()
        } else {
            let mut item_ids = ItemIdLists(Vec::with_capacity(paths.len()));
            let mut wide_path = Vec::new();
            for path in paths {
                if path.is_empty() || path.contains('\0') {
                    return Err(StorageDragError::InvalidPath);
                }
                wide_path.clear();
                // Shell PIDL parsing rejects Win32 verbatim prefixes used by canonical profile paths.
                if let Some(unc) = path.strip_prefix("\\\\?\\UNC\\") {
                    wide_path.extend("\\\\".encode_utf16());
                    wide_path.extend(unc.encode_utf16());
                } else {
                    wide_path.extend(path.strip_prefix("\\\\?\\").unwrap_or(path).encode_utf16());
                }
                wide_path.push(0);
                let item_id = ILCreateFromPathW(PCWSTR::from_raw(wide_path.as_ptr()));
                if item_id.is_null() {
                    return Err(StorageDragError::InvalidPath);
                }
                item_ids.0.push(item_id.cast_const());
            }
            let item_array = SHCreateShellItemArrayFromIDLists(&item_ids.0)?;
            let attributes =
                item_array.GetAttributes(SIATTRIBFLAGS_AND, SFGAO_VALIDATE | SFGAO_FILESYSTEM)?;
            if attributes & SFGAO_FILESYSTEM != SFGAO_FILESYSTEM {
                return Err(StorageDragError::InvalidPath);
            }
            item_array.BindToHandler(None, &BHID_DataObject)?
        };
        let drop_source: IDropSource = DropSource.into();
        let mut effect = DROPEFFECT_NONE;
        let status = DoDragDrop(&data_object, &drop_source, DROPEFFECT_COPY, &mut effect);
        if status == DRAGDROP_S_DROP {
            return Ok(if effect.contains(DROPEFFECT_COPY) {
                DragOutcome::Copy
            } else {
                DragOutcome::Rejected
            });
        }
        if status == DRAGDROP_S_CANCEL {
            return Ok(DragOutcome::Canceled);
        }
        if status.is_err() {
            return Err(StorageDragError::Windows(WindowsError::from_hresult(
                status,
            )));
        }
        Err(StorageDragError::UnexpectedStatus(status.0))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Ole::DROPEFFECT_MOVE;

    #[test]
    fn own_drag_session_rejects_storage_drop_as_intake() {
        let session = DragSession::default();

        // When drag session is active (application-owned drag), offered COPY + valid storage paths -> DROPEFFECT_NONE
        {
            let _guard = session.enter().unwrap();
            assert!(session.is_active());
            assert_eq!(
                crate::windows_drop::evaluate_storage_drop(
                    session.is_active(),
                    DROPEFFECT_COPY,
                    true
                ),
                DROPEFFECT_NONE
            );
        }

        // When guard drops (external drag), offered COPY + valid paths -> DROPEFFECT_COPY
        assert!(!session.is_active());
        assert_eq!(
            crate::windows_drop::evaluate_storage_drop(
                session.is_active(),
                DROPEFFECT_COPY,
                true
            ),
            DROPEFFECT_COPY
        );

        // Unsupported payloads (no valid paths) -> DROPEFFECT_NONE
        assert_eq!(
            crate::windows_drop::evaluate_storage_drop(
                session.is_active(),
                DROPEFFECT_COPY,
                false
            ),
            DROPEFFECT_NONE
        );

        // Move only effect without Copy -> DROPEFFECT_NONE
        assert_eq!(
            crate::windows_drop::evaluate_storage_drop(
                session.is_active(),
                DROPEFFECT_MOVE,
                true
            ),
            DROPEFFECT_NONE
        );
    }
    #[test]
    fn unicode_text_data_object_serves_cf_unicodetext_without_shell_file_formats() {
        use windows::Win32::System::Com::*;


        unsafe {
            let _ = OleInitialize(None);
            let sample_text = "DropCove text payload";
            let wide: Vec<u16> = sample_text.encode_utf16().chain(std::iter::once(0)).collect();
            let data_object: IDataObject = UnicodeTextDataObject { wide }.into();

            // QueryGetData accepts CF_UNICODETEXT
            let text_format = FORMATETC {
                cfFormat: CF_UNICODETEXT.0,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            };
            assert!(data_object.QueryGetData(&text_format).is_ok());

            // QueryGetData rejects CF_HDROP
            let hdrop_format = FORMATETC {
                cfFormat: windows::Win32::System::Ole::CF_HDROP.0,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            };
            assert!(data_object.QueryGetData(&hdrop_format).is_err());

            // GetData extracts exact unicode text
            let medium = data_object.GetData(&text_format).unwrap();
            assert_eq!(medium.tymed, TYMED_HGLOBAL.0 as u32);
            let ptr = windows::Win32::System::Memory::GlobalLock(medium.u.hGlobal);
            let text_slice = std::slice::from_raw_parts(ptr as *const u16, sample_text.encode_utf16().count());
            let recovered = String::from_utf16(text_slice).unwrap();
            assert_eq!(recovered, sample_text);
            let _ = windows::Win32::System::Memory::GlobalUnlock(medium.u.hGlobal);
            windows::Win32::System::Ole::ReleaseStgMedium(&medium as *const _ as *mut _);

            // EnumFormatEtc only returns CF_UNICODETEXT
            let enumerator = data_object.EnumFormatEtc(windows::Win32::System::Com::DATADIR_GET.0 as u32).unwrap();
            let mut formats = [FORMATETC::default(); 8];
            let mut fetched = 0u32;
            assert_eq!(enumerator.Next(&mut formats[..1], Some(&mut fetched)), S_OK);
            assert_eq!(fetched, 1);
            assert_eq!(formats[0].cfFormat, CF_UNICODETEXT.0);

            OleUninitialize();
        }
    }
}
