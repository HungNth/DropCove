//! One settings application across native integrations and the real atomic JSON store.

use crate::{
    settings_store::{HotKey, Settings, SettingsStore},
    windows_startup::RegistryValue,
};

pub(crate) trait Integrations {
    type RailSnapshot;
    fn rail_snapshot(&self) -> Result<Self::RailSnapshot, String>;
    fn rail_apply(&mut self, settings: &Settings) -> Result<(), String>;
    fn rail_restore(&mut self, previous: &Self::RailSnapshot) -> Result<(), String>;
    fn rail_description(&self) -> Result<String, String>;
    fn register(&mut self, key: &HotKey) -> Result<i32, String>;
    fn unregister(&mut self, id: i32) -> Result<(), String>;
    fn registered(&self, id: i32) -> bool;
    fn startup_snapshot(&self) -> Result<Option<RegistryValue>, String>;
    fn startup_apply(&mut self, enabled: bool) -> Result<(), String>;
    fn startup_restore(&mut self, previous: Option<&RegistryValue>) -> Result<(), String>;
    fn startup_description(&self) -> Result<String, String>;
}

pub(crate) struct ApplicationFailure {
    pub(crate) message: String,
    pub(crate) warning: Option<String>,
    pub(crate) active_key: HotKey,
    pub(crate) active_token: Option<i32>,
}

fn failure(
    message: String,
    active_key: HotKey,
    active_token: Option<i32>,
    notify: bool,
) -> ApplicationFailure {
    ApplicationFailure {
        warning: notify.then(|| message.clone()),
        message,
        active_key,
        active_token,
    }
}

pub(crate) fn apply(
    store: &SettingsStore,
    integrations: &mut impl Integrations,
    candidate: &Settings,
    active_key: HotKey,
    active_token: Option<i32>,
) -> Result<i32, ApplicationFailure> {
    candidate
        .validate()
        .map_err(|error| failure(error, active_key, active_token, false))?;
    let previous_file = store
        .read_raw()
        .map_err(|error| failure(error, active_key, active_token, true))?;
    let previous_startup = integrations
        .startup_snapshot()
        .map_err(|error| failure(error, active_key, active_token, true))?;
    let previous_rail = integrations
        .rail_snapshot()
        .map_err(|error| failure(error, active_key, active_token, true))?;
    let changed = active_token.is_none() || active_key != candidate.hot_key;
    let token = if changed {
        integrations
            .register(&candidate.hot_key)
            .map_err(|error| failure(error, active_key, active_token, false))?
    } else {
        // changed is false only for a retained active registration.
        active_token
            .ok_or_else(|| failure("active shortcut disappeared".into(), active_key, None, true))?
    };
    let mut file_attempted = false;
    let mut rail_attempted = false;
    let result = (|| {
        integrations.startup_apply(candidate.start_with_windows)?;
        rail_attempted = true;
        integrations.rail_apply(candidate)?;
        file_attempted = true;
        store.write(candidate)?;
        if changed {
            if let Some(old) = active_token {
                integrations.unregister(old)?;
            }
        }
        Ok::<(), String>(())
    })();
    let Err(error) = result else {
        return Ok(token);
    };

    let mut rollback_errors = Vec::new();
    if file_attempted {
        // A rejected atomic replacement normally leaves the original bytes intact.
        // Do not rewrite an unchanged/share-locked file and fabricate a rollback failure.
        if store.read_raw().as_ref().ok() != Some(&previous_file) {
            if let Err(restore) = store.restore_raw(previous_file.as_deref()) {
                rollback_errors.push(format!("settings restore: {restore}"));
            }
        }
    }
    if rail_attempted {
        if let Err(restore) = integrations.rail_restore(&previous_rail) {
            rollback_errors.push(format!("rail restore: {restore}"));
        }
    }
    if integrations.startup_snapshot().as_ref().ok() != Some(&previous_startup) {
        if let Err(restore) = integrations.startup_restore(previous_startup.as_ref()) {
            rollback_errors.push(format!("startup restore: {restore}"));
        }
    }
    if changed {
        if let Err(restore) = integrations.unregister(token) {
            rollback_errors.push(format!("candidate shortcut removal: {restore}"));
        }
    }
    let old_retained = active_token.filter(|id| integrations.registered(*id));
    let candidate_retained = changed && integrations.registered(token);
    let retained_token = old_retained.or(candidate_retained.then_some(token));
    let retained_key = if old_retained.is_some() || !candidate_retained {
        active_key
    } else {
        candidate.hot_key
    };
    if rollback_errors.is_empty() {
        return Err(failure(
            format!("Settings were not applied: {error}"),
            retained_key,
            retained_token,
            true,
        ));
    }

    let startup = integrations
        .startup_description()
        .unwrap_or_else(|query| format!("query failed: {query}"));
    let file = match store.read_raw() {
        Ok(bytes) if bytes == previous_file => "original bytes retained".to_owned(),
        Ok(Some(bytes)) => match serde_json::from_slice::<Settings>(&bytes) {
            Ok(settings) => format!(
                "hotkey {}; StartWithWindows {}",
                settings.hot_key.label(),
                settings.start_with_windows
            ),
            Err(_) => "present but not readable as current settings".to_owned(),
        },
        Ok(None) => "absent".to_owned(),
        Err(query) => format!("query failed: {query}"),
    };
    let rail = integrations
        .rail_description()
        .unwrap_or_else(|query| format!("query failed: {query}"));
    let message = format!(
        "Settings failed: {error}; rollback incomplete: {}. Retained state: old shortcut registered={}; candidate registered={candidate_retained}; active shortcut={}; startup={startup}; rail={rail}; settings file={file}",
        rollback_errors.join("; "), old_retained.is_some(),
        retained_token.map(|_| retained_key.label()).unwrap_or_else(|| "none (tray/launch access remains)".into()),
    );
    Err(failure(message, retained_key, retained_token, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Profile(PathBuf);
    impl Profile {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "dropcove-settings-transaction-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Profile {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    struct Native {
        keys: BTreeMap<i32, HotKey>,
        startup: Option<RegistryValue>,
        rail: Settings,
        fail_rail: bool,
        fail_rail_restore: bool,
        conflict: bool,
        fail_apply: bool,
        fail_restore: bool,
        fail_retire: bool,
    }
    fn old_startup() -> RegistryValue {
        RegistryValue {
            kind: 3,
            bytes: vec![1, 5, 9],
        }
    }
    fn new_startup() -> RegistryValue {
        RegistryValue {
            kind: 1,
            bytes: vec![2, 0, 0, 0],
        }
    }
    impl Integrations for Native {
        type RailSnapshot = Settings;
        fn rail_snapshot(&self) -> Result<Settings, String> {
            Ok(self.rail.clone())
        }
        fn rail_apply(&mut self, settings: &Settings) -> Result<(), String> {
            self.rail = settings.clone();
            if self.fail_rail {
                return Err("rail placement failed after effect".into());
            }
            Ok(())
        }
        fn rail_restore(&mut self, previous: &Settings) -> Result<(), String> {
            if self.fail_rail_restore {
                return Err("native rail restore rejected".into());
            }
            self.rail = previous.clone();
            Ok(())
        }
        fn rail_description(&self) -> Result<String, String> {
            Ok(format!(
                "{:?} {}",
                self.rail.rail_edge, self.rail.rail_monitor_id
            ))
        }
        fn register(&mut self, key: &HotKey) -> Result<i32, String> {
            if self.conflict {
                return Err("shortcut occupied".into());
            }
            self.keys.insert(2, *key);
            Ok(2)
        }
        fn unregister(&mut self, id: i32) -> Result<(), String> {
            if self.fail_retire && id == 1 {
                return Err("old registration could not retire".into());
            }
            self.keys.remove(&id);
            Ok(())
        }
        fn registered(&self, id: i32) -> bool {
            self.keys.contains_key(&id)
        }
        fn startup_snapshot(&self) -> Result<Option<RegistryValue>, String> {
            Ok(self.startup.clone())
        }
        fn startup_apply(&mut self, enabled: bool) -> Result<(), String> {
            self.startup = enabled.then(new_startup);
            if self.fail_apply {
                return Err("startup write rejected after effect".into());
            }
            Ok(())
        }
        fn startup_restore(&mut self, previous: Option<&RegistryValue>) -> Result<(), String> {
            if self.fail_restore {
                return Err("startup restore denied".into());
            }
            self.startup = previous.cloned();
            Ok(())
        }
        fn startup_description(&self) -> Result<String, String> {
            Ok(if self.startup == Some(new_startup()) {
                "candidate Run command"
            } else if self.startup.is_some() {
                "previous Run value"
            } else {
                "absent"
            }
            .into())
        }
    }
    fn fixture(profile: &Profile) -> (SettingsStore, Native, Settings, Settings) {
        let store = SettingsStore::new(&profile.0);
        let old = Settings::default();
        store.write(&old).unwrap();
        let candidate = Settings {
            hot_key: HotKey {
                control: true,
                alt: true,
                shift: false,
                windows: false,
                virtual_key: 0x4B,
            },
            start_with_windows: false,
            rail_edge: crate::settings_store::RailEdge::Left,
            rail_monitor_id: "other-display".into(),
            ..Settings::default()
        };
        let native = Native {
            keys: BTreeMap::from([(1, old.hot_key)]),
            startup: Some(old_startup()),
            rail: old.clone(),
            fail_rail: false,
            fail_rail_restore: false,
            conflict: false,
            fail_apply: false,
            fail_restore: false,
            fail_retire: false,
        };
        (store, native, old, candidate)
    }
    #[test]
    fn conflict_and_failed_startup_preserve_old_file_registration_and_exact_run_value() {
        for fail_startup in [false, true] {
            let profile = Profile::new();
            let (store, mut native, old, candidate) = fixture(&profile);
            let original_file = store.read_raw().unwrap();
            native.conflict = !fail_startup;
            native.fail_apply = fail_startup;
            let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
                .err()
                .unwrap();
            assert_eq!(failure.active_token, Some(1));
            assert_eq!(native.keys, BTreeMap::from([(1, old.hot_key)]));
            assert_eq!(native.startup, Some(old_startup()));
            assert_eq!(store.read_raw().unwrap(), original_file);
        }
    }
    #[test]
    fn failed_atomic_json_write_unwinds_startup_and_candidate_without_rewriting_locked_original() {
        use std::os::windows::fs::OpenOptionsExt;
        let profile = Profile::new();
        let (store, mut native, old, candidate) = fixture(&profile);
        let original_file = store.read_raw().unwrap();
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(profile.0.join("settings.json"))
            .unwrap();
        let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
            .err()
            .unwrap();
        assert!(!failure.message.contains("rollback incomplete"));
        assert_eq!(native.startup, Some(old_startup()));
        assert_eq!(native.keys, BTreeMap::from([(1, old.hot_key)]));
        assert_eq!(store.read_raw().unwrap(), original_file);
        assert_eq!(native.rail, old);
        drop(lock);
    }
    #[test]
    fn failed_old_retirement_restores_committed_file_and_run_before_releasing_candidate() {
        let profile = Profile::new();
        let (store, mut native, old, candidate) = fixture(&profile);
        let original_file = store.read_raw().unwrap();
        native.fail_retire = true;
        let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
            .err()
            .unwrap();
        assert_eq!(failure.active_key, old.hot_key);
        assert_eq!(native.startup, Some(old_startup()));
        assert_eq!(native.keys, BTreeMap::from([(1, old.hot_key)]));
        assert_eq!(store.read_raw().unwrap(), original_file);
    }
    #[test]
    fn failed_restore_reports_actual_retained_state_and_allows_a_later_retry() {
        let profile = Profile::new();
        let (store, mut native, old, mut candidate) = fixture(&profile);
        candidate.start_with_windows = true;
        native.fail_apply = true;
        native.fail_restore = true;
        let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
            .err()
            .unwrap();
        assert!(failure
            .warning
            .as_ref()
            .unwrap()
            .contains("rollback incomplete"));
        assert!(failure.message.contains("startup=candidate Run command"));
        assert_eq!(failure.active_token, Some(1));
        assert_eq!(store.load().0, old);
        assert_eq!(native.startup, Some(new_startup()));
        native.fail_apply = false;
        native.fail_restore = false;
        assert_eq!(
            apply(&store, &mut native, &candidate, old.hot_key, Some(1)).ok(),
            Some(2)
        );
        assert_eq!(store.load().0, candidate);
        assert_eq!(native.keys, BTreeMap::from([(2, candidate.hot_key)]));
    }
    #[test]
    fn unchanged_explicit_save_retries_missing_run_registration_without_duplicate_shortcut() {
        let profile = Profile::new();
        let (store, mut native, old, _) = fixture(&profile);
        native.startup = None;
        assert_eq!(
            apply(&store, &mut native, &old, old.hot_key, Some(1)).ok(),
            Some(1)
        );
        assert_eq!(native.keys, BTreeMap::from([(1, old.hot_key)]));
        assert_eq!(native.startup, Some(new_startup()));
        assert_eq!(store.load().0, old);
    }
    #[test]
    fn failed_rail_application_restores_disk_startup_shortcut_and_previous_placement() {
        let profile = Profile::new();
        let (store, mut native, old, candidate) = fixture(&profile);
        native.fail_rail = true;
        let original = store.read_raw().unwrap();
        let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
            .err()
            .unwrap();
        assert_eq!(failure.active_token, Some(1));
        assert_eq!(native.rail, old);
        assert_eq!(native.keys, BTreeMap::from([(1, old.hot_key)]));
        assert_eq!(native.startup, Some(old_startup()));
        assert_eq!(store.read_raw().unwrap(), original);
    }
    #[test]
    fn failed_rail_restore_reports_retained_candidate_and_later_save_can_retry() {
        let profile = Profile::new();
        let (store, mut native, old, candidate) = fixture(&profile);
        native.fail_rail = true;
        native.fail_rail_restore = true;
        let failure = apply(&store, &mut native, &candidate, old.hot_key, Some(1))
            .err()
            .unwrap();
        assert_eq!(native.rail, candidate);
        assert_eq!(store.load().0, old);
        assert_eq!(failure.active_token, Some(1));
        assert!(failure
            .warning
            .as_ref()
            .unwrap()
            .contains("Left other-display"));
        native.fail_rail = false;
        native.fail_rail_restore = false;
        assert_eq!(
            apply(&store, &mut native, &candidate, old.hot_key, Some(1)).ok(),
            Some(2)
        );
        assert_eq!(store.load().0, candidate);
    }
}
