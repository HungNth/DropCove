use std::{
    iter::once,
    os::windows::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, WIN32_ERROR},
        Globalization::{CompareStringOrdinal, CSTR_EQUAL},
        Storage::FileSystem::{
            GetFileAttributesW, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES,
        },
    },
};

use crate::shelf::Availability;

pub(crate) fn ordinal_equal(left: &[u16], right: &[u16]) -> bool {
    // SAFETY: Incoming paths have bounded lengths and both slices live through this call.
    unsafe { CompareStringOrdinal(left, right, true) == CSTR_EQUAL }
}

fn attributes(path: &Path) -> Result<u32, WIN32_ERROR> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    // SAFETY: `wide` is a live, NUL-terminated UTF-16 buffer. Read the error immediately.
    unsafe {
        let attributes = GetFileAttributesW(PCWSTR(wide.as_ptr()));
        if attributes == INVALID_FILE_ATTRIBUTES {
            Err(GetLastError())
        } else {
            Ok(attributes)
        }
    }
}

pub(crate) fn inspect(path: &str) -> (Availability, bool) {
    if path.is_empty() || path.contains('\0') {
        return (Availability::Unavailable, false);
    }
    match attributes(Path::new(path)) {
        Ok(value) => (
            Availability::Available,
            value & FILE_ATTRIBUTE_DIRECTORY.0 != 0,
        ),
        Err(error) if error == ERROR_FILE_NOT_FOUND || error == ERROR_PATH_NOT_FOUND => {
            let mut root = PathBuf::new();
            for component in Path::new(path).components() {
                match component {
                    Component::Prefix(_) | Component::RootDir => root.push(component.as_os_str()),
                    _ => break,
                }
            }
            let availability = if !root.as_os_str().is_empty() && attributes(&root).is_ok() {
                Availability::Missing
            } else {
                Availability::Unavailable
            };
            (availability, false)
        }
        Err(_) => (Availability::Unavailable, false),
    }
}

pub(crate) fn classify(path: &str) -> Availability {
    inspect(path).0
}
