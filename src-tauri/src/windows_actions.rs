//! Native Windows integration for item actions: Unicode clipboard and File Explorer.

use std::{
    ffi::OsStr,
    iter::once,
    os::windows::ffi::OsStrExt,
    path::{Component, Path},
    ptr,
};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GlobalFree, HWND},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            Ole::CF_UNICODETEXT,
        },
        UI::Shell::{
            ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteW,
        },
        UI::WindowsAndMessaging::SW_SHOWNORMAL,
    },
};

struct ScopedCom;

impl ScopedCom {
    fn init() -> Result<Self, String> {
        // SAFETY: This worker thread initializes COM and balances only successful initialization.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|error| format!("could not initialize Windows Shell: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ScopedCom {
    fn drop(&mut self) {
        // SAFETY: Balances CoInitializeEx for this thread.
        unsafe {
            CoUninitialize();
        }
    }
}

struct ScopedClipboard;

impl ScopedClipboard {
    fn open(owner: HWND) -> Result<Self, String> {
        // SAFETY: A live application window owns clipboard data after EmptyClipboard.
        unsafe { OpenClipboard(Some(owner)) }
            .map_err(|error| format!("could not open Windows clipboard: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ScopedClipboard {
    fn drop(&mut self) {
        // SAFETY: Balances successful OpenClipboard.
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub(crate) fn write_unicode_clipboard(owner: HWND, text: &str) -> Result<(), String> {
    let _clip = ScopedClipboard::open(owner)?;

    unsafe {
        // SAFETY: Clipboard is open; EmptyClipboard clears current content and assigns ownership.
        EmptyClipboard().map_err(|e| format!("could not clear clipboard: {e}"))?;

        let wide: Vec<u16> = text.encode_utf16().chain(once(0)).collect();
        let bytes_len = wide.len() * std::mem::size_of::<u16>();

        // SAFETY: GMEM_MOVEABLE allocation for CF_UNICODETEXT clipboard data.
        let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes_len)
            .map_err(|e| format!("could not allocate global memory for clipboard: {e}"))?;

        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            let _ = GlobalFree(Some(hmem));
            return Err("could not lock global memory for clipboard".to_string());
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr.cast::<u16>(), wide.len());
        let _ = GlobalUnlock(hmem);

        // SAFETY: SetClipboardData transfers ownership of hmem to the system upon success.
        if let Err(e) = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(windows::Win32::Foundation::HANDLE(hmem.0))) {
            let _ = GlobalFree(Some(hmem));
            return Err(format!("could not set clipboard data: {e}"));
        }
    }

    Ok(())
}

fn is_root_path(path: &Path) -> bool {
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Prefix(_)), Some(Component::RootDir)) => {
            components.all(|component| matches!(component, Component::CurDir))
        }
        (Some(Component::RootDir), None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_root_path_handles_drives_trailing_separators_and_non_roots() {
        assert!(is_root_path(Path::new("C:\\")));
        assert!(is_root_path(Path::new("C:/")));
        assert!(is_root_path(Path::new("C:\\\\")));
        assert!(is_root_path(Path::new("C:\\.")));
        assert!(is_root_path(Path::new("\\\\server\\share\\")));
        assert!(!is_root_path(Path::new("C:\\folder")));
        assert!(!is_root_path(Path::new("C:\\folder\\")));
        assert!(!is_root_path(Path::new("C:\\folder\\file.txt")));
    }
}

pub(crate) fn show_in_explorer(path_str: &str, _is_folder: bool) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("path does not exist: `{path_str}`"));
    }

    let _com = ScopedCom::init()?;

    if is_root_path(path) {
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
        let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(once(0)).collect();
        // SAFETY: ShellExecuteW opens the filesystem root in File Explorer without a selectable parent.
        unsafe {
            let result = ShellExecuteW(
                None,
                PCWSTR(operation.as_ptr()),
                PCWSTR(wide_path.as_ptr()),
                PCWSTR(ptr::null()),
                PCWSTR(ptr::null()),
                SW_SHOWNORMAL,
            );
            if (result.0 as isize) <= 32 {
                return Err(format!("ShellExecuteW failed for `{path_str}`"));
            }
        }
        return Ok(());
    }

    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    // SAFETY: ILCreateFromPathW creates an ITEMIDLIST for the given path.
    unsafe {
        let pidl = ILCreateFromPathW(PCWSTR(wide_path.as_ptr()));
        if pidl.is_null() {
            return Err(format!("could not create shell ID list for `{path_str}`"));
        }

        struct PidlGuard(*mut windows::Win32::UI::Shell::Common::ITEMIDLIST);
        impl Drop for PidlGuard {
            fn drop(&mut self) {
                // SAFETY: ILFree frees an ITEMIDLIST allocated by the Shell.
                unsafe {
                    ILFree(Some(self.0));
                }
            }
        }
        let _guard = PidlGuard(pidl);

        // SAFETY: When cidl == 0, pidlFolder specifies a single item; SHOpenFolderAndSelectItems
        // opens the parent folder and selects that item.
        SHOpenFolderAndSelectItems(pidl, None, 0)
            .map_err(|e| format!("SHOpenFolderAndSelectItems failed: {e}"))?;
    }

    Ok(())
}
