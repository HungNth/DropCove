# Tauri v2 NSIS and Windows Prerequisite Technical Research (DropCove 0.0.8)

This document establishes verified first-party technical facts, API contracts, and implementation options for DropCove 0.0.8 distribution hardening, based on official Tauri v2 source code and configuration references, NSIS v3 source code and documentation, and Microsoft Learn documentation.

---

## 1. Stock Tauri v2 NSIS Capabilities & Inherent Gaps

### 1.1 Verified Capabilities in Stock Tauri v2 (`installer.nsi`)

Tauri v2 generates its NSIS installer script (`installer.nsi`) from a handlebars template using `makensis` (NSIS 3.x with MUI2 - Modern UI 2). Inspection of the stock template in Tauri v2 (`tauri-bundler`) and the generated script in `src-tauri/target/x86_64-pc-windows-msvc/release/nsis/x64/installer.nsi` proves the following stock features:

1. **Install Mode (`currentUser` / `perMachine` / `both`)**:
   - Configured via `bundle.windows.nsis.installMode` (default `"currentUser"`).
   - In `currentUser` mode:
     - Sets `RequestExecutionLevel user` (runs without elevation).
     - Sets `!insertmacro SetContext` -> `SetShellVarContext current`.
     - Default target path: `StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"` (resolves to `%LOCALAPPDATA%\DropCove`).
   - *Primary Source*: Tauri Config Reference [`installMode`](https://v2.tauri.app/reference/config/#installmode), `installer.nsi:104-106, 504-506`.

2. **Remembered Install Directory**:
   - Stock installer saves the installed directory into the registry upon successful installation:
     `WriteRegStr SHCTX "${MANUPRODUCTKEY}" "" $INSTDIR`
     where `MANUPRODUCTKEY` is `Software\dropcove\DropCove` under `HKCU` in `currentUser` mode.
   - On subsequent runs (updates or reinstall), `RestorePreviousInstallLocation` reads:
     `ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""`
     `StrCmp $4 "" +2 0`
     `  StrCpy $INSTDIR $4`
   - *Primary Source*: `installer.nsi:659, 854-858`.

3. **Start Menu Shortcut**:
   - Stock installer writes a Start Menu shortcut during the `Install` section via `CreateOrUpdateStartMenuShortcut`:
     `CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"` (when `startMenuFolder` is unset), setting `AppUserModelId` via COM `IPropertyStore`.
   - Always created on fresh installation; updated if previously targeting an older binary name; skipped if in `/UPDATE` or `/NS` mode.
   - *Primary Source*: `installer.nsi:699-701, 871-911`.

4. **Desktop Shortcut**:
   - Stock installer provides `CreateOrUpdateDesktopShortcut` which writes `$DESKTOP\${PRODUCTNAME}.lnk`.
   - On the Finish page:
     ```nsis
     !define MUI_FINISHPAGE_SHOWREADME
     !define MUI_FINISHPAGE_SHOWREADME_TEXT "$(createDesktop)"
     !define MUI_FINISHPAGE_SHOWREADME_FUNCTION CreateOrUpdateDesktopShortcut
     ```
     By default, `MUI_FINISHPAGE_SHOWREADME_NOTCHECKED` is **not** set in stock Tauri. Therefore, the "Create desktop shortcut" checkbox on the Finish page is **checked by default** in stock Tauri.
   - For silent (`/S`) and passive (`/P`) modes, stock installer calls `CreateOrUpdateDesktopShortcut` automatically unless `/NS` is passed.
   - *Primary Source*: `installer.nsi:404-407, 703-708, 913-934`.

5. **Finish-Page Launch Option**:
   - Stock installer enables the Run checkbox on the Finish page:
     ```nsis
     !define MUI_FINISHPAGE_RUN
     !define MUI_FINISHPAGE_RUN_FUNCTION RunMainBinary
     ```
     where `RunMainBinary` calls `nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""`.
   - By default in NSIS MUI2, `MUI_FINISHPAGE_RUN` is **checked by default** unless `MUI_FINISHPAGE_RUN_NOTCHECKED` is defined.
   - *Primary Source*: `installer.nsi:409-416`, NSIS Modern UI 2 Documentation [Finish page settings](https://nsis.sourceforge.io/Docs/Modern%20UI%202/Readme.html).

6. **Custom Templates & Installer Hooks**:
   - `bundle.windows.nsis.template`: String path to a custom Handlebars `.nsi` template replacing Tauri's default `installer.nsi`.
   - `bundle.windows.nsis.installerHooks`: String path to a `.nsh` file included at the top of `installer.nsi` via `!include "{{installer_hooks}}"`.
   - Stock installer defines four hook macro insertion points:
     - `NSIS_HOOK_PREINSTALL`: In `Section Install` before copying files.
     - `NSIS_HOOK_POSTINSTALL`: In `Section Install` after copying files, registry writes, and shortcuts.
     - `NSIS_HOOK_PREUNINSTALL`: In `Section Uninstall` before deleting files.
     - `NSIS_HOOK_POSTUNINSTALL`: In `Section Uninstall` after removing files, registry keys, and shortcuts.
   - *Primary Source*: Tauri Docs [Windows Installer - Extending the Installer](https://v2.tauri.app/distribute/windows-installer/#extending-the-installer), `installer.nsi:24-26, 638-640, 710-712, 755-757, 843-845`.

7. **Update Behavior (Reinstall / Upgrade / Downgrade Page)**:
   - Stock installer detects prior installations by querying `ReadRegStr $R0 SHCTX "${UNINSTKEY}" ""` (`Software\Microsoft\Windows\CurrentVersion\Uninstall\DropCove`).
   - If found, it runs a custom `PageReinstall` comparing semver.
   - Upgrading offers radio options:
     - Radio 1: "Uninstall before installing" (executes old uninstaller with `/UPDATE _?=$INSTDIR`).
     - Radio 2: "Do not uninstall" (overwrites files directly in `$INSTDIR`).
   - If invoked with command-line `/UPDATE`, it skips the GUI prompt and executes in update mode directly.
   - *Primary Source*: `installer.nsi:181-380, 479-482`.

8. **Uninstall Registration & Hooks**:
   - Stock installer writes standard Windows Add/Remove Programs registry keys under `SHCTX` (`HKCU` for `currentUser`):
     `DisplayName`, `DisplayIcon`, `DisplayVersion`, `Publisher`, `InstallLocation`, `UninstallString` (`"$INSTDIR\uninstall.exe"`), `NoModify`, `NoRepair`, `EstimatedSize`.
   - Stock uninstaller:
     - Deletes main binary, shortcuts, uninstaller, and `$INSTDIR`.
     - Deletes `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value `"${PRODUCTNAME}"` (when `$UpdateMode <> 1`).
     - Deletes Uninstall registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\DropCove`.
     - Presents an optional checkbox: `"Delete the application data"`. If checked by user, it removes `$LOCALAPPDATA\${BUNDLEID}` and `$APPDATA\${BUNDLEID}`.
   - *Primary Source*: `installer.nsi:677-696, 753-852`.

---

### 1.2 Gaps Between Stock Tauri and DropCove Specification Requirements

The DropCove Product Specification (`docs/SPECIFICATION.md` §3.1-3.4) and Roadmap (`docs/ROADMAP.md` §0.0.8) require specific behaviors that diverge from stock Tauri:

| Requirement | DropCove 0.0.8 Contract | Stock Tauri v2 NSIS Behavior | Gap Analysis / Mechanism |
|---|---|---|---|
| **Default Install Directory** | `%LOCALAPPDATA%\Programs\DropCove` (§3.1 item 3) | Defaults to `$LOCALAPPDATA\${PRODUCTNAME}` (`%LOCALAPPDATA%\DropCove`) | **Gap**: Stock uses `%LOCALAPPDATA%\DropCove`, which collides with the profile directory. Must override in `.onInit` or custom template. |
| **Prerequisite Check** | Verify x64 VC++ 2015-2022 Redistributable compatible with binary; abort without network download if missing (§3.1 item 1) | Stock only checks WebView2; does not check VC++ Redistributable. | **Gap**: Requires custom check in `.onInit` or `NSIS_HOOK_PREINSTALL` querying official Microsoft registry keys. |
| **Desktop Shortcut Default** | Optional Desktop shortcut, **disabled by default** (§3.1 item 6) | `MUI_FINISHPAGE_SHOWREADME` is **checked by default** (`MUI_FINISHPAGE_SHOWREADME_NOTCHECKED` not defined). | **Gap**: Must define `MUI_FINISHPAGE_SHOWREADME_NOTCHECKED` before `MUI_PAGE_FINISH` (requires custom template or macro override). |
| **Wizard Page Sequence** | `Welcome → Directory → Shortcut options → Progress → Finish` (§3.1) | Stock has `Welcome → (License) → (Reinstall) → Directory → (StartMenu) → InstFiles → Finish`. Shortcuts are toggled on Finish page. | **Gap**: Stock puts Desktop shortcut checkbox on Finish page, not a dedicated "Shortcut options" page. |
| **Running App Shutdown** | Bounded ~5s graceful exit request; abort without force-kill if app does not exit (§3.3 item 2-4, §3.4) | Stock uses `CheckIfAppIsRunning` macro with Windows Restart Manager (`RmShutdown(RmForceShutdown)`) or messagebox offering to kill the app with `TerminateProcess`. | **Gap**: Stock force-terminates running processes! Must replace or intercept shutdown with graceful signaling and 5s polling. |
| **Profile Data Preservation** | Preserves `%LOCALAPPDATA%\DropCove` (`shelf.db`, `settings.json`, recovery) on uninstall (§3.4) | Stock uninstaller displays a checkbox: `"Delete the application data"`. If checked, it runs `RmDir /r "$LOCALAPPDATA\com.dropcove.app"`. | **Gap**: DropCove's canonical profile is `%LOCALAPPDATA%\DropCove` (not bundle ID). Must ensure canonical profile directory is never deleted by uninstaller. |
| **Autostart Cleanup on Uninstall** | Uninstaller removes **Start with Windows** registration (§3.4) | Stock uninstaller deletes `HKCU\...\Run\DropCove` when not in `/UPDATE` mode. | **Matches**: Stock uninstaller already contains this logic (`installer.nsi:821-823`). |

---

## 2. Running App Shutdown & Exit Verification

### 2.1 The Smallest Supported Graceful Shutdown Mechanism

In stock Tauri (`utils.nsh:24-75`), the `CheckIfAppIsRunning` macro calls:
```nsis
System::Call 'RSTRTMGR::RmShutdown(p R0, i ${RmForceShutdown}, p 0) i .r0'
```
Where `${RmForceShutdown}` is `0x1`. According to Microsoft Learn documentation (*RmShutdown function* and *Guidelines for Applications*):
- When `RmForceShutdown` is specified:
  - Restart Manager sends `WM_QUERYENDSESSION` (with `lParam = ENDSESSION_CLOSEAPP (0x1)`), then `WM_ENDSESSION`, then `WM_CLOSE`.
  - If the application does not shut down within the timeout period (or returns `FALSE` / hangs), Restart Manager **terminates the application forcibly** (equivalent to `TerminateProcess`).
- In addition, if Restart Manager fails or in passive/interactive fallback, stock `nsis_tauri_utils` provides `KillProcessCurrentUser`, which directly calls Win32 `TerminateProcess`.

#### Specification Contract:
> "setup requests the running resident process to exit; setup waits approximately five seconds; if DropCove does not exit, setup aborts and asks the user to close it" (SPECIFICATION.md §3.3 items 2-4, §3.4).

Forcing a kill violates the specification and risks corrupting SQLite WAL transactions or partial file writes.

#### Win32 / NSIS Implementation Options for Graceful Shutdown:
1. **Option A: Win32 Window Message (`WM_CLOSE` to Top-Level Window)**
   - DropCove's resident window (`SHELF_WINDOW_LABEL = "main"`) intercepts `tauri::WindowEvent::CloseRequested` and calls `resident::dismiss_shelf`, preventing actual process termination (`api.prevent_close()`). Therefore, `WM_CLOSE` to the "main" window **only dismisses the shelf**, it does not exit the resident process!
   - Win32 `WM_ENDSESSION` with `ENDSESSION_CLOSEAPP`: Currently DropCove has no custom window proc handling `WM_ENDSESSION` or `WM_QUERYENDSESSION`.

2. **Option B: Profile Lease Named Mutex / Custom Window Message (WM_COPYDATA or Registered Message)**
   - In `windows_instance.rs`, DropCove establishes:
     - Mutex: `Local\DropCove-Profile-{identity}`
     - HWND Property: `DropCove-Receiver-{identity}` with magic `0x4443_3031`.
     - Currently `receive()` handles `WM_COPYDATA` with `dwData == MAGIC && cbData == 0` by calling `resident::show_shelf`.
   - A dedicated command code in `COPYDATASTRUCT` (e.g. `dwData == MAGIC && cbData == 4` with command `CMD_SHUTDOWN = 0x2`) could invoke `crate::resident::exit(&receiver.app)`.

3. **Option C: Restart Manager without Force Flag (`RmShutdown(0)`) or `FindWindowW` + Lease Polling**
   - If Restart Manager is called with `lActionFlags = 0` (no `RmForceShutdown` flag):
     - `RmShutdown(hSession, 0, NULL)`: Sends `WM_QUERYENDSESSION` and `WM_ENDSESSION` / `WM_CLOSE`. If the application fails to shut down, `RmShutdown` returns `ERROR_FAIL_SHUTDOWN` (`351`) **without killing the process**.
   - However, for an unprivileged per-user utility with a known executable name `dropcove.exe`, NSIS can:
     1. Find target processes using `nsis_tauri_utils::FindProcessCurrentUser "dropcove.exe"`.
     2. If found, signal graceful exit (via `WM_CLOSE` / custom message or requesting user action).
     3. Poll for process or mutex exit for 5 seconds (10 iterations of 500 ms `Sleep`).
     4. If still running after 5 seconds: call `Abort "DropCove is running. Please exit DropCove from the tray icon before continuing."`.

---

## 3. Microsoft Visual C++ 2015-2022 Redistributable Detection

### 3.1 Binary Compatibility & Architecture Contract

- **DropCove Target**: `x86_64-pc-windows-msvc` (PE32+, 64-bit AMD64).
- **CRT Model in DropCove**:
  - `dropcove.exe` PE inspection (`llvm-objdump -p dropcove.exe`) reveals imports:
    - Universal CRT: `api-ms-win-crt-runtime-l1-1-0.dll`, `api-ms-win-crt-stdio-l1-1-0.dll`, `api-ms-win-crt-heap-l1-1-0.dll`, etc. (Included natively in Windows 10 and Windows 11).
    - Windows System DLLs: `user32.dll`, `kernel32.dll`, `shell32.dll`, `ole32.dll`, `dwmapi.dll`, `shcore.dll`.
    - No direct dynamic imports of `vcruntime140.dll` or `msvcp140.dll` exist in the current binary because `cargo-xwin` links against `libcmt.lib` / static MSVC CRT or Universal CRT forwarded symbols.
- However, `SPECIFICATION.md` §2 and §3.1 explicitly state:
  > Required prerequisite: x64 Microsoft Visual C++ Redistributable. The setup program checks for the x64 Microsoft Visual C++ Redistributable.
- **Microsoft v14 Binary Compatibility**:
  - Microsoft Visual Studio 2015, 2017, 2019, 2022, and 2026 all share major version **14** (v140, v141, v142, v143, v145).
  - Any installed version of the v14 Redistributable whose build version is equal to or greater than the build tools used to compile the binary satisfies all v14 MSVC dependencies.
  - The MSVC version in `cargo-xwin` CRT headers (`yvals_core.h`) is `_MSVC_STL_VERSION 143`, `_CPPLIB_VER 650`, corresponding to Visual Studio 2022 (MSVC v14.3x).

---

### 3.2 Official Microsoft Detection Method & Registry Evidence

According to official Microsoft Learn documentation (*Redistribute Visual C++ Files*, section *Install the redistributable packages*):

> "In Visual Studio 2015 and later, you can avoid this error by checking the registry to find out if a more recent version is installed. The current installed version number is stored in the `HKEY_LOCAL_MACHINE\SOFTWARE\Wow6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\{x86|x64|arm64}` key.
> The version number is 14.0 for Visual Studio 2015, 2017, 2019, and 2022 because the latest Redistributable is binary compatible with previous versions back to 2015. The key is `arm64`, `x86`, or `x64` depending on the installed `vcredist` versions for the platform...
> The version number is stored in the `REG_SZ` string value `Version` and also in the set of `Major`, `Minor`, `Bld`, and `Rbld` `REG_DWORD` values."

#### Registry Keys and Architecture Rules:
On 64-bit Windows running a 32-bit installer (NSIS is a 32-bit executable `x86-unicode`):
1. **64-bit Registry View (`SetRegView 64`)**:
   - Key: `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64`
   - Values:
     - `Installed` (`REG_DWORD`): `1` (indicates the runtime is installed).
     - `Major` (`REG_DWORD`): `14` (`0x0E`).
     - `Minor` (`REG_DWORD`): Minimum minor version (e.g. `>= 0x1E` for 2019/2022).
     - `Bld` (`REG_DWORD`): Build number (e.g. `>= 0x76B6` for 14.30+).
     - `Version` (`REG_SZ`): Formatted version string, e.g. `"v14.44.35211.00"`.
2. **32-bit WOW64 Registry View (`SetRegView 32`)**:
   - Accessing `HKLM\SOFTWARE\Wow6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64` also reflects the 64-bit runtime registration.

#### Verified Local Evidence on Windows 11 x64:
Direct execution of `reg query` on the build machine:
```text
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64
    Version    REG_SZ       v14.51.36247.00
    Installed  REG_DWORD    0x1
    Major      REG_DWORD    0xe
    Minor      REG_DWORD    0x33
    Bld        REG_DWORD    0x8d97
    Rbld       REG_DWORD    0x0
```

---

### 3.3 Official Microsoft Remediation URL / Permalink

According to Microsoft Learn (*Latest Supported Visual C++ Redistributable Downloads*, section *Latest supported redistributable version*):

- **Official Direct Download Permalink (x64)**:
  `https://aka.ms/vc14/vc_redist.x64.exe`
  (Alternatively: `https://aka.ms/vs/17/release/vc_redist.x64.exe`)
- **Official Documentation / Guidance Page**:
  `https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist`

#### Product Contract on Missing Prerequisite:
- DropCove requires **no network download behavior** inside the installer.
- If the prerequisite check fails, the installer must display an informative error dialog instructing the user to download and install the official x64 Redistributable from Microsoft's permalink, and then abort installation cleanly.

---

## 4. NSIS Mechanisms for Shortcuts, Install Location, Polling & Profile

### 4.1 Preserving Installer-Owned Shortcuts

- **Start Menu Shortcut**:
  - Shell path: `$SMPROGRAMS\DropCove.lnk` (in `currentUser` context, resolves to `%APPDATA%\Microsoft\Windows\Start Menu\Programs\DropCove.lnk`).
  - Stock Tauri macro `CreateOrUpdateStartMenuShortcut` invokes NSIS `CreateShortcut` and COM `SetLnkAppUserModelId`.
  - On update (`$UpdateMode == 1`), stock Tauri skips recreation if the shortcut already exists and preserves user customizations.
  - On uninstall (`Section Uninstall`), stock Tauri checks:
    ```nsis
    !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
      Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${EndIf}
    ```
    This verifies that the shortcut target points to DropCove's binary before deleting, preventing deletion of unrelated user shortcuts.

- **Desktop Shortcut**:
  - Shell path: `$DESKTOP\DropCove.lnk` (`%USERPROFILE%\Desktop\DropCove.lnk`).
  - Spec requirement: "offers an optional Desktop shortcut, disabled by default" (§3.1 item 6).
  - Modern UI 2 mechanism:
    `!define MUI_FINISHPAGE_SHOWREADME_NOTCHECKED`
    When defined before `!insertmacro MUI_PAGE_FINISH`, NSIS leaves the checkbox unchecked by default on the Finish page.
  - Uninstaller cleanup: Stock Tauri uses `IsShortcutTarget` against `$DESKTOP\${PRODUCTNAME}.lnk` and removes it if it points to `$INSTDIR\${MAINBINARYNAME}.exe`.

---

### 4.2 Storing & Remembering Install Location

- **Default Install Directory**:
  - Spec requirement: Defaults to `%LOCALAPPDATA%\Programs\DropCove`.
  - Stock Tauri default: `$LOCALAPPDATA\${PRODUCTNAME}` (`%LOCALAPPDATA%\DropCove`).
  - Customization:
    In `.onInit`, if `$INSTDIR` is uninitialized or placeholder, set:
    ```nsis
    StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    ```
- **Registry Key for Location**:
  - Stock Tauri registers the selected path:
    `WriteRegStr SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" "" $INSTDIR`
    (`HKCU\Software\dropcove\DropCove`, `(Default)` value).
  - In `RestorePreviousInstallLocation` (called during `.onInit`), it reads this key:
    ```nsis
    ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
    ${If} $4 != ""
      StrCpy $INSTDIR $4
    ${EndIf}
    ```
  - Standard Windows Uninstall registration also writes:
    `WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""`
  - On uninstall, stock Tauri preserves `HKCU\Software\dropcove\DropCove` unless the user explicitly checks "Delete application data". DropCove's specification (§3.4) states that uninstall removes "installer location registration". Therefore, `DeleteRegKey HKCU "Software\dropcove\DropCove"` belongs in uninstall.

---

### 4.3 Polling Process / Lease Exit for Five Seconds

The specification requires:
> "setup requests the running resident process to exit; setup waits approximately five seconds; if DropCove does not exit, setup aborts and asks the user to close it" (§3.3 items 2-4, §3.4).

#### NSIS Implementation Pattern:
Using standard NSIS instructions (`Sleep`, `IntOp`, `System::Call`):
```nsis
Function WaitForAppExit
  DetailPrint "Waiting for DropCove to close..."
  StrCpy $0 0 ; Loop counter (0 to 10 iterations of 500 ms = 5000 ms)

  wait_loop:
    nsis_tauri_utils::FindProcessCurrentUser "dropcove.exe"
    Pop $1 ; 0 = process found, 1 = no process found
    ${If} $1 == 1
      Goto app_closed
    ${EndIf}

    ${If} $0 >= 10
      Goto app_still_running
    ${EndIf}

    Sleep 500
    IntOp $0 $0 + 1
    Goto wait_loop

  app_still_running:
    MessageBox MB_ICONEXCLAMATION|MB_OK "DropCove is still running. Please exit DropCove from the notification area and run setup again."
    Abort "Installation aborted: DropCove is running."

  app_closed:
    DetailPrint "DropCove is closed."
FunctionEnd
```
*Note*: This pattern replaces `CheckIfAppIsRunning`'s `RmForceShutdown` behavior, ensuring zero risk of ungraceful process termination.

---

### 4.4 Preserving the Per-User Profile Directory

- **Profile Path**:
  `%LOCALAPPDATA%\DropCove` contains `shelf.db`, `settings.json`, and recovery snapshots (`shelf.corrupt-*.db`).
- **Install Path**:
  `%LOCALAPPDATA%\Programs\DropCove` contains `dropcove.exe` and `uninstall.exe`.
- **Preservation Contract**:
  - Separating install files into `%LOCALAPPDATA%\Programs\DropCove` guarantees that deleting `$INSTDIR` during uninstall never touches `%LOCALAPPDATA%\DropCove`.
  - Stock Tauri's uninstaller has a custom dialog checkbox: `$(deleteAppData)` (`"Delete the application data"`). If selected, it recursively deletes:
    `$APPDATA\com.dropcove.app` and `$LOCALAPPDATA\com.dropcove.app`.
  - To strictly satisfy DropCove specification (§3.4: *"Uninstall preserves: %LOCALAPPDATA%\DropCove. Therefore shelf.db, settings.json, and recovery data remain available to a later reinstall"*), the uninstaller must **never** offer or perform deletion of `%LOCALAPPDATA%\DropCove`. Removing the `un.ConfirmShow` / `un.ConfirmLeave` appdata checkbox or omitting `$LOCALAPPDATA\DropCove` from uninstaller deletion ensures 100% data preservation.

---

## 5. Concrete Recommendations for DropCove 0.0.8

### 5.1 Decision Matrix: Stock Config vs. Custom Template vs. Installer Hooks

| Feature | Stock Config Only? | Hook (`installerHooks`)? | Custom Template Required? | Justification & Supported Config Keys / Hooks |
|---|---|---|---|---|
| **Install Mode** | **Yes** | No | No | `"bundle" > "windows" > "nsis" > "installMode": "currentUser"` directly generates non-elevated `RequestExecutionLevel user`. |
| **Start Menu Shortcut** | **Yes** | No | No | Stock Tauri automatically creates Start Menu shortcut to `$INSTDIR\dropcove.exe` with AUMID. |
| **Finish-Page Launch Option** | **Yes** | No | No | Stock Tauri enables `MUI_FINISHPAGE_RUN` with `RunAsUser` by default. |
| **VC++ Redistributable Check** | No | **Yes** | No | Implemented in `NSIS_HOOK_PREINSTALL` or via custom hook in `.onInit`. Checks registry `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64`. If missing, shows MessageBox with aka.ms link and calls `Abort`. |
| **Remembered Install Dir** | **Yes** / Partial | **Yes** | No | Stock Tauri already writes and restores `HKCU\Software\dropcove\DropCove`. Overriding initial default to `%LOCALAPPDATA%\Programs\DropCove` can be set in `.onInit` or via hook before directory page. |
| **Optional Desktop Shortcut (Disabled by default)** | No | No | **Yes (Template)** | Stock `installer.nsi:405-407` defines `MUI_FINISHPAGE_SHOWREADME` without `MUI_FINISHPAGE_SHOWREADME_NOTCHECKED`. NSIS requires `!define MUI_FINISHPAGE_SHOWREADME_NOTCHECKED` *before* `!insertmacro MUI_PAGE_FINISH`. Because `installer_hooks` is included *before* page declarations, defining `MUI_FINISHPAGE_SHOWREADME_NOTCHECKED` in a hook **can** propagate to `MUI_PAGE_FINISH` if scoped globally! |
| **5-Second Bounded Shutdown Polling** | No | No | **Yes (Template or Check replacement)** | Stock `installer.nsi:642, 759` invokes `!insertmacro CheckIfAppIsRunning`, which executes `RmShutdown(RmForceShutdown)`. To prevent force-kill, `installer.nsi` template should replace `CheckIfAppIsRunning` with `WaitForAppExit`. |
| **Profile Data Preservation** | **Yes** | No | **Yes (Template cleanup)** | Stock uninstaller includes `DeleteAppDataCheckbox`. Removing this checkbox from the template ensures the user cannot accidentally delete their profile. |

---

### 5.2 Exact Configuration Keys, Macros, and Registry Definitions

#### 1. `src-tauri/tauri.conf.json` Settings:
```json
{
  "bundle": {
    "windows": {
      "allowDowngrades": true,
      "nsis": {
        "installMode": "currentUser",
        "installerHooks": "./windows/hooks.nsh",
        "template": "./windows/installer.nsi"
      }
    }
  }
}
```

#### 2. Visual C++ Prerequisite Registry Query Macro (in NSIS):
```nsis
!macro CheckVCRedistributableX64
  Push $0
  Push $1
  SetRegView 64
  ClearErrors
  ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ReadRegDWORD $1 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Major"
  SetRegView Default

  ${If} ${Errors}
  ${OrIf} $0 <> 1
  ${OrIf} $1 < 14
    MessageBox MB_ICONSTOP|MB_OK \
      "DropCove requires the Microsoft Visual C++ 2015-2022 Redistributable (x64).$\r$\n$\r$\nPlease download and install it from:$\r$\nhttps://aka.ms/vc14/vc_redist.x64.exe$\r$\n$\r$\nSetup will now exit."
    Abort
  ${EndIf}
  Pop $1
  Pop $0
!macroend
```

#### 3. Bounded 5-Second Graceful Shutdown Macro (Replacing Force-Kill):
```nsis
!macro BoundedAppShutdownPoll
  Push $0
  Push $1
  DetailPrint "Checking if DropCove is running..."
  StrCpy $0 0

  check_loop:
    nsis_tauri_utils::FindProcessCurrentUser "${MAINBINARYNAME}.exe"
    Pop $1
    ${If} $1 == 1
      Goto check_done
    ${EndIf}

    ${If} $0 == 0
      ; Inform the user on first detection
      DetailPrint "DropCove is running. Waiting for process exit (up to 5 seconds)..."
    ${EndIf}

    ${If} $0 >= 10
      MessageBox MB_ICONEXCLAMATION|MB_OK \
        "DropCove is currently running and could not be closed automatically.$\r$\n$\r$\nPlease exit DropCove from the notification area and run setup again."
      Abort "DropCove is currently running."
    ${EndIf}

    Sleep 500
    IntOp $0 $0 + 1
    Goto check_loop

  check_done:
    Pop $1
    Pop $0
!macroend
```

---

## 6. Unresolved Product Decisions & Grilling Questions

The research establishes the primary-source technical possibilities. The following product questions are ready for user review:

1. **Custom Template vs. Hook-Only Strategy**:
   - *Technical Fact*: Tauri v2 allows `installerHooks` (`hooks.nsh`), which can insert macros into `NSIS_HOOK_PREINSTALL` and `NSIS_HOOK_PREUNINSTALL`. However, stock Tauri hardcodes `CheckIfAppIsRunning` (which invokes `RmShutdown(RmForceShutdown)`) directly in `installer.nsi` outside of any hook macro.
   - *Question for User*: Should DropCove maintain a tracked custom `installer.nsi` template in `src-tauri/windows/installer.nsi` to achieve complete control over the wizard pages, default path (`%LOCALAPPDATA%\Programs\DropCove`), non-killing shutdown wait, and removal of the "Delete application data" checkbox? Or is an installer hook preferred with monkey-patching of macros? *(Recommendation: Use a custom template derived directly from stock Tauri v2.12 `installer.nsi`, cleanly fulfilling all 0.0.8 roadmap requirements without macro hacks).*

2. **Wizard Page Sequence for Shortcuts**:
   - *Technical Fact*: Modern UI 2 Finish page standardly contains the Desktop shortcut checkbox (`MUI_FINISHPAGE_SHOWREADME`) and Launch checkbox (`MUI_FINISHPAGE_RUN`). Stock Tauri does not have an independent "Shortcut options" wizard page between Directory and InstFiles; creating one requires a custom `nsDialogs` page.
   - *Question for User*: Does DropCove strictly require a separate wizard page (`Welcome → Directory → Shortcut options → Progress → Finish`), or is having the optional Desktop shortcut (unchecked by default) and Launch DropCove checkbox (checked by default) on the standard Finish page acceptable?

3. **Active Exit Request Signaling**:
   - *Technical Fact*: DropCove currently intercepts `WM_CLOSE` on its main window and merely dismisses the shelf (`api.prevent_close()`). It does not exit on `WM_CLOSE`.
   - *Question for User*: Should DropCove's resident activation receiver in `windows_instance.rs` support an explicit shutdown message command via `WM_COPYDATA` so the installer can actively ask the resident process to trigger `resident::exit()`, before polling process exit for 5 seconds?

---

## 7. Primary Source References

1. **Tauri v2 Documentation & Source**:
   - Tauri v2 Configuration Reference: <https://v2.tauri.app/reference/config/>
   - Tauri v2 Windows Installer Distribution: <https://v2.tauri.app/distribute/windows-installer/>
   - Tauri Bundler NSIS Module (`tauri-bundler/src/bundle/windows/nsis/mod.rs`): <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs>
   - Tauri Bundler Template (`installer.nsi`): <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi>
   - Tauri Bundler Utilities (`utils.nsh`): <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/utils.nsh>
   - Tauri Bundler PR #15372 (*Windows VC runtime linking and bundling options*): <https://github.com/tauri-apps/tauri/pull/15372>
   - Tauri NSIS Plugin Collection (`nsis-tauri-utils`): <https://github.com/tauri-apps/nsis-tauri-utils>

2. **NSIS (Nullsoft Scriptable Install System) Documentation & Source**:
   - NSIS Modern UI 2 Readme (`MUI2.nsh`): <https://nsis.sourceforge.io/Docs/Modern%20UI%202/Readme.html>
   - NSIS Scripting Reference (Instructions & Attributes): <https://raw.githubusercontent.com/kichik/nsis/master/Docs/src/attributes.but>
   - NSIS RestartManager Header (`RestartManager.nsh`): Local installation `C:\Users\Hung\AppData\Local\tauri\NSIS\Include\Win\RestartManager.nsh`

3. **Microsoft Documentation (Microsoft Learn & Win32 API)**:
   - Microsoft Learn: *Redistribute Visual C++ Files*: <https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files?view=msvc-170>
   - Microsoft Learn: *Latest Supported Visual C++ Redistributable Downloads*: <https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170>
   - Microsoft Learn: *C++ binary compatibility 2015-2026*: <https://learn.microsoft.com/en-us/cpp/porting/binary-compat-2015-2017?view=msvc-170>
   - Microsoft Learn: *RmShutdown function (restartmanager.h)*: <https://learn.microsoft.com/en-us/windows/win32/api/restartmanager/nf-restartmanager-rmshutdown>
   - Microsoft Learn: *Guidelines for Applications (Restart Manager)*: <https://learn.microsoft.com/en-us/windows/win32/rstmgr/guidelines-for-applications>
   - Microsoft Learn: *WM_CLOSE message*: <https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close>
   - Microsoft Learn: *WM_ENDSESSION message*: <https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-endsession>
