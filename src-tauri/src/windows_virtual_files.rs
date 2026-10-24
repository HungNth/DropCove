use std::{fs, io::Write, mem::{offset_of, size_of}, path::PathBuf};

use windows::{
    core::{GUID, PCWSTR},
    Win32::{
        Globalization::{MultiByteToWideChar, CP_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS},
        System::{Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL, TYMED_ISTREAM}, DataExchange::RegisterClipboardFormatW, Memory::{GlobalLock, GlobalSize, GlobalUnlock}},
        UI::Shell::{CFSTR_FILECONTENTS, CFSTR_FILEDESCRIPTORW, CFSTR_INETURLA, CFSTR_INETURLW, FILEDESCRIPTORW, FILEGROUPDESCRIPTORW},
    },
};

use crate::{managed_files::ManagedStore, shelf::{DropResult, ShelfState}, windows_drop::StorageMedium, website_images::{self, MAX_DESCRIPTORS, MAX_DROP_BYTES, MAX_IMAGE_BYTES}};

fn format(name: PCWSTR, index: i32, medium: u32) -> FORMATETC {
    // SAFETY: All callers provide static NUL-terminated clipboard-format names.
    let id = unsafe { RegisterClipboardFormatW(name) };
    FORMATETC { cfFormat: id as u16, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: index, tymed: medium }
}

fn global_bytes<T>(medium: &StorageMedium, use_bytes: impl FnOnce(&[u8]) -> Result<T, String>) -> Result<T, String> {
    if medium.0.tymed != TYMED_HGLOBAL.0 as u32 { return Err("Invalid image descriptor memory.".into()); }
    // SAFETY: The medium owns the HGLOBAL for this scope; GlobalLock is balanced before return.
    unsafe {
        let handle = medium.0.u.hGlobal;
        let len = GlobalSize(handle);
        if len == 0 || len > isize::MAX as usize { return Err("Invalid image descriptor memory.".into()); }
        let ptr = GlobalLock(handle);
        if ptr.is_null() { return Err("Invalid image descriptor memory.".into()); }
        struct Unlock(windows::Win32::Foundation::HGLOBAL);
        impl Drop for Unlock { fn drop(&mut self) { unsafe { let _ = GlobalUnlock(self.0); } } }
        let _unlock = Unlock(handle);
        use_bytes(std::slice::from_raw_parts(ptr.cast::<u8>(), len))
    }
}

pub(crate) fn descriptors(data: &IDataObject) -> Result<(Vec<Option<String>>, usize), String> {
    let request = format(CFSTR_FILEDESCRIPTORW, -1, TYMED_HGLOBAL.0 as u32);
    // SAFETY: GetData returns an owned medium released by StorageMedium on this thread.
    let medium = StorageMedium(unsafe { data.GetData(&request) }.map_err(|_| "Invalid image descriptors.".to_string())?);
    global_bytes(&medium, |bytes| {
        let header = offset_of!(FILEGROUPDESCRIPTORW, fgd);
        let count = u32::from_ne_bytes(bytes.get(..4).ok_or("Truncated image descriptors.")?.try_into().map_err(|_| "Truncated image descriptors.")?) as usize;
        let required = count.checked_mul(size_of::<FILEDESCRIPTORW>()).and_then(|n| n.checked_add(header)).ok_or("Invalid image descriptor count.")?;
        if count == 0 || required > bytes.len() { return Err("Invalid image descriptor count.".into()); }
        let mut entries = Vec::with_capacity(count.min(MAX_DESCRIPTORS));
        for index in 0..count.min(MAX_DESCRIPTORS) {
            // SAFETY: Required allocation is checked above. Descriptor memory may be unaligned.
            let descriptor = unsafe { bytes.as_ptr().add(header + index * size_of::<FILEDESCRIPTORW>()).cast::<FILEDESCRIPTORW>().read_unaligned() };
            let name = descriptor.cFileName;
            let extracted = name.iter().position(|&c| c == 0).and_then(|end| String::from_utf16(&name[..end]).ok());
            entries.push(Some(extracted.unwrap_or_default()));
        }
        Ok((entries, count))
    })
}

pub(crate) fn has_virtual_family(data: &IDataObject) -> bool {
    let request = format(CFSTR_FILEDESCRIPTORW, -1, TYMED_HGLOBAL.0 as u32);
    // SAFETY: QueryGetData only probes support on the live window-thread data object.
    if unsafe { data.QueryGetData(&request) }.is_ok() {
        return descriptors(data).map(|(names, _)| names.iter().any(|name| name.as_ref().is_none_or(|n| !n.to_ascii_lowercase().ends_with(".url")))).unwrap_or(true);
    }
    false
}

fn source_url(data: &IDataObject) -> Option<String> {
    for (name, wide) in [(CFSTR_INETURLW, true), (CFSTR_INETURLA, false)] {
        let request = format(name, -1, TYMED_HGLOBAL.0 as u32);
        let Ok(raw) = (unsafe { data.GetData(&request) }) else { continue; };
        let medium = StorageMedium(raw);
        let value = global_bytes(&medium, |bytes| {
            if bytes.len() > 128 * 1024 { return Err("Source URL is too large.".into()); }
            let units = if wide {
                if bytes.len() % 2 != 0 { return Err("Invalid source URL.".into()); }
                let units = bytes.chunks_exact(2).map(|b| u16::from_ne_bytes([b[0], b[1]])).collect::<Vec<_>>();
                let end = units.iter().position(|&c| c == 0).ok_or("Invalid source URL.")?;
                units[..end].to_vec()
            } else {
                let end = bytes.iter().position(|&b| b == 0).ok_or("Invalid source URL.")?;
                let bytes = &bytes[..end];
                // SAFETY: Both calls use the checked input slice and a correctly sized output slice.
                let count = unsafe { MultiByteToWideChar(CP_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0), bytes, None) };
                if count <= 0 { return Err("Invalid source URL.".into()); }
                let mut units = vec![0u16; count as usize];
                if unsafe { MultiByteToWideChar(CP_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0), bytes, Some(&mut units)) } != count { return Err("Invalid source URL.".into()); }
                units
            };
            String::from_utf16(&units).map_err(|_| "Invalid source URL.".into())
        });
        if let Ok(value) = value {
            if let Ok(parsed) = tauri::Url::parse(&value) {
                if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() { return Some(parsed.to_string()); }
            }
        }
    }
    None
}

struct StagedEntry { relative: String, path: PathBuf, name: String }

pub(crate) struct ImageDrop {
    store: ManagedStore,
    entries: Vec<StagedEntry>,
    skipped: usize,
    source: Option<String>,
    timestamp: String,
}

impl Drop for ImageDrop {
    fn drop(&mut self) {
        for entry in &self.entries {
            // Unowned staging stays discoverable by startup reconciliation if deletion is denied.
            if let Err(error) = self.store.delete_managed_target(&entry.relative) { eprintln!("image staging cleanup failed: {error}"); }
        }
    }
}

pub(crate) fn extract(data: &IDataObject, store: ManagedStore) -> Result<ImageDrop, String> {
    let (names, count) = descriptors(data)?;
    store.init_directories().map_err(|e| format!("Image I/O failed: {e}"))?;
    let mut result = ImageDrop { store, entries: Vec::with_capacity(names.len()), skipped: count.saturating_sub(names.len()), source: source_url(data), timestamp: crate::managed_files::format_local_timestamp_now() };
    for (index, name) in names.into_iter().enumerate() {
        let Some(name) = name else { result.skipped += 1; continue; };
        let request = format(CFSTR_FILECONTENTS, index as i32, (TYMED_ISTREAM.0 | TYMED_HGLOBAL.0) as u32);
        let medium = StorageMedium(unsafe { data.GetData(&request) }.map_err(|e| format!("Image I/O failed: {e}"))?);
        if medium.0.tymed != TYMED_ISTREAM.0 as u32 && medium.0.tymed != TYMED_HGLOBAL.0 as u32 { result.skipped += 1; continue; }
        let id = GUID::new().map_err(|e| format!("Image I/O failed: {e}"))?;
        let relative = ManagedStore::staging_rel_path(&format!("{:032x}", id.to_u128()));
        let path = result.store.full_path(&relative).map_err(|e| format!("Image I/O failed: {e}"))?;
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e| format!("Image I/O failed: {e}"))?;
        result.entries.push(StagedEntry { relative, path, name });
        let mut written = 0u64;
        if medium.0.tymed == TYMED_ISTREAM.0 as u32 {
            // SAFETY: Medium owns the COM stream and remains alive through all reads on this thread.
            let stream = unsafe { &*medium.0.u.pstm }.as_ref().ok_or("Image I/O failed: missing stream.")?;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let mut read = 0;
                let status = unsafe { stream.Read(buffer.as_mut_ptr().cast(), buffer.len() as u32, Some(&mut read)) };
                status.ok().map_err(|e| format!("Image I/O failed: {e}"))?;
                if read as usize > buffer.len() { return Err("Image I/O failed: invalid stream read count.".into()); }
                if read == 0 { break; }
                written += u64::from(read);
                if written > MAX_IMAGE_BYTES { break; }
                file.write_all(&buffer[..read as usize]).map_err(|e| format!("Image I/O failed: {e}"))?;
            }
        } else {
            global_bytes(&medium, |bytes| {
                written = bytes.len() as u64;
                if written <= MAX_IMAGE_BYTES { file.write_all(bytes).map_err(|e| format!("Image I/O failed: {e}"))?; }
                Ok(())
            })?;
        }
        file.sync_all().map_err(|e| format!("Image I/O failed: {e}"))?;
        drop(file);
        if written > MAX_IMAGE_BYTES || written == 0 {
            let entry = result.entries.pop().ok_or("Image I/O failed: staging entry missing.")?;
            result.store.delete_managed_target(&entry.relative).map_err(|e| format!("Image I/O failed: {e}"))?;
            result.skipped += 1;
        }
    }
    Ok(result)
}

impl ImageDrop {
    pub(crate) fn accept(self, state: &ShelfState) -> Result<DropResult, String> {
        self.accept_with_security(state, website_images::secure_image)
    }

    fn accept_with_security(mut self, state: &ShelfState, mut secure: impl FnMut(&std::path::Path, Option<&str>, &str) -> Result<(), String>) -> Result<DropResult, String> {
        let mut total = 0u64;
        let mut candidates = Vec::with_capacity(self.entries.len());
        let mut generated_names: Option<std::collections::HashSet<String>> = None;
        let timestamp = &self.timestamp;
        for entry in &self.entries {
            let before = fs::metadata(&entry.path).map_err(|e| format!("Image I/O failed: {e}"))?.len();
            let kind = match website_images::validate_image(&entry.path) {
                Ok(kind) => kind,
                Err(error) if error.is_io() => return Err(format!("Image I/O failed: {error}")),
                Err(_) => { self.skipped += 1; continue; }
            };
            if total + before > MAX_DROP_BYTES { self.skipped += 1; continue; }
            let name = match website_images::sanitize_image_name(&entry.name, kind) {
                Some(name) => name,
                None => {
                    if generated_names.is_none() {
                        let snapshot = state.snapshot().map_err(|e| format!("Image persistence failed: {e}"))?;
                        generated_names = Some(snapshot.batches.into_iter().flat_map(|batch| batch.items).map(|item| item.name.to_lowercase()).collect());
                    }
                    let names = generated_names.as_mut().ok_or("Image naming failed.")?;
                    names.extend(candidates.iter().map(|(_, name): &(String, String)| name.to_lowercase()));
                    let mut suffix = 1;
                    loop {
                        let ending = if suffix == 1 { String::new() } else { format!("-{suffix}") };
                        let candidate = format!("Dropped image {timestamp}{ending}.{}", kind.extension());
                        if names.insert(candidate.to_lowercase()) { break candidate; }
                        suffix += 1;
                    }
                }
            };
            secure(&entry.path, self.source.as_deref(), &name).map_err(|e| format!("Image Attachment Services failed: {e}"))?;
            let after = fs::metadata(&entry.path).map_err(|e| format!("Image Attachment Services failed: {e}"))?.len();
            if after > MAX_IMAGE_BYTES || total + after > MAX_DROP_BYTES { return Err("Image Attachment Services produced over-limit content.".into()); }
            let final_kind = website_images::validate_image(&entry.path).map_err(|e| format!("Image Attachment Services produced invalid content: {e}"))?;
            total += after;
            let name = website_images::sanitize_image_name(&name, final_kind).ok_or("Image naming failed after attachment validation.")?;
            candidates.push((entry.relative.clone(), name));
        }
        if candidates.is_empty() {
            return Ok(DropResult {
                snapshot: None,
                skipped: self.skipped,
                message: Some("Image drop is unsupported, invalid, or exceeds image limits.".into()),
            });
        }
        state.accept_image_files(candidates, self.skipped).map_err(|e| format!("Image persistence failed: {e}"))
    }
}

#[cfg(test)]
#[path = "windows_virtual_files_tests.rs"]
mod tests;
