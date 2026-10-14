# Use a custom NSIS template and profile-scoped Cooperative Shutdown

DropCove maintains a minimal custom template derived from the pinned Tauri NSIS template because stock behavior conflicts with the required install path, wizard sequence, shortcut defaults, Profile Data preservation, and no-force update policy. Update and uninstall invoke the installed executable in a profile-scoped shutdown mode, wait at most five seconds for normal Resident Instance teardown, and abort rather than force-kill; the template must be reviewed whenever Tauri is upgraded.
