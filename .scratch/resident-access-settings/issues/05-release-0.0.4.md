# 05: Release DropCove 0.0.4 from the installed artifact

Status: resolved
Type: task
Version: 0.0.4
Parent: [Specification](../spec.md)
Blocked by: 04
Implementation approval: approved by user on 2026-10-13

## What to build

Deliver the approved Resident access and Settings behavior as a version-consistent Windows x64 current-user installer, prove it from the installed executable, then stop for user review without beginning 0.0.5.

## Acceptance criteria

- [x] Tickets 01–04 are resolved with permanent coordinator/settings tests and actual Windows process/surface evidence.
- [x] VERSION, package/crate/lock/Tauri metadata, PE/runtime/installed version and artifact name consistently report 0.0.4.
- [x] Build with cargo-xwin for x86_64-pc-windows-msvc and produce `DropCove-Setup-0.0.4.exe` using current-user NSIS packaging without Visual Studio/MSVC Build Tools.
- [x] Execute the newly compiled integrated permanent suite; compilation or an old test executable is not counted as a pass.
- [x] Installed packaged executable launched with owned profile and embedded frontend.
- [x] Secondary process exited 0 and restored hidden primary presentation without a second state owner.
- [x] Shelf/hotkey/secondary access retained nonempty state; both available cursor-monitor work areas exercised.
- [x] Native tray Settings/Exit, delivered Windows warning history and foreground topmost repair without focus theft observed.
- [x] Settings recording/Save-disabled/Cancel, occupied shortcut, successful new shortcut and retired old shortcut, missing/malformed/share-denied settings and post-effect atomic rollback exercised.
- [x] Startup On/Off, quoted Run, startup failure/retry, post-effect Run rollback and hidden packaged startup followed by hotkey activation exercised.
- [x] Available two-monitor 96-DPI placement exercised; mixed-DPI limits stated without inference.
- [x] Owned native Exit/uninstaller executed; owned data removed; original Run, uninstall registration, shortcuts and ambient executable restored.
- [x] Exercised evidence recorded; 0.0.4 Completed, 0.0.5+ unstarted, no additional commit created.

## Testing seam

Run the integrated Resident Application Coordinator and real-settings suite once, then prove Win32 integration from the actual installed artifact. Process existence alone, mock echoes, Tauri menu construction, installer exit status and DOM-only events are not completion evidence.

## Demo path

Install 0.0.4, launch a primary with an owned profile, hide/add state, activate through a secondary/tray/hotkey, record and Save settings, verify rollback conflict, enable hidden startup and relaunch with `--autostart`, then Exit from tray and show cleanup/version evidence.

## Comments

## Release evidence

- Canonical VERSION and package/crate/lock/Tauri metadata report 0.0.4. Built and installed PE FileVersion/ProductVersion strings report `0.0.4`; installed runtime observations are recorded below.
- `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc && node scripts/copy-installer.cjs` succeeded, including vue-tsc/Vite and current-user NSIS packaging. Vendor CRT PDB linker warnings remain nonfatal. Import-only startup formatting check passed.
- Fresh 0.0.4 suite compiled through `cargo xwin test --target x86_64-pc-windows-msvc --lib --no-run`; direct execution of `dropcove_lib-215e9db13fca8c7a.exe` passed **66/66**, zero failures.
- Installer paths `build/DropCove-Setup-0.0.4.exe` and `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/DropCove_0.0.4_x64-setup.exe` are byte-identical, 3,141,424 bytes. SHA-256: `23ca00c1a3871a422703e8093fdbc350f16342cb98a4037d9eda97c2642d9408`.
- Initial harness failures were not counted as passes. A bounded native menu probe queried the actual popup HMENU and dispatched its Settings/Exit command to its owning tray window; Settings HWND and embedded CDP target were observed. Later installed cases below closed the missing gates.


### Installed runtime observations

- Installed `C:\DropCove_004_TestInstall\dropcove.exe`: 12,911,616 bytes; SHA-256 `b1ea49f8f74156fbad217f902b88ade633b3788876f8cce6679414270746d60f`; PE version strings and current-user uninstall DisplayVersion reported 0.0.4. Owned profile and isolated WebView folder used throughout.
- Secondary launch returned 0, primary remained alive and hidden Shelf became visible. Nonempty state survived hotkey/secondary access. Held Ctrl+Shift+Space for one second caused one toggle, not repeat toggles. Cursor-monitor show bounds: `[402,406,598,595]` on primary and `[2402,406,2598,595]` on secondary, each inside its native work area.
- Actual tray popup exposed Settings/Exit. Its Settings command opened native Settings plus `DropCove Settings` CDP target. Removing Shelf topmost and foregrounding an owned WinForms probe restored topmost while foreground stayed on the probe. Native Exit returned 0, removed all owned windows and closed SQLite WAL/SHM.
- Recorder pointer/Enter initiation, Escape, Save disabled during listening, Ctrl+Alt+K capture and Cancel were observed. Separately reserved Ctrl+Alt+J rejected Save with `0x80070581`, retaining prior active state. Valid K Save persisted VK75; K toggled and the old default chord no longer did.
- Real post-effect rollback: `FILE_SHARE_READ` lock permitted original JSON reads but denied replacement. Candidate L plus startup Off failed with `ReplaceFileW` error32; prior VK75 JSON and quoted Run were restored, K remained active and L was released. An earlier exclusive-lock preflight failure was not substituted for this rollback proof.
- Missing JSON loaded defaults without writing. Malformed JSON startup PID 31244 retained exact text and default shortcut; Windows toast history increased 15→16 with `Settings file is malformed: EOF while parsing a string at line 1 column 47`. Truly unreadable startup PID 20960 used `CreateFileW` share mode 0, loaded defaults, preserved bytes after unlocking; toast history increased 16→17 with `Could not open settings file ... (os error 32)`. Corrupt readable bytes were not counted as access failure.
- Startup On/Off removed/recreated only DropCove Run. Quoted command: `"C:\DropCove_004_TestInstall\dropcove.exe" --autostart --profile "\\?\C:\DropCove_004_Profile"`. Oversized owned Run data (70,000 bytes) induced registration query failure with desired On retained; after clearing the injected obstruction, unchanged Save recreated valid registration. Packaged autostart remained hidden and K later exposed Shelf.

### Cleanup and limits

- Owned NSIS uninstaller `/S _?=C:\DropCove_004_TestInstall` returned 0 and removed its binary. Owned install/profile/WebView paths are absent; no smoke process remains. DropCove Run returned to absent and five unrelated values were preserved. Ambient executable hash remained `7583ab748d4d84736babe696daafffd276bb95d821f5ea1f748188f5e3d826ab`.
- A cleanup defect initially left uninstall registration pointing at the deleted test path; this was repaired, not counted as success. Parent independently re-read original uninstall DisplayVersion 0.0.3/location/uninstall path, original ambient binary hash, absent owned paths/Run and both shortcut targets after final cleanup.
- Two physical monitors both 96 DPI; live mixed-DPI scaling was unavailable. Hidden launch command was exercised directly, not actual sign-out/sign-in. Warning delivery was observed in Windows toast history, not transient banner visibility. Native popup command dispatch bypassed unreliable overflow pointer selection; actual popup/menu owner, Settings surface and clean Exit were observed.
- User's existing installation remains 0.0.3 deliberately; only the isolated smoke installation was 0.0.4. Release artifact is ready for user installation. No commit created; no 0.0.5 work started.
