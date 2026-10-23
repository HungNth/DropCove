use std::{fs, io, io::Write, os::windows::fs::MetadataExt, path::{Path, PathBuf}};

pub(crate) const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;

pub(crate) struct ManagedStore {
    root: PathBuf,
}

impl ManagedStore {
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn managed_root(&self) -> PathBuf { self.root.join("managed") }
    pub(crate) fn staging_root(&self) -> PathBuf { self.root.join("staging") }

    pub(crate) fn ensure_no_reparse_ancestors(&self, path: &Path) -> io::Result<()> {
        if !path.starts_with(&self.root) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Managed File path is outside its profile"));
        }
        for ancestor in path.ancestors() {
            if !ancestor.starts_with(&self.root) { break; }
            match fs::symlink_metadata(ancestor) {
                Ok(metadata) if metadata.file_attributes() & 0x400 != 0 => {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Managed File path contains a reparse point"));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    pub(crate) fn init_directories(&self) -> io::Result<()> {
        for root in [self.managed_root(), self.staging_root()] {
            self.ensure_no_reparse_ancestors(&root)?;
            fs::create_dir_all(&root)?;
            self.ensure_no_reparse_ancestors(&root)?;
        }
        Ok(())
    }

    pub(crate) fn validate_safe_relative_path(relative: &str) -> io::Result<PathBuf> {
        let invalid = || io::Error::new(io::ErrorKind::InvalidInput, "invalid Managed File relative path");
        if relative.chars().any(|ch| ch.is_control() || ":*?\"<>|".contains(ch)) {
            return Err(invalid());
        }
        let parts: Vec<_> = relative.split(['/', '\\']).collect();
        let valid_id = |id: &str| id.len() == 32 && id.bytes().all(|byte| byte.is_ascii_hexdigit());
        match parts.as_slice() {
            ["staging", filename] if filename.strip_suffix(".tmp").is_some_and(valid_id) => {}
            ["managed", id, filename] if valid_id(id) => {
                let stem = filename.split('.').next().unwrap_or_default().to_ascii_uppercase();
                let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (stem.len() == 4 && (stem.starts_with("COM") || stem.starts_with("LPT"))
                        && matches!(stem.as_bytes()[3], b'1'..=b'9'));
                if filename.is_empty() || filename.starts_with('.') || filename.ends_with(['.', ' '])
                    || !filename.ends_with(".txt") || reserved {
                    return Err(invalid());
                }
            }
            _ => return Err(invalid()),
        }
        Ok(parts.iter().collect())
    }

    pub(crate) fn staging_rel_path(item_id: &str) -> String { format!("staging/{item_id}.tmp") }
    pub(crate) fn final_rel_path(item_id: &str, filename: &str) -> String { format!("managed/{item_id}/{filename}") }

    pub(crate) fn full_path(&self, relative: &str) -> io::Result<PathBuf> {
        let path = self.root.join(Self::validate_safe_relative_path(relative)?);
        self.ensure_no_reparse_ancestors(&path)?;
        Ok(path)
    }

    pub(crate) fn stage_bytes(&self, relative: &str, bytes: &[u8]) -> io::Result<PathBuf> {
        self.init_directories()?;
        let path = self.full_path(relative)?;
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            drop(file);
            // Failed uncommitted writes remain inside the dedicated staging area for startup cleanup.
            if let Err(cleanup) = fs::remove_file(&path) {
                return Err(io::Error::new(cleanup.kind(), format!("{error}; staging cleanup failed: {cleanup}")));
            }
            return Err(error);
        }
        Ok(path)
    }

    pub(crate) fn promote_staged(&self, staging: &str, final_relative: &str) -> io::Result<PathBuf> {
        let source = self.full_path(staging)?;
        let target = self.full_path(final_relative)?;
        let parent = target.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Managed File parent missing"))?;
        fs::create_dir_all(parent)?;
        self.ensure_no_reparse_ancestors(&target)?;
        fs::rename(source, &target)?;
        Ok(target)
    }

    pub(crate) fn delete_managed_target(&self, relative: &str) -> io::Result<bool> {
        let path = self.full_path(relative)?;
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if relative.starts_with("managed/") || relative.starts_with("managed\\") {
            let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Managed File parent missing"))?;
            self.ensure_no_reparse_ancestors(parent)?;
            match fs::remove_dir(parent) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }

    pub(crate) fn cleanup_unowned_staging(&self, known: &[String]) -> io::Result<()> {
        let root = self.staging_root();
        self.ensure_no_reparse_ancestors(&root)?;
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        let known_paths = known.iter().map(|path| self.full_path(path)).collect::<io::Result<Vec<_>>>()?;
        for entry in entries {
            let path = entry?.path();
            self.ensure_no_reparse_ancestors(&path)?;
            let filename = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid staging filename"))?;
            Self::validate_safe_relative_path(&format!("staging/{filename}"))?;
            if !known_paths.contains(&path) { fs::remove_file(path)?; }
        }
        Ok(())
    }
}

pub(crate) fn format_local_timestamp_now() -> String {
    // SAFETY: GetLocalTime returns an initialized SYSTEMTIME and takes no borrowed pointers.
    let time = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    format!("{:04}-{:02}-{:02} {:02}-{:02}-{:02}", time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond)
}
