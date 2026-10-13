# 04: Start with Windows and Hidden Start

Status: resolved
Type: task
Version: 0.0.4
Parent: [Specification](../spec.md)
Blocked by: 03
Implementation approval: approved by user on 2026-10-13

## What to build

Settings adds Start with Windows and applies it transactionally with HotKey and `settings.json`. Sign-in launch restores the resident in `Hidden` state with tray/global access initialized and no Shelf surface shown.

## Acceptance criteria

- [x] Settings exposes Start with Windows, default On, with exactly `HotKey` plus `StartWithWindows` persisted.
- [x] On/Off changes only the owned current-user DropCove Run value.
- [x] Quoted executable/profile command includes `--autostart`; Windows parser boundary tests cover escaping and the 260 UTF-16-unit limit.
- [x] Hidden primary restores Shelf/settings and initializes tray/hotkey/startup without showing presentation.
- [x] Tray, hotkey and normal secondary launch expose the hidden resident; autostart secondary does not.
- [x] Failed default startup registration warns, retains desired On and permits unchanged Save retry.
- [x] Save coordinates candidate hotkey, startup registration and atomic JSON before retiring old registration.
- [x] Hotkey/startup/JSON failures restore prior state when rollback succeeds.
- [x] Incomplete rollback queries retained state, keeps Settings open and reports the actual result.
- [x] Cancel/close preserves Run, active hotkey and original JSON.
- [x] Missing/malformed/unreadable settings retain documented defaults and source-file safety.
- [x] Optional integration failures remain recoverable warnings.
- [x] Startup belongs to the application; NSIS configuration has no startup hook. Final packaged installation smoke belongs to ticket 05.

## Testing seam

Coordinator tests use the real temporary Settings Adapter plus deterministic hotkey/startup adapters, injecting failure at each step and during rollback. Actual Windows smoke snapshots/restores any pre-existing owned Run value, verifies quoted command data and exercises normal versus `--autostart` process surfaces with an owned profile.

## Demo path

Enable Start with Windows and Save, inspect the owned current-user registration, exit and launch with its hidden-start command, observe only the resident tray/integrations, then show from tray/hotkey. Disable and Save, verify removal, and exercise a forced registration failure with prior hotkey/settings retained.

## Non-goals

Installer-managed startup, Task Scheduler, machine-wide startup, Edge Rail and Shake-to-open are out of scope.

## Comments

## Answer

Ticket 04 resolved. `windows_startup.rs` owns the current-user Run value; `settings_transaction.rs` coordinates native integration and real atomic JSON rollback. Settings exposes the startup draft and `windows_instance.rs` routes hidden launch intent. No compatibility schema or installer-managed startup was added.

### Exercised verification

- Final-source `cargo xwin build --release --target x86_64-pc-windows-msvc --features tauri/custom-protocol` succeeded. Test compilation succeeded through cargo-xwin; its automatic runner tried unavailable `wine`, so the newly compiled Windows test executable was run directly: **66 passed, 0 failed**. Only missing vendor CRT PDB linker warnings remain; the unused production import was removed.
- Actual Windows executable with owned paths containing spaces: exact quoted Run command launched hidden, normal secondary activated the resident, autostart secondary stayed hidden, and hotkey/tray access worked. Hidden restart restored the exact nonempty Shelf snapshot and 300×220 sizing.
- Real Settings surface: On/Off Save changed only DropCove Run; joint hotkey/startup Save persisted the full schema; draft/Cancel changed neither Run nor file. Windows file-sharing denial forced atomic JSON failure: prior Run/file restored, old F23 remained active and candidate F24 was released.
- First-run oversized owned Run data forced startup query failure without changing Run-key ACLs or unrelated entries. Desired On remained visible, Windows toast history contained the warning, and unchanged Save retried successfully. Malformed old intermediate schema and genuinely unreadable files loaded defaults without rewriting their bytes.
- Permanent coordinator checks inject conflict, startup failure, old-registration retirement failure and incomplete rollback, using a real temporary settings store. Native smoke restored the exact pre-existing DropCove value (absent) and all five unrelated entries. Owned processes exited; attached tabs closed; owned profile/binary/WebView/fixtures were removed. No ambient profile or user installation was changed by ticket 04.

### Coverage limits and schema cutover

- Actual sign-out/sign-in, final 0.0.4 installer/update behavior and mixed-DPI/multi-monitor hardware were not exercised here; ticket 05 owns packaged release proof. Incomplete rollback is deterministic adapter evidence, not an induced live system failure.
- Intermediate HotKey-only JSON is now malformed: defaults plus warning, original bytes untouched until successful explicit Save. Final JSON requires both fields; no migration is provided.
- Version metadata remains 0.0.3. No commit or 0.0.4 release was created.

