# 04: Cooperative Update and Uninstall

**What to build:** Make repair, update, and uninstall coordinate with the active Resident Instance through profile-scoped Cooperative Shutdown. Setup must wait at most five seconds for normal teardown, abort without force-killing when shutdown cannot complete, remove only owned program artifacts, and preserve the entire Profile Data directory.

**Blocked by:** 03 — Current-User Installer Wizard.

**Status:** resolved

**Parent specification:** [DropCove 0.0.8 — Recovery and Distribution Hardening](../spec.md)

## Acceptance criteria

- [ ] `dropcove.exe --shutdown` is handled before Tauri presentation startup and exits successfully without opening a surface when no Resident Instance exists.
- [ ] With a matching Resident Instance, shutdown mode sends a distinct profile-scoped request through the existing receiver, waits for normal teardown and lease release, and returns a stable success/failure exit status.
- [ ] A resident busy with drag, resize, persistence, or integration work is not force-cancelled; repeated requests may succeed at its next safe point within the five-second deadline.
- [ ] Setup and uninstaller abort with a clear close-DropCove instruction when Cooperative Shutdown cannot complete within five seconds.
- [ ] No update, repair, uninstall, or fallback path uses Restart Manager force shutdown, `TerminateProcess`, task killing, or an unbounded wait.
- [ ] A newer setup updates the remembered installation, the same version repairs it, and an older setup is blocked.
- [ ] Program-file replacement starts only after successful shutdown. A copy/registration failure aborts and preserves Profile Data; rerunning setup is the repair path rather than a custom binary rollback transaction.
- [ ] Update preserves install location, Profile Data, approved shortcuts, and user-selected Desktop shortcut state.
- [ ] Uninstall removes installed program files, Installed Apps registration, installer-location registration, and only Installer-Owned Shortcuts.
- [ ] Uninstall removes the Start with Windows value only when its target and arguments prove DropCove ownership; a same-named mismatched value is preserved and warned.
- [ ] Update and uninstall preserve the complete per-user profile, including unknown future files and every Recovery Backup, byte-for-byte.
- [ ] Interactive, passive, and silent lifecycle paths apply the same shutdown, version, ownership, and preservation rules.

## Testing seam

Exercise the pre-UI shutdown command and receiver contract with native process tests, then verify the exact installed setup/uninstaller against real process, lease, registry, shortcut, install-directory, and Profile Data state.

## Demo path

Install DropCove, populate Profile Data, enable Start with Windows, and keep the Resident Instance running. Run newer setup and observe clean exit plus update. Hold the resident busy past five seconds and observe abort without termination. Uninstall and compare the complete profile while confirming only owned program artifacts were removed.

## Non-goals

Automatic update download, service-based coordination, force termination, operation cancellation for setup, transactional rollback of program files, Profile Data deletion, or downgrade support.

## Answer

- Added pre-UI profile-scoped `--shutdown`; native receiver uses normal resident teardown and confirms OS process exit plus profile lease release within one five-second deadline. Missing profiles are not created, and no force-kill path remains in setup/uninstall.
- Five native instance tests passed, including safe-point retries, timeout rejection, argument conflicts, and process/lease lifetime.
- Actual embedded resident shutdown returned 0 and resident exited 0; absent-profile shutdown returned 0 without creating a directory.
- Exact setup repaired a running owned installation and cleanly stopped its resident. Holding a real native resize caused setup to abort with exit 2 after 5.859 seconds including setup overhead; resident remained running and subsequently closed cleanly when the interaction ended.
- Exact setup rejected a downgrade with exit 2 before writing an executable. Uninstall returned 0, removed owned shortcuts and owned Run registration, preserved a mismatched ambient Run registration, and left `shelf.db` plus an unknown profile file byte-for-byte unchanged.
- Non-empty Start Menu folder cleanup was exercised after fixing shared shortcut path selection. Captured ambient registrations and user resident were preserved.
- Final newer-version update and complete exact-artifact acceptance remain release checks in ticket 05.
