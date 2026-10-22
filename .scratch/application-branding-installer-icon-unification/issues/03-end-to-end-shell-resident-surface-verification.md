# 03: End-to-end Windows shell and resident surface verification (Release gate)

**What to build:**
Perform end-to-end verification of DropCove 0.0.21 on Windows 11 across all 6 shell and resident surfaces. Record the observed evidence in `docs/ROADMAP.md` and present the verified candidate for user acceptance.

**Parent specification:** `.scratch/application-branding-installer-icon-unification/spec.md`

**Blocked by:** 02-packaging-pe-icon-resource-validation

**Status:** resolved — user tested and accepted on 2026-10-22.

## Acceptance criteria

- [x] The installer binary `DropCove-Setup-0.0.21.exe` displays the DropCove icon in Windows File Explorer.
- [x] Running the installer wizard displays the DropCove icon on its window title bar and Windows Taskbar.
- [x] Installed `uninstall.exe` in `%LOCALAPPDATA%\Programs\DropCove` displays the DropCove icon in File Explorer.
- [x] Add or Remove Programs displays the DropCove icon for the application entry; registry `DisplayIcon` points to the installed application.
- [x] Start Menu and Desktop shortcuts resolve to the installed executable and inherit its branded icon; OS-extracted shortcut icon visuals were inspected.
- [x] In-app resident surfaces consistently display the Brand Visual in Dark, Light, and Contrast modes.
- [x] Roadmap records exact artifact hashes/sizes, observed checks, restoration, and remaining limits.

## Testing seam

Live Windows 11 interactive installation smoke test, registry verification, and in-app visual verification across appearance modes.

## Demo path

Run installer, inspect Explorer, launch app from Start Menu, verify System Tray and About dialog, inspect Windows Installed Apps, and verify clean uninstall.

## Comments

- Interactive installation completed successfully with Desktop shortcut selected and Launch unchecked. Actual Explorer showed the setup icon and installed application/uninstaller icons. Programs and Features showed the branded entry. The notification-area overflow showed the DropCove tray icon. Existing native icons were present on Drop Shelf, Edge Rail, About, and Settings; no window visibility policy changed.
- Exact installed WebView2 About loaded embedded assets and reported 0.0.21. Settings appearance preview produced observed Dark and Light About rendering. Forced-colors emulation is not actual Windows Contrast Theme evidence.
- Standard uninstall, after public profile-scoped shutdown and closing PE inspection handles, removed both executable files, the owned installation directory, installer registry, and owned shortcuts without manual deletion; isolated Profile Data survived.
- An early smoke failure was caused by an open PE memory map. Another interactive reinstall attempted while the custom-profile candidate was running correctly reported a copy failure. These invalid smoke runs are not release passes; the later non-conflicting installation and uninstall are the lifecycle evidence.
- Restored the existing 0.0.20 installation registration and Start Menu through its installer. Existing executable SHA-256 remained unchanged. Its Resident Instance reconciled startup registration to the ambient install and durable StartWithWindows preference; settings bytes stayed unchanged. Original pre-smoke Run/registry/shortcut bytes were not captured, so exact byte-for-byte restoration is not claimed.
- Human gate: install the retained 0.0.21 candidate at the normal path, confirm Setup Taskbar and default-path uninstall branding plus actual Windows Contrast Theme, and approve the release. No release completion or commit is claimed before that gate.
- The user confirmed complete verification and user acceptance on 2026-10-22 (`tất cả đã đạt`, `tôi đã test và tất cả đã đạt`), closing human acceptance and authorizing the release commit.
