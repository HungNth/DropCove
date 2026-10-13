use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use windows::{
    core::{implement, Error as WindowsError, BOOL, HRESULT, PCWSTR},
    Win32::{
        Foundation::{DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS, S_OK},
        System::{
            Com::IDataObject,
            Ole::{
                DoDragDrop, IDropSource, IDropSource_Impl, OleInitialize, OleUninitialize,
                DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
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
) -> Result<DragOutcome, StorageDragError> {
    if paths.len() == 0 {
        return Err(StorageDragError::InvalidPath);
    }
    let _session = session.enter()?;
    let _ole = OleGuard::initialize()?;
    let mut item_ids = ItemIdLists(Vec::with_capacity(paths.len()));
    let mut wide_path = Vec::new();

    // SAFETY: Buffers are NUL-terminated and live through each synchronous parsing call.
    // ItemIdLists frees every PIDL; COM wrappers own the returned array/data-object references.
    unsafe {
        for path in paths {
            if path.is_empty() || path.contains('\0') {
                return Err(StorageDragError::InvalidPath);
            }
            wide_path.clear();
            wide_path.extend(path.encode_utf16());
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
        let data_object: IDataObject = item_array.BindToHandler(None, &BHID_DataObject)?;
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
