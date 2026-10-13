//! Windows Startup native adapter for HKCU Run DropCove registration.

use std::{iter::once, path::Path};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR},
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
            RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_BINARY,
            REG_DWORD, REG_EXPAND_SZ, REG_MULTI_SZ, REG_NONE, REG_OPTION_NON_VOLATILE, REG_QWORD,
            REG_SZ, REG_VALUE_TYPE,
        },
    },
};

/// Microsoft Run/RunOnce keys support command lines no longer than 260 characters.
pub(crate) const MAX_RUN_COMMAND_CHARS: usize = 260;

/// Standard HKCU Run subkey path.
const RUN_SUBKEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Standard DropCove Run value name.
const VALUE_NAME: &str = "DropCove";

/// Raw registry value preserving exact kind and byte representation for rollbacks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegistryValue {
    pub(crate) kind: u32,
    pub(crate) bytes: Vec<u8>,
}

/// Owned RAII wrapper for an HKEY ensuring deterministic closure.
struct OwnedKey(HKEY);

impl OwnedKey {
    fn open(
        root: HKEY,
        subkey: &[u16],
        flags: windows::Win32::System::Registry::REG_SAM_FLAGS,
    ) -> Result<Self, WIN32_ERROR> {
        let mut handle = HKEY::default();
        // SAFETY: `root` is a valid root/open handle; `subkey` is a live NUL-terminated wide string;
        // `handle` is an out-pointer for the opened HKEY.
        let status =
            unsafe { RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), None, flags, &mut handle) };
        if status == ERROR_SUCCESS {
            Ok(Self(handle))
        } else {
            Err(status)
        }
    }

    fn create(
        root: HKEY,
        subkey: &[u16],
        flags: windows::Win32::System::Registry::REG_SAM_FLAGS,
    ) -> Result<Self, WIN32_ERROR> {
        let mut handle = HKEY::default();
        // SAFETY: `root` is a valid root/open handle; `subkey` is a live NUL-terminated wide string;
        // `handle` is an out-pointer receiving the created or opened HKEY.
        let status = unsafe {
            RegCreateKeyExW(
                root,
                PCWSTR(subkey.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                flags,
                None,
                &mut handle,
                None,
            )
        };
        if status == ERROR_SUCCESS {
            Ok(Self(handle))
        } else {
            Err(status)
        }
    }

    fn raw(&self) -> HKEY {
        self.0
    }
}

impl Drop for OwnedKey {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: `self.0` is an owned open registry handle closed exactly once here.
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }
}

/// Helper converting a Rust string slice to a NUL-terminated wide vector.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(once(0)).collect()
}

/// Quotes a single argument according to Windows CRT CommandLineToArgvW parsing rules.
///
/// Handles embedded backslashes and trailing backslashes before the closing quote:
/// - 2N backslashes followed by a quotation mark produce N backslashes followed by begin/end quote.
/// - 2N+1 backslashes followed by a quotation mark produce N backslashes followed by literal quote.
/// - N backslashes not followed by a quotation mark produce N backslashes.
pub(crate) fn quote_windows_arg(arg: &str) -> String {
    let mut quoted = String::with_capacity(arg.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        if c == '\\' {
            backslashes += 1;
        } else if c == '"' {
            quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
            quoted.push('"');
            backslashes = 0;
        } else {
            quoted.extend(std::iter::repeat_n('\\', backslashes));
            quoted.push(c);
            backslashes = 0;
        }
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

/// Formats the complete autostart command line:
/// `"{exe}" --autostart --profile "{profile}"`
pub(crate) fn format_command(exe: &Path, profile: &Path) -> Result<String, String> {
    let exe_str = exe
        .to_str()
        .ok_or_else(|| format!("executable path {} contains invalid UTF-8", exe.display()))?;
    let profile_str = profile
        .to_str()
        .ok_or_else(|| format!("profile path {} contains invalid UTF-8", profile.display()))?;
    let quoted_exe = quote_windows_arg(exe_str);
    let quoted_profile = quote_windows_arg(profile_str);
    let cmd = format!("{quoted_exe} --autostart --profile {quoted_profile}");
    let chars = cmd.encode_utf16().count();
    if chars > MAX_RUN_COMMAND_CHARS {
        return Err(format!(
            "command length ({chars} chars) exceeds Windows Run limit ({} chars)",
            MAX_RUN_COMMAND_CHARS
        ));
    }
    Ok(cmd)
}

/// Native Windows startup registration manager for HKCU Run DropCove.
#[derive(Clone, Debug)]
pub(crate) struct StartupRegistration {
    subkey: Vec<u16>,
    value_name: Vec<u16>,
    command: String,
    command_bytes: Vec<u8>,
}

impl StartupRegistration {
    /// Constructs registration target for HKCU Run DropCove with the active profile.
    pub(crate) fn new(profile: &Path) -> Result<Self, String> {
        let exe = std::env::current_exe()
            .map_err(|error| format!("failed to resolve current executable: {error}"))?;
        // The launch coordinator supplies the already-resolved profile. Never silently
        // substitute another path if resolution failed.
        let command = format_command(&exe, profile)?;
        Ok(Self::with_target(RUN_SUBKEY, VALUE_NAME, command))
    }

    /// Internal constructor allowing test isolation under dedicated subkeys.
    fn with_target(subkey: &str, value_name: &str, command: String) -> Self {
        let command_bytes = command
            .encode_utf16()
            .chain(once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        Self {
            subkey: to_wide(subkey),
            value_name: to_wide(value_name),
            command,
            command_bytes,
        }
    }

    /// Read-only snapshot of the exact current registry value under this target.
    pub(crate) fn snapshot(&self) -> Result<Option<RegistryValue>, String> {
        let key = match OwnedKey::open(HKEY_CURRENT_USER, &self.subkey, KEY_QUERY_VALUE) {
            Ok(k) => k,
            Err(e) if e == ERROR_FILE_NOT_FOUND => return Ok(None),
            Err(e) => {
                return Err(format!(
                    "failed to open registry key for query: WIN32 error 0x{:08X}",
                    e.0
                ))
            }
        };

        let mut kind = REG_VALUE_TYPE::default();
        let mut byte_count: u32 = 0;
        // Bounded first query to check existence and size.
        // SAFETY: `key` is valid; `value_name` is live NUL-terminated wide string;
        // passing None for data queries size only.
        let status = unsafe {
            RegQueryValueExW(
                key.raw(),
                PCWSTR(self.value_name.as_ptr()),
                None,
                Some(&mut kind),
                None,
                Some(&mut byte_count),
            )
        };

        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
            return Err(format!(
                "failed to query registry value size: WIN32 error 0x{:08X}",
                status.0
            ));
        }

        // Bound registry value size to reasonable allocation limit (64 KB).
        const MAX_VALUE_BYTES: u32 = 65536;
        if byte_count > MAX_VALUE_BYTES {
            return Err(format!(
                "registry value size ({byte_count} bytes) exceeds safety limit ({MAX_VALUE_BYTES} bytes)"
            ));
        }

        let mut buffer = vec![0u8; byte_count as usize];
        // SAFETY: `buffer` has capacity and length of `byte_count`; out pointers are valid.
        let status = unsafe {
            RegQueryValueExW(
                key.raw(),
                PCWSTR(self.value_name.as_ptr()),
                None,
                Some(&mut kind),
                Some(buffer.as_mut_ptr()),
                Some(&mut byte_count),
            )
        };

        if status == ERROR_SUCCESS {
            buffer.truncate(byte_count as usize);
            Ok(Some(RegistryValue {
                kind: kind.0,
                bytes: buffer,
            }))
        } else if status == ERROR_FILE_NOT_FOUND {
            Ok(None)
        } else {
            Err(format!(
                "failed to read registry value data: WIN32 error 0x{:08X}",
                status.0
            ))
        }
    }

    /// Applies the desired startup state: enable writes the owned command; disable removes it.
    pub(crate) fn apply(&self, enabled: bool) -> Result<(), String> {
        if enabled {
            let key =
                OwnedKey::create(HKEY_CURRENT_USER, &self.subkey, KEY_SET_VALUE).map_err(|e| {
                    format!(
                        "failed to open/create registry key for writing: WIN32 error 0x{:08X}",
                        e.0
                    )
                })?;
            // SAFETY: `key` is valid with KEY_SET_VALUE access; `value_name` is NUL-terminated;
            // `command_bytes` is a valid slice containing UTF-16 bytes including trailing NUL.
            let status = unsafe {
                RegSetValueExW(
                    key.raw(),
                    PCWSTR(self.value_name.as_ptr()),
                    None,
                    REG_SZ,
                    Some(&self.command_bytes),
                )
            };
            if status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(format!(
                    "failed to set registry value: WIN32 error 0x{:08X}",
                    status.0
                ))
            }
        } else {
            let key = match OwnedKey::open(HKEY_CURRENT_USER, &self.subkey, KEY_SET_VALUE) {
                Ok(k) => k,
                Err(e) if e == ERROR_FILE_NOT_FOUND => return Ok(()),
                Err(e) => {
                    return Err(format!(
                        "failed to open registry key for deletion: WIN32 error 0x{:08X}",
                        e.0
                    ))
                }
            };
            // SAFETY: `key` is valid with KEY_SET_VALUE; deleting missing value returns ERROR_FILE_NOT_FOUND.
            let status = unsafe { RegDeleteValueW(key.raw(), PCWSTR(self.value_name.as_ptr())) };
            if status == ERROR_SUCCESS || status == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                Err(format!(
                    "failed to delete registry value: WIN32 error 0x{:08X}",
                    status.0
                ))
            }
        }
    }

    /// Restores the exact prior registry value or removes it if it was absent.
    pub(crate) fn restore(&self, old: Option<&RegistryValue>) -> Result<(), String> {
        match old {
            Some(value) => {
                let key = OwnedKey::create(HKEY_CURRENT_USER, &self.subkey, KEY_SET_VALUE)
                    .map_err(|e| {
                        format!(
                            "failed to open/create registry key for restore: WIN32 error 0x{:08X}",
                            e.0
                        )
                    })?;
                // SAFETY: `key` is valid with KEY_SET_VALUE access; restoring exact kind and bytes.
                let status = unsafe {
                    RegSetValueExW(
                        key.raw(),
                        PCWSTR(self.value_name.as_ptr()),
                        None,
                        REG_VALUE_TYPE(value.kind),
                        Some(&value.bytes),
                    )
                };
                if status == ERROR_SUCCESS {
                    Ok(())
                } else {
                    Err(format!(
                        "failed to restore registry value: WIN32 error 0x{:08X}",
                        status.0
                    ))
                }
            }
            None => {
                // If old value was absent, remove only this value (missing is success).
                self.apply(false)
            }
        }
    }

    /// Checks whether the exact owned command is currently registered.
    pub(crate) fn enabled(&self) -> Result<bool, String> {
        let snap = self.snapshot()?;
        match snap {
            Some(value) => {
                if value.kind == REG_SZ.0 {
                    let observed = decode_reg_sz(&value.bytes);
                    Ok(observed == self.command)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    /// Describes the actual retained registration (for error/incomplete rollback reporting).
    pub(crate) fn describe(&self) -> Result<String, String> {
        let snap = self.snapshot()?;
        match snap {
            Some(value) => {
                if value.kind == REG_SZ.0 {
                    let text = decode_reg_sz(&value.bytes);
                    Ok(format!("REG_SZ: {text}"))
                } else {
                    Ok(format!(
                        "type {} ({} bytes)",
                        describe_reg_kind(value.kind),
                        value.bytes.len()
                    ))
                }
            }
            None => Ok("absent".to_string()),
        }
    }
}

/// Decodes REG_SZ bytes (little-endian UTF-16 with optional trailing NUL) into a String.
fn decode_reg_sz(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    // Trim trailing NUL characters if present.
    let trimmed = match units.iter().position(|&c| c == 0) {
        Some(pos) => &units[..pos],
        None => &units[..],
    };
    String::from_utf16_lossy(trimmed)
}

/// Formats standard Win32 registry type IDs into readable names.
fn describe_reg_kind(kind: u32) -> &'static str {
    if kind == REG_SZ.0 {
        "REG_SZ"
    } else if kind == REG_EXPAND_SZ.0 {
        "REG_EXPAND_SZ"
    } else if kind == REG_BINARY.0 {
        "REG_BINARY"
    } else if kind == REG_DWORD.0 {
        "REG_DWORD"
    } else if kind == REG_MULTI_SZ.0 {
        "REG_MULTI_SZ"
    } else if kind == REG_QWORD.0 {
        "REG_QWORD"
    } else if kind == REG_NONE.0 {
        "REG_NONE"
    } else {
        "non-standard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::UI::Shell::CommandLineToArgvW;

    /// Helper converting a command line to argument list via Win32 CommandLineToArgvW.
    fn parse_command_line(cmd: &str) -> Vec<String> {
        let wide = to_wide(cmd);
        let mut argc = 0i32;
        // SAFETY: `wide` is a valid NUL-terminated UTF-16 string; CommandLineToArgvW allocates an array
        // of PWSTR pointers that must be freed with LocalFree.
        let argv = unsafe { CommandLineToArgvW(PCWSTR(wide.as_ptr()), &mut argc) };
        assert!(!argv.is_null(), "CommandLineToArgvW returned null");
        let mut args = Vec::with_capacity(argc as usize);
        for i in 0..argc {
            // SAFETY: `argv` is valid for `argc` elements; each element points to a NUL-terminated wide string.
            unsafe {
                let pwstr = *argv.offset(i as isize);
                args.push(pwstr.to_string().expect("valid UTF-16 argument"));
            }
        }
        // SAFETY: CommandLineToArgvW buffer must be freed using LocalFree.
        unsafe {
            let _ = windows::Win32::Foundation::LocalFree(Some(HLOCAL(argv as *mut _)));
        }
        args
    }

    #[test]
    fn test_quoting_standard_paths() {
        let exe = Path::new(r"C:\Program Files\DropCove\dropcove.exe");
        let profile = Path::new(r"C:\Users\User Name\AppData\Local\DropCove");
        let cmd = format_command(exe, profile).unwrap();

        let parsed = parse_command_line(&cmd);
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], r"C:\Program Files\DropCove\dropcove.exe");
        assert_eq!(parsed[1], "--autostart");
        assert_eq!(parsed[2], "--profile");
        assert_eq!(parsed[3], r"C:\Users\User Name\AppData\Local\DropCove");
    }

    #[test]
    fn test_quoting_trailing_backslash() {
        let exe = Path::new(r"D:\Tools\dropcove.exe");
        let profile = Path::new(r"D:\Custom Profiles\DropCove\");
        let cmd = format_command(exe, profile).unwrap();

        let parsed = parse_command_line(&cmd);
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], r"D:\Tools\dropcove.exe");
        assert_eq!(parsed[1], "--autostart");
        assert_eq!(parsed[2], "--profile");
        assert_eq!(parsed[3], r"D:\Custom Profiles\DropCove\");
    }

    #[test]
    fn test_quoting_unicode_and_quotes() {
        let exe = Path::new(r"C:\用户\Drop Cove\dropcove.exe");
        let profile = Path::new(r#"C:\Users\Special\"User"\Profile\"#);
        let cmd = format_command(exe, profile).unwrap();

        let parsed = parse_command_line(&cmd);
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0], r"C:\用户\Drop Cove\dropcove.exe");
        assert_eq!(parsed[1], "--autostart");
        assert_eq!(parsed[2], "--profile");
        assert_eq!(parsed[3], r#"C:\Users\Special\"User"\Profile\"#);
    }

    #[test]
    fn test_command_length_limit() {
        let exe = Path::new(r"C:\dropcove.exe");
        let base = format_command(exe, Path::new(r"C:\x"))
            .unwrap()
            .encode_utf16()
            .count();
        let at_limit = PathBuf::from(format!(r"C:\x{}", "a".repeat(260 - base)));
        assert_eq!(
            format_command(exe, &at_limit)
                .unwrap()
                .encode_utf16()
                .count(),
            260
        );
        let beyond = PathBuf::from(format!("{}a", at_limit.display()));
        assert!(format_command(exe, &beyond).is_err());
    }

    fn cleanup_test_key(subkey: &str) {
        let wide = to_wide(subkey);
        // SAFETY: Deleting the test subkey created specifically by tests.
        unsafe {
            let _ = windows::Win32::System::Registry::RegDeleteKeyW(
                HKEY_CURRENT_USER,
                PCWSTR(wide.as_ptr()),
            );
        }
    }

    #[test]
    fn test_owned_registry_lifecycle() {
        let subkey = format!(
            "Software\\DropCove_UnitTests\\StartupTest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let registration = StartupRegistration::with_target(
            &subkey,
            "DropCoveTest",
            r#""C:\Test\dropcove.exe" --autostart --profile "C:\Test\profile""#.to_string(),
        );

        // 1. Initial snapshot must be None
        assert_eq!(registration.snapshot().unwrap(), None);
        assert!(!registration.enabled().unwrap());
        assert_eq!(registration.describe().unwrap(), "absent");

        // 2. Disable on non-existing value succeeds cleanly
        assert!(registration.apply(false).is_ok());
        let unrelated =
            StartupRegistration::with_target(&subkey, "UnrelatedOwned", "unrelated fixture".into());
        unrelated.apply(true).unwrap();
        let unrelated_before = unrelated.snapshot().unwrap();

        // 3. Enable writes the exact REG_SZ command
        registration.apply(true).unwrap();
        assert!(registration.enabled().unwrap());

        let snap = registration.snapshot().unwrap().expect("snapshot exists");
        assert_eq!(snap.kind, REG_SZ.0);

        // 4. Writing foreign value to verify enabled() returns false
        let foreign_reg = StartupRegistration::with_target(
            &subkey,
            "DropCoveTest",
            r#""C:\Other\dropcove.exe" --autostart"#.to_string(),
        );
        foreign_reg.apply(true).unwrap();
        assert!(!registration.enabled().unwrap());

        // 5. Restore previous snapshot restores original command
        registration.restore(Some(&snap)).unwrap();
        assert!(registration.enabled().unwrap());

        // 6. Restore non-REG_SZ value (e.g. REG_DWORD) to verify exact kind preservation
        let dword_val = RegistryValue {
            kind: REG_DWORD.0,
            bytes: vec![42, 0, 0, 0],
        };
        registration.restore(Some(&dword_val)).unwrap();
        assert!(!registration.enabled().unwrap());
        let snap_dword = registration.snapshot().unwrap().unwrap();
        assert_eq!(snap_dword.kind, REG_DWORD.0);
        assert_eq!(snap_dword.bytes, vec![42, 0, 0, 0]);

        // 7. Restore None removes the value
        registration.restore(None).unwrap();
        assert_eq!(registration.snapshot().unwrap(), None);
        assert!(!registration.enabled().unwrap());
        assert_eq!(registration.describe().unwrap(), "absent");
        assert_eq!(unrelated.snapshot().unwrap(), unrelated_before);
        unrelated.restore(None).unwrap();

        cleanup_test_key(&subkey);
    }
}
