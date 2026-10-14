//! DropCove Settings persistence model and Windows atomic store.
//!
//! Handles canonical JSON serialization/deserialization for `HotKey`,
//! validation matching the native recorder allowlist, current layout-aware label formatting,
//! and atomic Windows file transactions (create_new temp + write_all + sync_all + ReplaceFileW/MoveFileExW).

use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write},
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND},
        Storage::FileSystem::{
            MoveFileExW, ReplaceFileW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            REPLACEFILE_WRITE_THROUGH,
        },
        UI::Input::KeyboardAndMouse::{GetKeyNameTextW, MapVirtualKeyW, MAPVK_VK_TO_VSC},
    },
};

const MAX_SETTINGS_SIZE_BYTES: u64 = 64 * 1024; // 64 KiB trust-boundary limit
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Canonical HotKey model matching ticket 03 specification.
/// Serialized with PascalCase field keys: Control, Alt, Shift, Windows, VirtualKey.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HotKey {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub windows: bool,
    pub virtual_key: u16,
}

impl Default for HotKey {
    fn default() -> Self {
        // Ctrl + Shift + Space
        Self {
            control: true,
            alt: false,
            shift: true,
            windows: false,
            virtual_key: 0x20, // VK_SPACE
        }
    }
}

impl HotKey {
    /// Returns true if at least one modifier key is active.
    pub fn has_modifier(&self) -> bool {
        self.control || self.alt || self.shift || self.windows
    }

    /// Validates that the combination contains at least one modifier and an allowed primary key.
    ///
    /// Allowed primary keys:
    /// - Letters: 'A'..='Z' (0x41..=0x5A)
    /// - Digits: '0'..='9' (0x30..=0x39)
    /// - Function keys: F1..=F24 (0x70..=0x87) EXCEPT F12 (0x7B) reserved by Windows
    /// - Navigation/editing: Backspace, Tab, Enter, Space, PageUp, PageDown, End, Home, Left, Up, Right, Down,
    ///   Select, Print, Execute, Snapshot (PrintScreen), Insert, Delete
    /// - Numpad: 0-9, Multiply, Add, Separator, Subtract, Decimal, Divide (0x60..=0x6F)
    /// - OEM punctuation: 0xBA..=0xC0, 0xDB..=0xDF, 0xE2
    ///
    /// Rejects:
    /// - Combinations without modifiers
    /// - Escape (0x1B)
    /// - Modifier-only keys (0x10..=0x12, 0x5B..=0x5C, 0xA0..=0xA5)
    /// - Mouse buttons (0x01..=0x06)
    /// - Windows reserved F12 (0x7B)
    /// - System / Media / Browser / Volume keys (0xA6..=0xB7, 0xE5..=0xF5)
    /// - Out of range (0 or > 0xFE)
    pub fn validate(&self) -> Result<(), String> {
        if !self.has_modifier() {
            return Err(
                "HotKey requires at least one modifier (Ctrl, Alt, Shift, or Win)".to_string(),
            );
        }

        let vk = self.virtual_key;

        if vk == 0 || vk > 0xFE {
            return Err(format!("Virtual key code 0x{vk:02X} is out of legal range"));
        }

        // Explicitly rejected special keys
        if vk == 0x1B {
            return Err("Escape cannot be used as a global hotkey".to_string());
        }
        if vk == 0x7B {
            return Err(
                "F12 is reserved by Windows for system debugging and cannot be registered"
                    .to_string(),
            );
        }

        // Modifier-only keys
        if matches!(vk, 0x10..=0x12 | 0x5B..=0x5C | 0xA0..=0xA5) {
            return Err("Modifier keys alone cannot be used as primary hotkeys".to_string());
        }

        // Mouse buttons
        if matches!(vk, 0x01..=0x06) {
            return Err("Mouse buttons cannot be used as global hotkeys".to_string());
        }

        // Media, Browser, Volume, System launch keys
        if matches!(vk, 0xA6..=0xB7 | 0xE5..=0xF5) {
            return Err("Media, browser, and system keys are not supported as hotkeys".to_string());
        }

        // Allowed checks
        let is_letter = (0x41..=0x5A).contains(&vk);
        let is_digit = (0x30..=0x39).contains(&vk);
        let is_fkey = ((0x70..=0x87).contains(&vk)) && vk != 0x7B;
        let is_numpad = (0x60..=0x6F).contains(&vk);

        let is_nav_edit = matches!(
            vk,
            0x08 // Backspace
            | 0x09 // Tab (allowed since has_modifier() is verified)
            | 0x0D // Enter (allowed with modifier)
            | 0x20 // Space (allowed with modifier)
            | 0x21..=0x2E // PageUp (0x21), PageDown (0x22), End (0x23), Home (0x24), Left (0x25), Up (0x26), Right (0x27), Down (0x28), Select (0x29), Print (0x2A), Execute (0x2B), Snapshot/PrintScreen (0x2C), Insert (0x2D), Delete (0x2E)
        );

        let is_oem = matches!(
            vk,
            0xBA..=0xC0 // VK_OEM_1 (;:) through VK_OEM_3 (~`) and comma/minus/period/slash
            | 0xDB..=0xDF // VK_OEM_4 ([{) through VK_OEM_8
            | 0xE2 // VK_OEM_102 (<> or \| on 102-key keyboards)
        );

        if is_letter || is_digit || is_fkey || is_numpad || is_nav_edit || is_oem {
            Ok(())
        } else {
            Err(format!("Unsupported virtual key code: 0x{vk:02X}"))
        }
    }

    /// Formats the HotKey into human-readable text in strict `Ctrl + Alt + Shift + Win + Key` order.
    /// Uses Windows current layout `GetKeyNameTextW` when available, with standard fallbacks.
    pub fn label(&self) -> String {
        let mut parts = Vec::new();

        if self.control {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.windows {
            parts.push("Win");
        }

        let key_name = format_virtual_key(self.virtual_key);
        parts.push(&key_name);

        parts.join(" + ")
    }
}

/// Helper to format a virtual key code into its display name.
fn format_virtual_key(vk: u16) -> String {
    #[cfg(windows)]
    {
        if let Some(name) = get_windows_key_name(vk) {
            return name;
        }
    }

    fallback_key_name(vk)
}

#[cfg(windows)]
fn get_windows_key_name(vk: u16) -> Option<String> {
    // Some keys have standardized display names regardless of driver quirks
    match vk {
        0x20 => return Some("Space".to_string()),
        0x08 => return Some("Backspace".to_string()),
        0x09 => return Some("Tab".to_string()),
        0x0D => return Some("Enter".to_string()),
        0x1B => return Some("Esc".to_string()),
        0x2E => return Some("Delete".to_string()),
        0x2D => return Some("Insert".to_string()),
        0x24 => return Some("Home".to_string()),
        0x23 => return Some("End".to_string()),
        0x21 => return Some("Page Up".to_string()),
        0x22 => return Some("Page Down".to_string()),
        0x25 => return Some("Left".to_string()),
        0x26 => return Some("Up".to_string()),
        0x27 => return Some("Right".to_string()),
        0x28 => return Some("Down".to_string()),
        0x2C => return Some("Print Screen".to_string()),
        _ => {}
    }

    // Map virtual key to scan code
    // SAFETY: MapVirtualKeyW is a standard Win32 call taking unsigned int
    let scan_code = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) };
    if scan_code == 0 {
        return None;
    }

    // Construct LPARAM for GetKeyNameTextW:
    // Bits 16..23: scan code
    // Bit 24: extended key flag (for nav keys, numpad divide, etc.)
    let mut lparam = (scan_code as i32) << 16;
    if matches!(
        vk,
        0x21
            ..=0x2E // PageUp, PageDown, End, Home, Left, Up, Right, Down, PrintScreen, Insert, Delete
        | 0x6F // Numpad divide
    ) {
        lparam |= 1 << 24;
    }

    let mut buffer = [0u16; 64];
    // SAFETY: buffer is valid for up to 64 UTF-16 code units.
    let len = unsafe { GetKeyNameTextW(lparam, &mut buffer) };
    if len > 0 {
        let name = String::from_utf16_lossy(&buffer[..len as usize]);
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            // Capitalize first letter if single character or clean title case
            return Some(trimmed.to_string());
        }
    }

    None
}

fn fallback_key_name(vk: u16) -> String {
    match vk {
        0x08 => "Backspace".to_string(),
        0x09 => "Tab".to_string(),
        0x0D => "Enter".to_string(),
        0x1B => "Esc".to_string(),
        0x20 => "Space".to_string(),
        0x21 => "Page Up".to_string(),
        0x22 => "Page Down".to_string(),
        0x23 => "End".to_string(),
        0x24 => "Home".to_string(),
        0x25 => "Left".to_string(),
        0x26 => "Up".to_string(),
        0x27 => "Right".to_string(),
        0x28 => "Down".to_string(),
        0x2C => "Print Screen".to_string(),
        0x2D => "Insert".to_string(),
        0x2E => "Delete".to_string(),
        0x30..=0x39 => format!("{}", (vk as u8 as char)),
        0x41..=0x5A => format!("{}", (vk as u8 as char)),
        0x60..=0x69 => format!("Num {}", vk - 0x60),
        0x6A => "Num *".to_string(),
        0x6B => "Num +".to_string(),
        0x6C => "Num Separator".to_string(),
        0x6D => "Num -".to_string(),
        0x6E => "Num .".to_string(),
        0x6F => "Num /".to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x70 + 1),
        0xBA => ";".to_string(),
        0xBB => "=".to_string(),
        0xBC => ",".to_string(),
        0xBD => "-".to_string(),
        0xBE => ".".to_string(),
        0xBF => "/".to_string(),
        0xC0 => "`".to_string(),
        0xDB => "[".to_string(),
        0xDC => "\\".to_string(),
        0xDD => "]".to_string(),
        0xDE => "'".to_string(),
        0xE2 => "\\".to_string(),
        _ => format!("0x{vk:02X}"),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RailEdge {
    Left,
    #[default]
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ShakeSensitivity {
    Low,
    #[default]
    Normal,
    High,
}

/// Active settings contract; missing or unknown fields are malformed, never migrated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Settings {
    pub(crate) hot_key: HotKey,
    pub(crate) start_with_windows: bool,
    pub(crate) rail_edge: RailEdge,
    pub(crate) rail_monitor_id: String,
    pub(crate) show_rail_over_fullscreen: bool,
    pub(crate) shake_enabled: bool,
    pub(crate) shake_sensitivity: ShakeSensitivity,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hot_key: HotKey::default(),
            start_with_windows: true,
            rail_edge: RailEdge::Right,
            rail_monitor_id: String::new(),
            show_rail_over_fullscreen: false,
            shake_enabled: true,
            shake_sensitivity: ShakeSensitivity::Normal,
        }
    }
}

impl Settings {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.hot_key.validate()?;
        if self.rail_monitor_id.contains('\0') || self.rail_monitor_id.encode_utf16().count() > 1024
        {
            return Err("invalid target monitor identifier".into());
        }
        Ok(())
    }
}

/// Settings store persisting configuration to `<profile>/settings.json`.
#[derive(Clone, Debug)]
pub struct SettingsStore {
    settings_path: PathBuf,
}

impl SettingsStore {
    /// Creates a store instance targeting `<profile>/settings.json`.
    pub fn new(profile: &Path) -> Self {
        Self {
            settings_path: profile.join("settings.json"),
        }
    }

    /// Path to the settings file.
    #[cfg(test)]
    pub fn path(&self) -> &Path {
        &self.settings_path
    }

    /// Loads settings from disk.
    ///
    /// - If the file does not exist (NotFound), returns silent defaults `(Settings::default(), None)` without writing to disk.
    /// - If the file is malformed, unreadable (PermissionDenied, I/O error), exceeds 64 KiB, or fails validation:
    ///   returns defaults with a warning message `(Settings::default(), Some(warning_message))` without modifying disk.
    /// - If valid: returns `(settings, None)`.
    pub fn load(&self) -> (Settings, Option<String>) {
        let mut file = match File::open(&self.settings_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    return (Settings::default(), None);
                }
                return (
                    Settings::default(),
                    Some(format!("Could not open settings file: {err}")),
                );
            }
        };

        let metadata = match file.metadata() {
            Ok(meta) => meta,
            Err(err) => {
                return (
                    Settings::default(),
                    Some(format!("Could not read settings metadata: {err}")),
                );
            }
        };

        if metadata.len() > MAX_SETTINGS_SIZE_BYTES {
            return (
                Settings::default(),
                Some(format!(
                    "Settings file exceeds maximum allowed size ({} bytes > {} bytes)",
                    metadata.len(),
                    MAX_SETTINGS_SIZE_BYTES
                )),
            );
        }

        let mut bytes = Vec::new();
        let bounded_reader = (&mut file).take(MAX_SETTINGS_SIZE_BYTES + 1);
        let mut reader = bounded_reader;
        if let Err(err) = reader.read_to_end(&mut bytes) {
            return (
                Settings::default(),
                Some(format!("Could not read settings file: {err}")),
            );
        }

        if bytes.len() as u64 > MAX_SETTINGS_SIZE_BYTES {
            return (
                Settings::default(),
                Some(format!(
                    "Settings file exceeds maximum allowed size ({} bytes > {} bytes)",
                    bytes.len(),
                    MAX_SETTINGS_SIZE_BYTES
                )),
            );
        }

        let settings: Settings = match serde_json::from_slice(&bytes) {
            Ok(parsed) => parsed,
            Err(err) => {
                return (
                    Settings::default(),
                    Some(format!("Settings file is malformed: {err}")),
                );
            }
        };

        if let Err(val_err) = settings.validate() {
            return (
                Settings::default(),
                Some(format!("Settings file contains invalid hotkey: {val_err}")),
            );
        }

        (settings, None)
    }

    /// Reads raw settings file content if present, or `None` if missing.
    /// Subject to trust-boundary size limit (64 KiB).
    pub fn read_raw(&self) -> Result<Option<Vec<u8>>, String> {
        let mut file = match File::open(&self.settings_path) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(format!("Failed to open settings file: {err}"));
            }
        };

        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to inspect settings file: {e}"))?;

        if metadata.len() > MAX_SETTINGS_SIZE_BYTES {
            return Err(format!(
                "Settings file exceeds maximum allowed size ({} bytes)",
                metadata.len()
            ));
        }

        let mut content = Vec::new();
        let bounded_reader = (&mut file).take(MAX_SETTINGS_SIZE_BYTES + 1);
        let mut reader = bounded_reader;
        reader
            .read_to_end(&mut content)
            .map_err(|e| format!("Failed to read raw settings file: {e}"))?;

        if content.len() as u64 > MAX_SETTINGS_SIZE_BYTES {
            return Err(format!(
                "Settings file exceeds maximum allowed size ({} bytes)",
                content.len()
            ));
        }

        Ok(Some(content))
    }

    /// Atomically writes the given Settings to `settings.json`.
    ///
    /// Sequence:
    /// 1. Validate `settings`.
    /// 2. Serialize canonical JSON.
    /// 3. Create a unique temporary file in the same directory using `create_new(true)`.
    /// 4. Write all bytes and flush/sync all to disk.
    /// 5. Atomically replace existing file using `ReplaceFileW` (or `MoveFileExW` if destination does not exist).
    /// 6. Clean up temporary file on failure.
    pub fn write(&self, settings: &Settings) -> Result<(), String> {
        settings.validate()?;

        let json_bytes = serde_json::to_vec_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;

        self.atomic_write_bytes(&json_bytes)
    }

    /// Restores raw previous content for rollback.
    ///
    /// - If `content` is `Some(bytes)`: atomically writes `bytes` to `settings.json`.
    /// - If `content` is `None`: removes `settings.json` if it exists.
    pub fn restore_raw(&self, content: Option<&[u8]>) -> Result<(), String> {
        match content {
            Some(bytes) => {
                if bytes.len() as u64 > MAX_SETTINGS_SIZE_BYTES {
                    return Err("Raw restore bytes exceed size limit".to_string());
                }
                self.atomic_write_bytes(bytes)
            }
            None => {
                if self.settings_path.exists() {
                    fs::remove_file(&self.settings_path).map_err(|e| {
                        format!("Failed to remove settings file during rollback: {e}")
                    })?;
                }
                Ok(())
            }
        }
    }

    fn atomic_write_bytes(&self, bytes: &[u8]) -> Result<(), String> {
        let parent_dir = self.settings_path.parent().ok_or_else(|| {
            format!(
                "Settings path has no parent directory: {:?}",
                self.settings_path
            )
        })?;

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("Failed to create settings directory: {e}"))?;
        }

        // Create unique temporary file in the same directory
        let (temp_path, mut temp_file) = create_unique_temp_file(parent_dir)?;

        // Ensure temp file is cleaned up if we return early with an error
        struct TempCleanup<'a>(&'a Path);
        impl<'a> Drop for TempCleanup<'a> {
            fn drop(&mut self) {
                let _ = fs::remove_file(self.0);
            }
        }
        let cleanup = TempCleanup(&temp_path);

        temp_file
            .write_all(bytes)
            .map_err(|e| format!("Failed to write settings temp file: {e}"))?;
        temp_file
            .sync_all()
            .map_err(|e| format!("Failed to sync settings temp file: {e}"))?;
        drop(temp_file);

        // Perform atomic replace on Windows
        atomic_replace_file(&temp_path, &self.settings_path)?;

        // Disarm cleanup since replace succeeded and moved the file
        std::mem::forget(cleanup);

        Ok(())
    }
}

/// Creates a unique temporary file in `dir` using `create_new(true)`.
fn create_unique_temp_file(dir: &Path) -> Result<(PathBuf, File), String> {
    let pid = std::process::id();
    for _ in 0..100 {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let filename = format!(".settings-{pid}-{counter}.tmp");
        let candidate = dir.join(filename);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(format!("Failed to create temporary settings file: {err}")),
        }
    }
    Err("Failed to create a unique temporary settings file after multiple attempts".to_string())
}

fn to_u16_null(path: &Path) -> Vec<u16> {
    let os_str: &OsStr = path.as_os_str();
    os_str.encode_wide().chain(std::iter::once(0)).collect()
}

/// Atomically replaces `destination` with `source` using Windows Win32 APIs.
#[cfg(windows)]
fn atomic_replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    let source_u16 = to_u16_null(source);
    let dest_u16 = to_u16_null(destination);

    let source_pcwstr = PCWSTR(source_u16.as_ptr());
    let dest_pcwstr = PCWSTR(dest_u16.as_ptr());

    // If destination does not exist yet, MoveFileExW is atomic within the same volume.
    if !destination.exists() {
        // SAFETY: source_pcwstr and dest_pcwstr are valid null-terminated UTF-16 wide strings.
        let ok = unsafe {
            MoveFileExW(
                source_pcwstr,
                dest_pcwstr,
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if ok.is_ok() {
            return Ok(());
        } else {
            let error = unsafe { GetLastError() };
            // If it failed because destination was created in the meantime, fall through to ReplaceFileW
            if error != ERROR_FILE_NOT_FOUND && error != ERROR_PATH_NOT_FOUND {
                // If it wasn't a race with non-existence, check if dest now exists
                if !destination.exists() {
                    return Err(format!("MoveFileExW failed: error code {:?}", error));
                }
            }
        }
    }

    // Destination exists: use ReplaceFileW for atomic replacement preserving attributes and backup safety.
    // SAFETY: ReplaceFileW requires lpReplacedFileName (dest), lpReplacementFileName (source),
    // and optionally lpBackupFileName (None here).
    let ok = unsafe {
        ReplaceFileW(
            dest_pcwstr,
            source_pcwstr,
            PCWSTR::null(),
            REPLACEFILE_WRITE_THROUGH,
            None,
            None,
        )
    };

    if ok.is_ok() {
        Ok(())
    } else {
        let error = unsafe { GetLastError() };
        // Fallback to MoveFileExW if ReplaceFileW fails because destination was removed or not accessible
        if error == ERROR_FILE_NOT_FOUND {
            let move_ok = unsafe {
                MoveFileExW(
                    source_pcwstr,
                    dest_pcwstr,
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if move_ok.is_ok() {
                return Ok(());
            }
        }
        Err(format!("ReplaceFileW failed: Win32 error code {:?}", error))
    }
}

#[cfg(not(windows))]
fn atomic_replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    fs::rename(source, destination).map_err(|e| format!("Failed to replace settings file: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("dropcove_test_{}_{}", name, std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_hotkey_defaults_and_validation() {
        let default_hk = HotKey::default();
        assert!(default_hk.control);
        assert!(!default_hk.alt);
        assert!(default_hk.shift);
        assert!(!default_hk.windows);
        assert_eq!(default_hk.virtual_key, 0x20);
        assert!(default_hk.validate().is_ok());
        assert_eq!(default_hk.label(), "Ctrl + Shift + Space");

        // Reject without modifiers
        let no_mod = HotKey {
            control: false,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x41, // 'A'
        };
        assert!(no_mod.validate().is_err());

        // Reject Escape
        let esc = HotKey {
            control: true,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x1B,
        };
        assert!(esc.validate().is_err());

        // Reject F12 (reserved Windows debug key)
        let f12 = HotKey {
            control: true,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x7B,
        };
        assert!(f12.validate().is_err());

        // Reject modifier-only keys
        let mod_ctrl = HotKey {
            control: true,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x11, // VK_CONTROL
        };
        assert!(mod_ctrl.validate().is_err());

        // Reject mouse button
        let mouse_btn = HotKey {
            control: true,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x01, // VK_LBUTTON
        };
        assert!(mouse_btn.validate().is_err());

        // Allowed: Letters, digits, F1-F24 (except F12), navigation, punctuation, modified Tab/Enter/Space
        let tab = HotKey {
            control: true,
            alt: false,
            shift: false,
            windows: false,
            virtual_key: 0x09, // Tab
        };
        assert!(tab.validate().is_ok());

        let enter = HotKey {
            control: false,
            alt: true,
            shift: false,
            windows: false,
            virtual_key: 0x0D, // Enter
        };
        assert!(enter.validate().is_ok());

        let letter_a = HotKey {
            control: true,
            alt: true,
            shift: true,
            windows: true,
            virtual_key: 0x41, // 'A'
        };
        assert!(letter_a.validate().is_ok());
        assert_eq!(letter_a.label(), "Ctrl + Alt + Shift + Win + A");
    }

    #[test]
    fn rail_schema_requires_all_fields_and_preserves_old_configuration_until_save() {
        let temp = TempDir::new("rail-cutover");
        let store = SettingsStore::new(&temp.path);
        let previous = br#"{"HotKey":{"Control":true,"Alt":true,"Shift":false,"Windows":false,"VirtualKey":75},"StartWithWindows":false,"RailEdge":"Left","RailMonitorId":"display","ShowRailOverFullscreen":true}"#;
        fs::write(store.path(), previous).unwrap();
        let (loaded, warning) = store.load();
        assert_eq!(loaded, Settings::default());
        assert!(warning.is_some());
        assert_eq!(fs::read(store.path()).unwrap(), previous);
        let settings = Settings {
            rail_edge: RailEdge::Left,
            rail_monitor_id: "disconnected-saved-display".into(),
            start_with_windows: false,
            ..loaded
        };
        store.write(&settings).unwrap();
        assert_eq!(store.load(), (settings.clone(), None));
        let value = serde_json::to_value(&settings).unwrap();
        for field in [
            "HotKey",
            "StartWithWindows",
            "RailEdge",
            "RailMonitorId",
            "ShowRailOverFullscreen",
            "ShakeEnabled",
            "ShakeSensitivity",
        ] {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<Settings>(missing).is_err());
        }
        let mut invalid = value.clone();
        invalid["RailEdge"] = "Top".into();
        assert!(serde_json::from_value::<Settings>(invalid).is_err());
        let mut unknown = value;
        unknown["Unexpected"] = true.into();
        assert!(serde_json::from_value::<Settings>(unknown).is_err());
    }

    #[test]
    fn test_load_defaults_on_missing_file_without_mutation() {
        let temp = TempDir::new("missing");
        let store = SettingsStore::new(&temp.path);

        assert!(!store.path().exists());
        let (settings, warning) = store.load();
        assert_eq!(settings, Settings::default());
        assert_eq!(settings.start_with_windows, true);
        assert_eq!(settings.hot_key, HotKey::default());
        assert!(warning.is_none());
        // Must not create file eagerly
        assert!(!store.path().exists());
    }

    #[test]
    fn test_load_malformed_or_invalid_yields_warning_and_preserves_file() {
        let temp = TempDir::new("malformed");
        let store = SettingsStore::new(&temp.path);

        // Write corrupt JSON
        let corrupt_content = b"{\"HotKey\": {invalid_json";
        fs::write(store.path(), corrupt_content).unwrap();

        let (settings, warning) = store.load();
        assert_eq!(settings, Settings::default());
        assert!(warning.is_some());

        // Verify the original file content was preserved untouched
        let retained = fs::read(store.path()).unwrap();
        assert_eq!(retained, corrupt_content);

        // Old ticket 03 schema without StartWithWindows is malformed, file preserved, warning returned
        let old_format = r#"{"HotKey":{"Control":true,"Alt":false,"Shift":true,"Windows":false,"VirtualKey":75}}"#;
        fs::write(store.path(), old_format).unwrap();

        let (settings_old, warning_old) = store.load();
        assert_eq!(settings_old, Settings::default());
        assert!(warning_old.is_some());
        assert_eq!(fs::read_to_string(store.path()).unwrap(), old_format);

        // Write valid JSON with invalid HotKey (no modifiers)
        let invalid_hk_json = r#"{"HotKey":{"Control":false,"Alt":false,"Shift":false,"Windows":false,"VirtualKey":65},"StartWithWindows":true,"RailEdge":"Right","RailMonitorId":"","ShowRailOverFullscreen":false,"ShakeEnabled":true,"ShakeSensitivity":"Normal"}"#;
        fs::write(store.path(), invalid_hk_json).unwrap();

        let (settings2, warning2) = store.load();
        assert_eq!(settings2, Settings::default());
        assert!(warning2.is_some());
        assert_eq!(fs::read_to_string(store.path()).unwrap(), invalid_hk_json);
    }

    #[test]
    fn test_atomic_write_and_read_raw_roundtrip() {
        let temp = TempDir::new("atomic_write");
        let store = SettingsStore::new(&temp.path);

        let settings = Settings {
            hot_key: HotKey {
                control: true,
                alt: true,
                shift: false,
                windows: false,
                virtual_key: 0x42, // 'B'
            },
            start_with_windows: false,
            ..Settings::default()
        };

        store.write(&settings).expect("write succeeded");
        assert!(store.path().exists());

        let raw = store.read_raw().expect("read_raw").expect("file exists");
        let parsed: Settings = serde_json::from_slice(&raw).expect("parse json");
        assert_eq!(parsed, settings);

        let (loaded, warning) = store.load();
        assert!(warning.is_none());
        assert_eq!(loaded, settings);
    }

    #[test]
    fn test_restore_raw_rollback() {
        let temp = TempDir::new("rollback");
        let store = SettingsStore::new(&temp.path);

        // Case 1: Initial state was missing file
        assert_eq!(store.read_raw().unwrap(), None);

        let initial_settings = Settings::default();
        store.write(&initial_settings).unwrap();
        assert!(store.path().exists());

        // Rollback to None
        store.restore_raw(None).unwrap();
        assert!(!store.path().exists());
        assert_eq!(store.read_raw().unwrap(), None);

        // Case 2: Initial state was malformed raw file
        let corrupt_raw = b"{ this is corrupt raw data }";
        fs::write(store.path(), corrupt_raw).unwrap();
        let raw_backup = store.read_raw().unwrap();
        assert_eq!(raw_backup.as_deref(), Some(&corrupt_raw[..]));

        // Write new settings
        let new_settings = Settings {
            hot_key: HotKey {
                control: true,
                alt: false,
                shift: true,
                windows: false,
                virtual_key: 0x4B,
            },
            start_with_windows: false,
            ..Settings::default()
        };
        store.write(&new_settings).unwrap();
        assert_ne!(fs::read(store.path()).unwrap(), corrupt_raw);

        // Rollback to exact raw bytes
        store.restore_raw(raw_backup.as_deref()).unwrap();
        assert_eq!(fs::read(store.path()).unwrap(), corrupt_raw);
    }

    #[test]
    fn test_atomic_replace_failure_when_locked() {
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let temp = TempDir::new("locked");
            let store = SettingsStore::new(&temp.path);

            let initial_settings = Settings::default();
            store.write(&initial_settings).unwrap();

            // Lock destination file exclusively with no share permissions (share_mode = 0)
            // On Windows, FILE_SHARE_READ = 1, FILE_SHARE_WRITE = 2, FILE_SHARE_DELETE = 4.
            // Using share_mode = 0 blocks ReplaceFileW and MoveFileExW.
            let locked_file = OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(store.path())
                .expect("exclusive lock file");

            let new_settings = Settings {
                hot_key: HotKey {
                    control: true,
                    alt: true,
                    shift: false,
                    windows: false,
                    virtual_key: 0x43, // 'C'
                },
                start_with_windows: false,
                ..Settings::default()
            };

            // Attempting to write while locked must return Err
            let write_res = store.write(&new_settings);
            assert!(
                write_res.is_err(),
                "Write must fail when destination is locked"
            );

            drop(locked_file);

            // Verify original file remained intact
            let (loaded, warning) = store.load();
            assert!(warning.is_none());
            assert_eq!(loaded, initial_settings);
        }
    }
}
