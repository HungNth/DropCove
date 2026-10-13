//! Resident-owned Settings draft and one native/disk application transaction.

use crate::{
    resident,
    settings_store::{HotKey, Settings, SettingsStore},
    settings_transaction::{self, Integrations},
    windows_hotkey,
    windows_startup::{RegistryValue, StartupRegistration},
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, MutexGuard,
    },
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub(crate) const SETTINGS_WINDOW_LABEL: &str = "settings";
const SETTINGS_EVENT: &str = "settings-changed";
const SETTINGS_WIDTH: f64 = 420.0;
const SETTINGS_HEIGHT: f64 = 380.0;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsSnapshot {
    pub(crate) hot_key: HotKey,
    pub(crate) label: String,
    pub(crate) listening: bool,
    pub(crate) start_with_windows: bool,
}
struct SettingsSession {
    profile: PathBuf,
    store: SettingsStore,
    desired: Settings,
    active_key: HotKey,
    active_token: Option<i32>,
    draft: Option<Settings>,
    listening: bool,
    startup_warning: Option<String>,
}
#[derive(Default)]
pub(crate) struct SettingsState {
    session: Mutex<Option<SettingsSession>>,
    busy: AtomicBool,
}
impl SettingsState {
    fn lock(&self) -> Result<MutexGuard<'_, Option<SettingsSession>>, String> {
        self.session
            .lock()
            .map_err(|_| "settings coordinator lock is poisoned".into())
    }
}

pub(crate) fn initialize(
    app: &AppHandle,
    profile: &Path,
    window: &WebviewWindow,
) -> Result<(), String> {
    let store = SettingsStore::new(profile);
    let (desired, load_warning) = store.load();
    let mut warnings: Vec<String> = load_warning.into_iter().collect();
    let active_token = match windows_hotkey::initialize(window)
        .and_then(|()| windows_hotkey::register(app, &desired.hot_key))
    {
        Ok(token) => Some(token),
        Err(error) => {
            warnings.push(format!(
                "Global hotkey unavailable: {error}; tray/launch access remains"
            ));
            None
        }
    };
    let startup_result = (|| {
        let startup = StartupRegistration::new(profile)?;
        // Do not rewrite an already-correct Run command during its own sign-in execution.
        if desired.start_with_windows && startup.enabled()? {
            return Ok(());
        }
        startup.apply(desired.start_with_windows)
    })();
    if let Err(error) = startup_result {
        warnings.push(format!("Start with Windows could not be applied: {error}. Desired setting remains {}; Save can retry", desired.start_with_windows));
    }
    app.manage(SettingsState {
        session: Mutex::new(Some(SettingsSession {
            profile: profile.to_path_buf(),
            store,
            desired,
            active_key: desired.hot_key,
            active_token,
            draft: None,
            listening: false,
            startup_warning: (!warnings.is_empty()).then(|| warnings.join("\n")),
        })),
        busy: AtomicBool::new(false),
    });
    Ok(())
}

fn warn(app: &AppHandle, message: &str) {
    if let Err(error) = resident::publish_warning(app, message) {
        eprintln!("{message}; warning delivery failed: {error}");
    }
}
pub(crate) fn publish_startup_warning_if_any(app: &AppHandle) {
    let warning = {
        let state = app.state::<SettingsState>();
        let warning = match state.lock() {
            Ok(mut session) => session.as_mut().and_then(|session| session.startup_warning.take()),
            Err(error) => Some(error),
        };
        warning
    };
    if let Some(warning) = warning {
        warn(app, &warning);
    }
}
pub(crate) fn get_snapshot(app: &AppHandle) -> Result<SettingsSnapshot, String> {
    let state = app.state::<SettingsState>();
    let session = state.lock()?;
    let session = session
        .as_ref()
        .ok_or("settings coordinator is not initialized")?;
    let settings = session.draft.unwrap_or(session.desired);
    Ok(SettingsSnapshot {
        hot_key: settings.hot_key,
        label: settings.hot_key.label(),
        listening: session.listening,
        start_with_windows: settings.start_with_windows,
    })
}
fn emit_snapshot(app: &AppHandle) -> Result<SettingsSnapshot, String> {
    let snapshot = get_snapshot(app)?;
    app.emit(SETTINGS_EVENT, &snapshot)
        .map_err(|error| error.to_string())?;
    Ok(snapshot)
}
pub(crate) fn open(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        emit_snapshot(app)?;
        return Ok(());
    }
    let owner = app
        .get_webview_window("main")
        .ok_or("Drop Shelf window is missing")?;
    WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Settings — DropCove")
    .inner_size(SETTINGS_WIDTH, SETTINGS_HEIGHT)
    .min_inner_size(SETTINGS_WIDTH, SETTINGS_HEIGHT)
    .resizable(false)
    .decorations(true)
    .skip_taskbar(false)
    .owner(&owner)
    .map_err(|error| error.to_string())?
    .build()
    .map_err(|error| error.to_string())?;
    emit_snapshot(app)?;
    Ok(())
}
pub(crate) fn start_recording(app: &AppHandle) -> Result<SettingsSnapshot, String> {
    if app.get_webview_window(SETTINGS_WINDOW_LABEL).is_none() {
        return Err("Settings window is not open".into());
    }
    {
        let state = app.state::<SettingsState>();
        let mut session = state.lock()?;
        session
            .as_mut()
            .ok_or("settings coordinator is not initialized")?
            .listening = true;
    }
    emit_snapshot(app)
}
pub(crate) fn cancel_recording(app: &AppHandle) -> Result<SettingsSnapshot, String> {
    {
        let state = app.state::<SettingsState>();
        let mut session = state.lock()?;
        if let Some(session) = session.as_mut() {
            session.listening = false;
        }
    }
    emit_snapshot(app)
}
fn record_draft(session: &mut SettingsSession, key: HotKey) -> Result<(), String> {
    key.validate()?;
    if !session.listening {
        return Err("shortcut recorder is not listening".into());
    }
    session.draft.get_or_insert(session.desired).hot_key = key;
    session.listening = false;
    Ok(())
}
pub(crate) fn record_key(app: &AppHandle, key: HotKey) -> Result<(), String> {
    {
        let state = app.state::<SettingsState>();
        let mut session = state.lock()?;
        record_draft(
            session
                .as_mut()
                .ok_or("settings coordinator is not initialized")?,
            key,
        )?;
    }
    emit_snapshot(app).map(|_| ())
}
pub(crate) fn set_start_with_windows(
    app: &AppHandle,
    enabled: bool,
) -> Result<SettingsSnapshot, String> {
    if app.get_webview_window(SETTINGS_WINDOW_LABEL).is_none() {
        return Err("Settings window is not open".into());
    }
    {
        let state = app.state::<SettingsState>();
        let mut session = state.lock()?;
        let session = session
            .as_mut()
            .ok_or("settings coordinator is not initialized")?;
        session
            .draft
            .get_or_insert(session.desired)
            .start_with_windows = enabled;
    }
    emit_snapshot(app)
}
pub(crate) fn handle_hotkey(app: &AppHandle, id: i32) -> Result<(), String> {
    let recording_key = {
        let state = app.state::<SettingsState>();
        let session = state.lock()?;
        let Some(session) = session.as_ref() else {
            return Ok(());
        };
        if session.active_token != Some(id) {
            return Ok(());
        }
        session.listening.then_some(session.active_key)
    };
    if let Some(key) = recording_key {
        return record_key(app, key);
    }
    match app.state::<resident::ResidentState>().visibility()? {
        resident::Visibility::Visible => resident::hide_shelf(app),
        resident::Visibility::Hidden => resident::show_shelf(app),
    }
}

struct NativeIntegrations<'a> {
    app: &'a AppHandle,
    startup: StartupRegistration,
}
impl Integrations for NativeIntegrations<'_> {
    fn register(&mut self, key: &HotKey) -> Result<i32, String> {
        windows_hotkey::register(self.app, key)
    }
    fn unregister(&mut self, id: i32) -> Result<(), String> {
        windows_hotkey::unregister(self.app, id)
    }
    fn registered(&self, id: i32) -> bool {
        windows_hotkey::is_registered(self.app, id)
    }
    fn startup_snapshot(&self) -> Result<Option<RegistryValue>, String> {
        self.startup.snapshot()
    }
    fn startup_apply(&mut self, enabled: bool) -> Result<(), String> {
        self.startup.apply(enabled)
    }
    fn startup_restore(&mut self, previous: Option<&RegistryValue>) -> Result<(), String> {
        self.startup.restore(previous)
    }
    fn startup_description(&self) -> Result<String, String> {
        self.startup.describe()
    }
}
pub(crate) fn save(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    if state.busy.swap(true, Ordering::AcqRel) {
        return Err("a settings application is already active".into());
    }
    struct Guard<'a>(&'a AtomicBool);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }
    let _guard = Guard(&state.busy);
    let (candidate, active_key, active_token, store, startup) = {
        let session = state.lock()?;
        let session = session
            .as_ref()
            .ok_or("settings coordinator is not initialized")?;
        if session.listening {
            return Err("cannot Save while shortcut recording is active".into());
        }
        (
            session.draft.unwrap_or(session.desired),
            session.active_key,
            session.active_token,
            session.store.clone(),
            StartupRegistration::new(&session.profile),
        )
    };
    let startup = match startup {
        Ok(startup) => startup,
        Err(error) => {
            warn(app, &error);
            return Err(error);
        }
    };
    if active_token.is_none() {
        let window = app
            .get_webview_window("main")
            .ok_or("Drop Shelf window is missing")?;
        windows_hotkey::initialize(&window)?;
    }
    let mut integrations = NativeIntegrations { app, startup };
    let token = match settings_transaction::apply(
        &store,
        &mut integrations,
        candidate,
        active_key,
        active_token,
    ) {
        Ok(token) => token,
        Err(failure) => {
            {
                let mut session = state.lock()?;
                if let Some(session) = session.as_mut() {
                    session.active_key = failure.active_key;
                    session.active_token = failure.active_token;
                }
            }
            if let Some(warning) = failure.warning.as_ref() {
                warn(app, warning);
            }
            return Err(failure.message);
        }
    };
    {
        let mut session = state.lock()?;
        let session = session
            .as_mut()
            .ok_or("settings coordinator is not initialized")?;
        session.desired = candidate;
        session.active_key = candidate.hot_key;
        session.active_token = Some(token);
        session.draft = None;
        session.listening = false;
    }
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}
fn discard_draft(session: &mut SettingsSession) {
    session.draft = None;
    session.listening = false;
}
fn discard_session(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<SettingsState>();
    let mut session = state.lock()?;
    if let Some(session) = session.as_mut() {
        discard_draft(session);
    }
    Ok(())
}
pub(crate) fn cancel(app: &AppHandle) -> Result<(), String> {
    discard_session(app)?;
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}
pub(crate) fn on_settings_window_close_requested(app: &AppHandle) {
    if let Err(error) = discard_session(app) {
        warn(app, &error);
    }
}
pub(crate) fn on_settings_window_focus_changed(app: &AppHandle, focused: bool) {
    if !focused {
        if let Err(error) = cancel_recording(app) {
            warn(app, &error);
        }
    }
}
pub(crate) fn shutdown(app: &AppHandle) -> Result<(), String> {
    windows_hotkey::shutdown(app)?;
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.close().map_err(|error| error.to_string())?;
    }
    *app.state::<SettingsState>().lock()? = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_and_cancel_preserve_active_settings_file_and_discard_both_draft_fields() {
        let directory =
            std::env::temp_dir().join(format!("dropcove-record-draft-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let desired = Settings {
            hot_key: HotKey {
                control: true,
                alt: true,
                shift: false,
                windows: false,
                virtual_key: 0x87,
            },
            start_with_windows: true,
        };
        let candidate = HotKey {
            virtual_key: 0x4B,
            ..desired.hot_key
        };
        let mut session = SettingsSession {
            profile: directory.clone(),
            store: SettingsStore::new(&directory),
            desired,
            active_key: desired.hot_key,
            active_token: Some(12),
            draft: Some(Settings {
                start_with_windows: false,
                ..desired
            }),
            listening: true,
            startup_warning: None,
        };
        session.store.write(&desired).unwrap();
        let original = session.store.read_raw().unwrap();
        assert!(record_draft(
            &mut session,
            HotKey {
                control: false,
                alt: false,
                ..candidate
            }
        )
        .is_err());
        assert!(session.listening);
        record_draft(&mut session, candidate).unwrap();
        assert_eq!(
            session.draft,
            Some(Settings {
                hot_key: candidate,
                start_with_windows: false
            })
        );
        assert!(!session.listening);
        assert_eq!(session.active_key, desired.hot_key);
        assert_eq!(session.active_token, Some(12));
        assert_eq!(session.store.read_raw().unwrap(), original);
        assert!(record_draft(&mut session, desired.hot_key).is_err());
        discard_draft(&mut session);
        assert_eq!(session.draft, None);
        assert_eq!(session.desired, desired);
        assert_eq!(session.store.read_raw().unwrap(), original);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
