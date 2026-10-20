//! Resident-owned Settings draft and one native/disk application transaction.

use crate::{
    resident,
    settings_store::{HotKey, RailEdge, Settings, SettingsStore, ShakeSensitivity},
    settings_transaction::{self, Integrations},
    windows_hotkey,
    windows_auxiliary::{self, InvocationContext},
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
const SETTINGS_HEIGHT: f64 = 520.0;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsSnapshot {
    pub(crate) hot_key: HotKey,
    pub(crate) label: String,
    pub(crate) listening: bool,
    pub(crate) start_with_windows: bool,
    pub(crate) rail_edge: RailEdge,
    pub(crate) rail_monitor_id: String,
    pub(crate) show_rail_over_fullscreen: bool,
    pub(crate) shake_enabled: bool,
    pub(crate) shake_sensitivity: ShakeSensitivity,
    pub(crate) monitors: Vec<crate::edge_rail::MonitorChoice>,
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
    if let Err(error) = crate::windows_shake::initialize(window, &desired) {
        warnings.push(format!("Shake-to-open unavailable: {error}; tray, hotkey, Shelf and Rail remain available; Save can retry"));
    }
    app.manage(SettingsState {
        session: Mutex::new(Some(SettingsSession {
            profile: profile.to_path_buf(),
            store,
            desired: desired.clone(),
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
            Ok(mut session) => session
                .as_mut()
                .and_then(|session| session.startup_warning.take()),
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
    let settings = session.draft.as_ref().unwrap_or(&session.desired);
    Ok(SettingsSnapshot {
        hot_key: settings.hot_key,
        label: settings.hot_key.label(),
        listening: session.listening,
        start_with_windows: settings.start_with_windows,
        rail_edge: settings.rail_edge,
        rail_monitor_id: settings.rail_monitor_id.clone(),
        show_rail_over_fullscreen: settings.show_rail_over_fullscreen,
        shake_enabled: settings.shake_enabled,
        shake_sensitivity: settings.shake_sensitivity,
        monitors: crate::edge_rail::choices()?,
    })
}
fn emit_snapshot(app: &AppHandle) -> Result<SettingsSnapshot, String> {
    let snapshot = get_snapshot(app)?;
    app.emit(SETTINGS_EVENT, &snapshot)
        .map_err(|error| error.to_string())?;
    Ok(snapshot)
}
pub(crate) fn open(app: &AppHandle, context: InvocationContext) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show().map_err(|error| error.to_string())?;
        window.unminimize().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        emit_snapshot(app)?;
        return Ok(());
    }
    let monitor = windows_auxiliary::target_monitor(app, context)?;
    let window = WebviewWindowBuilder::new(
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
    .always_on_top(false)
    .visible(false)
    .build()
    .map_err(|error| error.to_string())?;
    let setup = (|| {
        windows_auxiliary::center_in_work_area(&window, &monitor)?;
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        emit_snapshot(app)?;
        Ok(())
    })();
    if setup.is_err() {
        let _ = window.close();
    }
    setup
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
    session
        .draft
        .get_or_insert_with(|| session.desired.clone())
        .hot_key = key;
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
            .get_or_insert_with(|| session.desired.clone())
            .start_with_windows = enabled;
    }
    emit_snapshot(app)
}
pub(crate) fn active(app: &AppHandle) -> Result<Settings, String> {
    let state = app.state::<SettingsState>();
    let session = state.lock()?;
    Ok(session
        .as_ref()
        .ok_or("settings coordinator is not initialized")?
        .desired
        .clone())
}
pub(crate) fn set_rail_placement(
    app: &AppHandle,
    edge: RailEdge,
    monitor_id: String,
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
        let mut candidate = session.draft.as_ref().unwrap_or(&session.desired).clone();
        candidate.rail_edge = edge;
        candidate.rail_monitor_id = monitor_id;
        candidate.validate()?;
        session.draft = Some(candidate);
    }
    emit_snapshot(app)
}
pub(crate) fn set_show_rail_over_fullscreen(
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
        let mut candidate = session.draft.as_ref().unwrap_or(&session.desired).clone();
        candidate.show_rail_over_fullscreen = enabled;
        candidate.validate()?;
        session.draft = Some(candidate);
    }
    emit_snapshot(app)
}
pub(crate) fn set_shake(
    app: &AppHandle,
    enabled: bool,
    sensitivity: ShakeSensitivity,
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
        let candidate = session.draft.get_or_insert_with(|| session.desired.clone());
        candidate.shake_enabled = enabled;
        candidate.shake_sensitivity = sensitivity;
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
        resident::Visibility::Visible => resident::dismiss_shelf(app),
        resident::Visibility::Hidden | resident::Visibility::EdgeDocked => {
            resident::show_shelf(app)
        }
    }
}

struct NativeIntegrations<'a> {
    app: &'a AppHandle,
    startup: StartupRegistration,
}
impl Integrations for NativeIntegrations<'_> {
    type RailSnapshot = Option<crate::edge_rail::Placement>;
    type ShakeSnapshot = crate::windows_shake::RuntimeSnapshot;
    fn rail_snapshot(&self) -> Result<Self::RailSnapshot, String> {
        resident::rail_placement(self.app)
    }
    fn rail_apply(&mut self, settings: &Settings) -> Result<(), String> {
        resident::apply_rail_settings(self.app, settings)
    }
    fn rail_restore(&mut self, previous: &Self::RailSnapshot) -> Result<(), String> {
        resident::restore_rail(self.app, previous.as_ref())
    }
    fn rail_description(&self) -> Result<String, String> {
        crate::edge_rail::description(self.app)
    }
    fn shake_snapshot(&self) -> Result<Self::ShakeSnapshot, String> {
        crate::windows_shake::snapshot(self.app)
    }
    fn shake_apply(&mut self, settings: &Settings) -> Result<(), String> {
        let window = self
            .app
            .get_webview_window("main")
            .ok_or("Drop Shelf window is missing")?;
        crate::windows_shake::initialize(&window, settings)
    }
    fn shake_restore(&mut self, previous: &Self::ShakeSnapshot) -> Result<(), String> {
        crate::windows_shake::restore(self.app, previous)
    }
    fn shake_description(&self) -> Result<String, String> {
        crate::windows_shake::description(self.app)
    }
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
            session.draft.as_ref().unwrap_or(&session.desired).clone(),
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
        &candidate,
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
        session.active_key = candidate.hot_key;
        session.active_token = Some(token);
        session.desired = candidate;
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
    crate::windows_shake::shutdown(app)?;
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
            ..Settings::default()
        };
        let candidate = HotKey {
            virtual_key: 0x4B,
            ..desired.hot_key
        };
        let mut session = SettingsSession {
            profile: directory.clone(),
            store: SettingsStore::new(&directory),
            desired: desired.clone(),
            active_key: desired.hot_key,
            active_token: Some(12),
            draft: Some(Settings {
                start_with_windows: false,
                ..desired.clone()
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
                start_with_windows: false,
                ..desired.clone()
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
