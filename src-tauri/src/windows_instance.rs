//! Profile lease acquired before presentation or persistence is initialized.

use std::{
    ffi::OsStr,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, WebviewWindow};
use windows::{
    core::{BOOL, PCWSTR},
    Win32::{
        Foundation::{
            GetLastError, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, HANDLE, HWND, LPARAM,
            LRESULT, WAIT_OBJECT_0, WPARAM,
        },
        Storage::FileSystem::{
            CreateFileW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::{
            DataExchange::COPYDATASTRUCT,
            Threading::{
                CreateMutexW, OpenMutexW, OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
                SYNCHRONIZATION_SYNCHRONIZE,
            },
        },
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
const SHUTDOWN_MAGIC: usize = 0x4443_3032;
const SUBCLASS: usize = 0xDC06;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(25);

pub(crate) enum Launch {
    Primary(InstanceLease),
    Secondary,
}

pub(crate) struct InstanceLease {
    mutex: Arc<Mutex<Option<OwnedHandle>>>,
    property: Vec<u16>,
}

struct Receiver {
    app: AppHandle,
    mutex: Arc<Mutex<Option<OwnedHandle>>>,
    property: Vec<u16>,
}

impl InstanceLease {
    pub(crate) fn register(&mut self, window: &WebviewWindow) -> Result<(), String> {
        let hwnd = window.hwnd().map_err(|error| error.to_string())?;
        let receiver = Box::into_raw(Box::new(Receiver {
            app: window.app_handle().clone(),
            mutex: Arc::clone(&self.mutex),
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
        if request.dwData == SHUTDOWN_MAGIC && request.cbData == 0 {
            let accepted = crate::resident::exit(&receiver.app).is_ok();
            if accepted {
                unsafe {
                    let _ = RemovePropW(hwnd, PCWSTR(receiver.property.as_ptr()));
                }
                if let Ok(mut guard) = receiver.mutex.lock() {
                    guard.take();
                }
            }
            return LRESULT(isize::from(accepted));
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LaunchOptions {
    pub(crate) profile: PathBuf,
    pub(crate) autostart: bool,
    pub(crate) shutdown: bool,
}

fn parse_arguments(
    mut arguments: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(Option<PathBuf>, bool, bool), String> {
    let mut profile = None;
    let mut autostart = false;
    let mut shutdown = false;
    while let Some(argument) = arguments.next() {
        if argument == OsStr::new("--profile") {
            if profile.is_some() {
                return Err("--profile may only be specified once".into());
            }
            let directory = arguments.next().ok_or("--profile requires a directory")?;
            if directory.is_empty()
                || directory == OsStr::new("--autostart")
                || directory == OsStr::new("--profile")
                || directory == OsStr::new("--shutdown")
            {
                return Err("--profile requires a directory".into());
            }
            profile = Some(PathBuf::from(directory));
        } else if argument == OsStr::new("--autostart") {
            if autostart {
                return Err("--autostart may only be specified once".into());
            }
            if shutdown {
                return Err("--shutdown cannot be combined with --autostart".into());
            }
            autostart = true;
        } else if argument == OsStr::new("--shutdown") {
            if shutdown {
                return Err("--shutdown may only be specified once".into());
            }
            if autostart {
                return Err("--shutdown cannot be combined with --autostart".into());
            }
            shutdown = true;
        } else {
            return Err(format!(
                "unsupported launch argument: {}",
                argument.to_string_lossy()
            ));
        }
    }
    Ok((profile, autostart, shutdown))
}

pub(crate) fn resolve_launch() -> Result<LaunchOptions, String> {
    let (profile, autostart, shutdown) = parse_arguments(std::env::args_os().skip(1))?;
    let profile = match profile {
        Some(profile) => profile,
        None => PathBuf::from(
            std::env::var_os("LOCALAPPDATA").ok_or("Windows LOCALAPPDATA is unavailable")?,
        )
        .join("DropCove"),
    };
    if shutdown {
        let profile = if profile.exists() {
            profile
                .canonicalize()
                .map_err(|error| format!("could not resolve profile {}: {error}", profile.display()))?
        } else {
            profile
        };
        return Ok(LaunchOptions {
            profile,
            autostart,
            shutdown: true,
        });
    }
    std::fs::create_dir_all(&profile)
        .map_err(|error| format!("could not create profile {}: {error}", profile.display()))?;
    // Preserve canonical UTF-16, including extended/UNC paths and case-sensitive directories.
    let profile = profile
        .canonicalize()
        .map_err(|error| format!("could not resolve profile {}: {error}", profile.display()))?;
    Ok(LaunchOptions {
        profile,
        autostart,
        shutdown: false,
    })
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
            mutex: Arc::new(Mutex::new(Some(mutex))),
            property,
        }))
    }
}

fn lease_exists(mutex_name: &[u16]) -> Result<bool, String> {
    // SAFETY: mutex_name is a live NUL-terminated UTF-16 string. OpenMutexW never creates
    // a missing mutex; closing the returned handle immediately prevents keeping the lease alive.
    match unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, PCWSTR(mutex_name.as_ptr())) } {
        Ok(raw) => {
            // SAFETY: OpenMutexW returned a uniquely owned handle.
            let _owned = unsafe { OwnedHandle::from_raw_handle(raw.0) };
            Ok(true)
        }
        Err(error) if error.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(false),
        Err(error) => Err(format!("could not query profile lease: {error}")),
    }
}

pub(crate) fn shutdown(profile: &Path) -> Result<(), String> {
    shutdown_with_timeout(profile, SHUTDOWN_TIMEOUT)
}

fn shutdown_with_timeout(profile: &Path, timeout: Duration) -> Result<(), String> {
    if !profile.exists() {
        return Ok(());
    }
    let profile = profile
        .canonicalize()
        .map_err(|error| format!("could not resolve profile {}: {error}", profile.display()))?;
    let identity = profile_identity(&profile)?;
    let mutex_name: Vec<u16> = format!("Local\\DropCove-Profile-{identity}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let property: Vec<u16> = format!("DropCove-Receiver-{identity}")
        .encode_utf16()
        .chain(Some(0))
        .collect();

    if !lease_exists(&mutex_name)? && find_receiver(&property).is_none() {
        return Ok(());
    }

    let deadline = Instant::now() + timeout;
    let mut resident_process: Option<OwnedHandle> = None;
    let mut shutdown_acknowledged = false;

    while Instant::now() < deadline {
        if let Some(hwnd) = find_receiver(&property) {
            let mut pid = 0;
            // SAFETY: The enumerated HWND is passed by value; pid is writable output.
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid == 0 {
                if find_receiver(&property).is_none() && !lease_exists(&mutex_name)? {
                    break;
                }
                return Err("resident receiver disappeared without releasing profile lease".into());
            }
            if resident_process.is_none() {
                // SAFETY: pid identifies the live process owning the receiver window;
                // PROCESS_SYNCHRONIZE requests only wait permission without keeping its mutex alive.
                match unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) } {
                    Ok(raw) => {
                        // SAFETY: OpenProcess returned a uniquely owned process handle.
                        resident_process = Some(unsafe { OwnedHandle::from_raw_handle(raw.0) });
                    }
                    Err(error) => {
                        if find_receiver(&property).is_none() && !lease_exists(&mutex_name)? {
                            break;
                        }
                        return Err(format!(
                            "could not open resident process {pid} to confirm exit: {error}"
                        ));
                    }
                }
            }
            let remaining = deadline
                .saturating_duration_since(Instant::now())
                .as_millis();
            if remaining == 0 {
                break;
            }
            let request = COPYDATASTRUCT {
                dwData: SHUTDOWN_MAGIC,
                cbData: 0,
                lpData: std::ptr::null_mut(),
            };
            let mut acknowledgement = 0;
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
                shutdown_acknowledged = true;
                break;
            }
        } else if !lease_exists(&mutex_name)? {
            break;
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        std::thread::sleep(remaining.min(SHUTDOWN_POLL_INTERVAL));
    }

    if let Some(process) = resident_process {
        let remaining = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u32::MAX as u128) as u32;
        // SAFETY: process is a valid SYNCHRONIZE handle owned for this wait.
        let waited = unsafe { WaitForSingleObject(HANDLE(process.as_raw_handle()), remaining) };
        if waited != WAIT_OBJECT_0 {
            return Err("resident process did not exit within 5 seconds".into());
        }
    }

    while lease_exists(&mutex_name)? {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("resident profile lease was not released within 5 seconds".into());
        }
        std::thread::sleep(remaining.min(SHUTDOWN_POLL_INTERVAL));
    }

    if !shutdown_acknowledged && find_receiver(&property).is_some() {
        return Err("resident could not complete cooperative shutdown within 5 seconds".into());
    }

    Ok(())
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
        let expected = (Some(PathBuf::from("C:\\Owned Profile")), true, false);
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
    fn shutdown_argument_parses_with_optional_profile_and_rejects_conflicts() {
        let parse = |args: &[&str]| parse_arguments(args.iter().map(std::ffi::OsString::from));
        assert_eq!(parse(&["--shutdown"]).unwrap(), (None, false, true));
        assert_eq!(
            parse(&["--shutdown", "--profile", "C:\\Owned Profile"]).unwrap(),
            (Some(PathBuf::from("C:\\Owned Profile")), false, true)
        );
        assert_eq!(
            parse(&["--profile", "C:\\Owned Profile", "--shutdown"]).unwrap(),
            (Some(PathBuf::from("C:\\Owned Profile")), false, true)
        );
        assert!(parse(&["--shutdown", "--autostart"]).is_err());
        assert!(parse(&["--autostart", "--shutdown"]).is_err());
        assert!(parse(&["--shutdown", "--shutdown"]).is_err());
        assert!(parse(&["--profile", "--shutdown"]).is_err());
    }

    #[test]
    fn shutdown_succeeds_without_creating_missing_or_idle_profile_and_leaves_no_lease() {
        let root = std::env::temp_dir().join(format!(
            "dropcove-shutdown-idle-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let missing = root.join("MissingProfile");
        shutdown(&missing).unwrap();
        assert!(!missing.exists());

        let idle = root.join("IdleProfile");
        std::fs::create_dir_all(&idle).unwrap();
        shutdown(&idle).unwrap();
        let Launch::Primary(lease) = acquire(&idle.canonicalize().unwrap(), false).unwrap() else {
            panic!("idle shutdown left a profile lease behind");
        };
        drop(lease);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cooperative_shutdown_retries_until_safe_point_and_waits_for_process_exit() {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use windows::{
            core::w,
            Win32::UI::WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, PeekMessageW, TranslateMessage,
                MSG, PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE,
            },
        };

        struct ChildReceiver {
            attempts: Arc<AtomicUsize>,
            required_attempts: usize,
            stop: Arc<AtomicBool>,
            mutex: Arc<Mutex<Option<OwnedHandle>>>,
            property: Vec<u16>,
        }

        unsafe extern "system" fn child_receive(
            hwnd: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
            _id: usize,
            data: usize,
        ) -> LRESULT {
            // SAFETY: Child test installs a live ChildReceiver allocation on its UI thread.
            let receiver = unsafe { &*(data as *const ChildReceiver) };
            if message == WM_COPYDATA && lparam.0 != 0 {
                // SAFETY: Windows marshals COPYDATASTRUCT for synchronous WM_COPYDATA.
                let request = unsafe { &*(lparam.0 as *const COPYDATASTRUCT) };
                if request.dwData == SHUTDOWN_MAGIC && request.cbData == 0 {
                    let count = receiver.attempts.fetch_add(1, Ordering::AcqRel) + 1;
                    let accepted = count >= receiver.required_attempts;
                    if accepted {
                        // SAFETY: Valid HWND and NUL-terminated property on owning thread.
                        unsafe {
                            let _ = RemovePropW(hwnd, PCWSTR(receiver.property.as_ptr()));
                        }
                        if let Ok(mut guard) = receiver.mutex.lock() {
                            guard.take();
                        }
                        receiver.stop.store(true, Ordering::Release);
                    }
                    return LRESULT(isize::from(accepted));
                }
            }
            if message == WM_NCDESTROY {
                // SAFETY: Subclass owns this allocation and removes itself on destroy.
                unsafe {
                    let _ = RemovePropW(hwnd, PCWSTR(receiver.property.as_ptr()));
                    let _ = RemoveWindowSubclass(hwnd, Some(child_receive), SUBCLASS);
                    drop(Box::from_raw(data as *mut ChildReceiver));
                }
            }
            // SAFETY: Forward to remaining subclass chain.
            unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
        }

        if let Ok(child_profile) = std::env::var("DROPCOVE_TEST_RESIDENT_PROFILE") {
            let required_attempts: usize = std::env::var("DROPCOVE_TEST_REQUIRED_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let canonical = PathBuf::from(child_profile).canonicalize().unwrap();
            let Launch::Primary(lease) = acquire(&canonical, false).unwrap() else {
                panic!("child expected primary lease");
            };
            let hwnd = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("STATIC"),
                    w!(""),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    None,
                    None,
                )
            }
            .unwrap();
            let stop = Arc::new(AtomicBool::new(false));
            let receiver = Box::into_raw(Box::new(ChildReceiver {
                attempts: Arc::new(AtomicUsize::new(0)),
                required_attempts,
                stop: Arc::clone(&stop),
                mutex: Arc::clone(&lease.mutex),
                property: lease.property.clone(),
            }));
            assert!(unsafe {
                SetWindowSubclass(hwnd, Some(child_receive), SUBCLASS, receiver as usize)
            }
            .as_bool());
            unsafe {
                SetPropW(
                    hwnd,
                    PCWSTR(lease.property.as_ptr()),
                    Some(HANDLE(MAGIC as *mut _)),
                )
                .unwrap();
            }
            let mut msg = MSG::default();
            while !stop.load(Ordering::Acquire) {
                while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
                    unsafe {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            drop(lease);
            // Delay process exit after lease release to prove caller waits on OS process handle too.
            std::thread::sleep(Duration::from_millis(120));
            return;
        }

        let root = std::env::temp_dir().join(format!(
            "dropcove-shutdown-proc-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let profile = root.join("Profile");
        std::fs::create_dir_all(&profile).unwrap();
        let canonical = profile.canonicalize().unwrap();
        let identity = profile_identity(&canonical).unwrap();
        let mutex_name: Vec<u16> = format!("Local\\DropCove-Profile-{identity}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let property: Vec<u16> = format!("DropCove-Receiver-{identity}")
            .encode_utf16()
            .chain(Some(0))
            .collect();

        // 1. Busy for 2 attempts then accepts on 3rd; caller retries and waits for process exit + lease release.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("windows_instance::tests::cooperative_shutdown_retries_until_safe_point_and_waits_for_process_exit")
            .env("DROPCOVE_TEST_RESIDENT_PROFILE", &canonical)
            .env("DROPCOVE_TEST_REQUIRED_ATTEMPTS", "3")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();

        let start = Instant::now();
        while find_receiver(&property).is_none() {
            assert!(start.elapsed() < Duration::from_secs(5), "child receiver did not start");
            std::thread::sleep(Duration::from_millis(10));
        }

        shutdown(&canonical).unwrap();
        let status = child
            .try_wait()
            .unwrap()
            .expect("shutdown returned before resident process exited");
        assert!(status.success());
        assert!(!lease_exists(&mutex_name).unwrap());

        // 2. Busy past deadline: caller times out with Err while child process and lease remain active.
        let mut busy_child = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("windows_instance::tests::cooperative_shutdown_retries_until_safe_point_and_waits_for_process_exit")
            .env("DROPCOVE_TEST_RESIDENT_PROFILE", &canonical)
            .env("DROPCOVE_TEST_REQUIRED_ATTEMPTS", "999999")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();

        let start = Instant::now();
        while find_receiver(&property).is_none() {
            assert!(start.elapsed() < Duration::from_secs(5), "busy child receiver did not start");
            std::thread::sleep(Duration::from_millis(10));
        }

        assert!(shutdown_with_timeout(&canonical, Duration::from_millis(250)).is_err());
        assert!(busy_child.try_wait().unwrap().is_none(), "timeout must not kill busy resident");
        assert!(lease_exists(&mutex_name).unwrap());

        let _ = busy_child.kill();
        let _ = busy_child.wait();
        std::fs::remove_dir_all(root).unwrap();
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
