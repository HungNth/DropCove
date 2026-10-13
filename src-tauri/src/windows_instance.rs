//! Profile lease acquired before presentation or persistence is initialized.

use std::{
    ffi::OsStr,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, WebviewWindow};
use windows::{
    core::{BOOL, PCWSTR},
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, WPARAM},
        Storage::FileSystem::{
            CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::{DataExchange::COPYDATASTRUCT, Threading::CreateMutexW},
        UI::{
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                AllowSetForegroundWindow, EnumWindows, GetPropW, GetWindowThreadProcessId,
                RemovePropW, SendMessageTimeoutW, SetPropW, SMTO_ABORTIFHUNG, WM_COPYDATA,
                WM_NCDESTROY,
            },
        },
    },
};

const MAGIC: usize = 0x4443_3031;
const SUBCLASS: usize = 0xDC06;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) enum Launch {
    Primary(InstanceLease),
    Secondary,
}

pub(crate) struct InstanceLease {
    _mutex: OwnedHandle,
    property: Vec<u16>,
}

struct Receiver {
    app: AppHandle,
    property: Vec<u16>,
}

impl InstanceLease {
    pub(crate) fn register(&mut self, window: &WebviewWindow) -> Result<(), String> {
        let hwnd = window.hwnd().map_err(|error| error.to_string())?;
        let receiver = Box::into_raw(Box::new(Receiver {
            app: window.app_handle().clone(),
            property: self.property.clone(),
        }));
        // SAFETY: Setup runs on the window's UI thread. The subclass owns receiver until
        // WM_NCDESTROY; unsuccessful installation frees it without exposing it to callbacks.
        if !unsafe { SetWindowSubclass(hwnd, Some(receive), SUBCLASS, receiver as usize) }.as_bool()
        {
            // SAFETY: Installation failed, so no callback owns this allocation.
            unsafe {
                drop(Box::from_raw(receiver));
            }
            return Err("could not install resident activation receiver".into());
        }
        // Publish readiness only after the receiver and all application state exist.
        // SAFETY: The window and NUL-terminated property are live for this synchronous call.
        if let Err(error) = unsafe {
            SetPropW(
                hwnd,
                PCWSTR(self.property.as_ptr()),
                Some(HANDLE(MAGIC as *mut _)),
            )
        } {
            // SAFETY: Setup still owns the UI thread; no callback can concurrently use receiver.
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(receive), SUBCLASS);
                drop(Box::from_raw(receiver));
            }
            return Err(format!(
                "could not publish resident activation receiver: {error}"
            ));
        }
        Ok(())
    }
}

unsafe extern "system" fn receive(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    // SAFETY: register installs a live Receiver allocation as subclass data. Only
    // WM_NCDESTROY frees it; callbacks execute serially on the owning UI thread.
    let receiver = unsafe { &*(data as *const Receiver) };
    if message == WM_COPYDATA && lparam.0 != 0 {
        // SAFETY: Windows marshals COPYDATASTRUCT for the synchronous WM_COPYDATA call.
        let request = unsafe { &*(lparam.0 as *const COPYDATASTRUCT) };
        if request.dwData == MAGIC && request.cbData == 0 {
            // Already on the UI thread: queueing and waiting here would deadlock its message pump.
            return LRESULT(isize::from(
                crate::resident::show_shelf(&receiver.app).is_ok(),
            ));
        }
    }
    if message == WM_NCDESTROY {
        // SAFETY: The subclass owns this allocation; removing it prevents further callbacks.
        unsafe {
            let _ = RemovePropW(hwnd, PCWSTR(receiver.property.as_ptr()));
            let _ = RemoveWindowSubclass(hwnd, Some(receive), SUBCLASS);
            drop(Box::from_raw(data as *mut Receiver));
        }
    }
    // SAFETY: Forward the original window message to the remaining subclass chain.
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

pub(crate) struct LaunchOptions {
    pub(crate) profile: PathBuf,
    pub(crate) autostart: bool,
}

fn parse_arguments(
    mut arguments: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(Option<PathBuf>, bool), String> {
    let mut profile = None;
    let mut autostart = false;
    while let Some(argument) = arguments.next() {
        if argument == OsStr::new("--profile") {
            if profile.is_some() {
                return Err("--profile may only be specified once".into());
            }
            let directory = arguments.next().ok_or("--profile requires a directory")?;
            if directory.is_empty()
                || directory == OsStr::new("--autostart")
                || directory == OsStr::new("--profile")
            {
                return Err("--profile requires a directory".into());
            }
            profile = Some(PathBuf::from(directory));
        } else if argument == OsStr::new("--autostart") {
            if autostart {
                return Err("--autostart may only be specified once".into());
            }
            autostart = true;
        } else {
            return Err(format!(
                "unsupported launch argument: {}",
                argument.to_string_lossy()
            ));
        }
    }
    Ok((profile, autostart))
}

pub(crate) fn resolve_launch() -> Result<LaunchOptions, String> {
    let (profile, autostart) = parse_arguments(std::env::args_os().skip(1))?;
    let profile = match profile {
        Some(profile) => profile,
        None => PathBuf::from(
            std::env::var_os("LOCALAPPDATA").ok_or("Windows LOCALAPPDATA is unavailable")?,
        )
        .join("DropCove"),
    };
    std::fs::create_dir_all(&profile)
        .map_err(|error| format!("could not create profile {}: {error}", profile.display()))?;
    // Preserve canonical UTF-16, including extended/UNC paths and case-sensitive directories.
    let profile = profile
        .canonicalize()
        .map_err(|error| format!("could not resolve profile {}: {error}", profile.display()))?;
    Ok(LaunchOptions { profile, autostart })
}

fn profile_identity(profile: &Path) -> Result<String, String> {
    let path: Vec<u16> = profile.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: The canonical path is NUL-terminated. Open metadata only, sharing ordinary
    // reads/writes/deletion; no database or profile content is opened by a secondary.
    let handle = unsafe {
        CreateFileW(
            PCWSTR(path.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }
    .map_err(|error| format!("could not identify profile: {error}"))?;
    // SAFETY: CreateFileW returned a uniquely owned handle; OwnedHandle closes it once.
    let handle = unsafe { OwnedHandle::from_raw_handle(handle.0) };
    let mut identity = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: The metadata handle is live and identity is a valid writable output.
    unsafe { GetFileInformationByHandle(HANDLE(handle.as_raw_handle()), &mut identity) }
        .map_err(|error| format!("could not read profile identity: {error}"))?;
    Ok(format!(
        "{:08x}-{:08x}{:08x}",
        identity.dwVolumeSerialNumber, identity.nFileIndexHigh, identity.nFileIndexLow
    ))
}

pub(crate) fn acquire(profile: &Path, autostart: bool) -> Result<Launch, String> {
    let identity = profile_identity(profile)?;
    let name: Vec<u16> = format!("Local\\DropCove-Profile-{identity}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: The NUL-terminated name lives for the call. This mutex's existence is the
    // lease; ownership is unnecessary and would tie teardown to the startup thread.
    let raw = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }
        .map_err(|error| format!("could not acquire profile lease: {error}"))?;
    // SAFETY: Read the result immediately before another API can change last-error.
    let exists = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    // SAFETY: CreateMutexW returned a uniquely owned handle.
    let mutex = unsafe { OwnedHandle::from_raw_handle(raw.0) };
    let property: Vec<u16> = format!("DropCove-Receiver-{identity}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    if exists {
        drop(mutex);
        if !autostart {
            activate(&property)?;
        }
        Ok(Launch::Secondary)
    } else {
        Ok(Launch::Primary(InstanceLease {
            _mutex: mutex,
            property,
        }))
    }
}

fn activate(property: &[u16]) -> Result<(), String> {
    let deadline = Instant::now() + HANDSHAKE_TIMEOUT;
    loop {
        if let Some(hwnd) = find_receiver(property) {
            let mut pid = 0;
            // SAFETY: The enumerated HWND is passed by value; pid is writable output.
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid == 0 {
                return Err("resident activation receiver disappeared; launch again".into());
            }
            // SAFETY: Grant the primary permission to activate; Windows validates the PID.
            let _ = unsafe { AllowSetForegroundWindow(pid) };
            let request = COPYDATASTRUCT {
                dwData: MAGIC,
                cbData: 0,
                lpData: std::ptr::null_mut(),
            };
            let mut acknowledgement = 0;
            let remaining = deadline
                .saturating_duration_since(Instant::now())
                .as_millis();
            if remaining == 0 {
                break;
            }
            // SAFETY: Windows copies request for this synchronous message; the timeout
            // bounds an unresponsive receiver and the stack data stays live during the call.
            let sent = unsafe {
                SendMessageTimeoutW(
                    hwnd,
                    WM_COPYDATA,
                    WPARAM(0),
                    LPARAM(&request as *const _ as isize),
                    SMTO_ABORTIFHUNG,
                    remaining.min(u32::MAX as u128) as u32,
                    Some(&mut acknowledgement),
                )
            };
            if sent.0 != 0 && acknowledgement == 1 {
                return Ok(());
            }
            return Err("resident could not acknowledge Shelf activation; close it explicitly and launch again".into());
        }
        if Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err("resident is not ready after 10 seconds; no second profile owner was started".into())
}

fn find_receiver(property: &[u16]) -> Option<HWND> {
    struct Search {
        property: PCWSTR,
        found: Option<HWND>,
    }
    unsafe extern "system" fn visit(hwnd: HWND, parameter: LPARAM) -> BOOL {
        // SAFETY: EnumWindows receives a live mutable Search for its synchronous traversal.
        let search = unsafe { &mut *(parameter.0 as *mut Search) };
        // SAFETY: HWND came from EnumWindows; property points to a live NUL-terminated name.
        if unsafe { GetPropW(hwnd, search.property) }.0 as usize == MAGIC {
            search.found = Some(hwnd);
            return false.into();
        }
        true.into()
    }
    let mut search = Search {
        property: PCWSTR(property.as_ptr()),
        found: None,
    };
    // SAFETY: The callback and Search remain live until synchronous enumeration ends.
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(&mut search as *mut _ as isize)) };
    search.found
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn autostart_is_independent_of_profile_argument_order_and_rejects_duplicates() {
        let parse = |args: &[&str]| parse_arguments(args.iter().map(std::ffi::OsString::from));
        let expected = (Some(PathBuf::from("C:\\Owned Profile")), true);
        assert_eq!(
            parse(&["--autostart", "--profile", "C:\\Owned Profile"]).unwrap(),
            expected
        );
        assert_eq!(
            parse(&["--profile", "C:\\Owned Profile", "--autostart"]).unwrap(),
            expected
        );
        assert!(parse(&["--profile", "--autostart"]).is_err());
        assert!(parse(&["--autostart", "--autostart"]).is_err());
        assert!(parse(&["--profile", "one", "--profile", "two"]).is_err());
        assert!(parse(&["--unsupported"]).is_err());
    }

    #[test]
    fn aliases_share_identity_but_distinct_profiles_do_not() {
        let root = std::env::temp_dir().join(format!("dropcove-instance-{}", std::process::id()));
        std::fs::create_dir_all(root.join("Profile")).unwrap();
        std::fs::create_dir_all(root.join("Other")).unwrap();
        let profile = root.join("Profile").canonicalize().unwrap();
        assert_eq!(
            profile_identity(&profile).unwrap(),
            profile_identity(&root.join("PROFILE")).unwrap()
        );
        assert_ne!(
            profile_identity(&profile).unwrap(),
            profile_identity(&root.join("Other")).unwrap()
        );
        let Launch::Primary(lease) = acquire(&profile, false).unwrap() else {
            panic!("expected owned primary");
        };
        let Launch::Primary(other) = acquire(&root.join("Other"), false).unwrap() else {
            panic!("distinct profile blocked");
        };
        drop(lease);
        let Launch::Primary(reopened) = acquire(&profile, false).unwrap() else {
            panic!("lease was not released");
        };
        drop((other, reopened));
        std::fs::remove_dir_all(root).unwrap();
    }
}
