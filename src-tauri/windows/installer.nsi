; DropCove custom NSIS installer template.
; Derived from the pinned @tauri-apps/cli 2.12.2 stock `installer.nsi` template.
; Review this template against upstream Tauri changes whenever @tauri-apps/cli or tauri-bundler is upgraded.

Unicode true
ManifestDPIAware true
; Add in `dpiAwareness` `PerMonitorV2` to manifest for Windows 10 1607+ (note this should not affect lower versions since they should be able to ignore this and pick up `dpiAware` `true` set by `ManifestDPIAware true`)
; Currently undocumented on NSIS's website but is in the Docs folder of source tree, see
; https://github.com/kichik/nsis/blob/5fc0b87b819a9eec006df4967d08e522ddd651c9/Docs/src/attributes.but#L286-L300
; https://github.com/tauri-apps/tauri/pull/10106
ManifestDPIAwareness PerMonitorV2

!if "{{compression}}" == "none"
  SetCompress off
!else
  ; Set the compression algorithm. We default to LZMA.
  SetCompressor /SOLID "{{compression}}"
!endif

; Keep above !include to stay ahead of any plugin command
; see https://github.com/tauri-apps/tauri/pull/15422#discussion_r3289239624
{{#if signed_plugins_path}}
!addplugindir "{{signed_plugins_path}}"
{{/if}}

!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh
!include "utils.nsh"
!include "FileAssociation.nsh"
!include "Win\COM.nsh"
!include "Win\Propkey.nsh"
!include "Win\RestartManager.nsh"
!include "StrFunc.nsh"
${StrLoc}

{{#if installer_hooks}}
!include "{{installer_hooks}}"
{{/if}}

!define WEBVIEW2APPGUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

!define MANUFACTURER "{{manufacturer}}"
!define PRODUCTNAME "{{product_name}}"
!define VERSION "{{version}}"
!define VERSIONWITHBUILD "{{version_with_build}}"
!define HOMEPAGE "{{homepage}}"
!define INSTALLMODE "{{install_mode}}"
!define LICENSE "{{license}}"
!define INSTALLERICON "{{installer_icon}}"
!define SIDEBARIMAGE "{{sidebar_image}}"
!define HEADERIMAGE "{{header_image}}"
!define UNINSTALLERICON "{{uninstaller_icon}}"
!define UNINSTALLERHEADERIMAGE "{{uninstaller_header_image}}"
!define MAINBINARYNAME "{{main_binary_name}}"
!define MAINBINARYSRCPATH "{{main_binary_path}}"
!define BUNDLEID "{{bundle_id}}"
!define COPYRIGHT "{{copyright}}"
!define OUTFILE "{{out_file}}"
!define ARCH "{{arch}}"
!define ADDITIONALPLUGINSPATH "{{additional_plugins_path}}"
!define ALLOWDOWNGRADES "{{allow_downgrades}}"
!define DISPLAYLANGUAGESELECTOR "{{display_language_selector}}"
!define INSTALLWEBVIEW2MODE "{{install_webview2_mode}}"
!define WEBVIEW2INSTALLERARGS "{{webview2_installer_args}}"
!define WEBVIEW2BOOTSTRAPPERPATH "{{webview2_bootstrapper_path}}"
!define WEBVIEW2INSTALLERPATH "{{webview2_installer_path}}"
!define MINIMUMWEBVIEW2VERSION "{{minimum_webview2_version}}"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"
!define MANUKEY "Software\${MANUFACTURER}"
!define MANUPRODUCTKEY "${MANUKEY}\${PRODUCTNAME}"
!define UNINSTALLERSIGNCOMMAND "{{uninstaller_sign_cmd}}"
!define ESTIMATEDSIZE "{{estimated_size}}"
!define STARTMENUFOLDER "{{start_menu_folder}}"
!if "${STARTMENUFOLDER}" != ""
  !define STARTMENUSHORTCUT "$SMPROGRAMS\${STARTMENUFOLDER}\${PRODUCTNAME}.lnk"
!else
  !define STARTMENUSHORTCUT "$SMPROGRAMS\${PRODUCTNAME}.lnk"
!endif

Var PassiveMode
Var UpdateMode
Var NoShortcutMode
Var OldMainBinaryName
Var PreviousInstallDir
Var StartMenuShortcutCheckbox
Var DesktopShortcutCheckbox
Var DesktopShortcutRequested

Name "${PRODUCTNAME}"
BrandingText "${COPYRIGHT}"
OutFile "${OUTFILE}"

; We don't actually use this value as default install path,
; it's just for nsis to append the product name folder in the directory selector
; https://nsis.sourceforge.io/Reference/InstallDir
!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"
InstallDir "${PLACEHOLDER_INSTALL_DIR}"

VIProductVersion "${VERSIONWITHBUILD}"
VIAddVersionKey "ProductName" "${PRODUCTNAME}"
VIAddVersionKey "FileDescription" "${PRODUCTNAME}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

# additional plugins
!addplugindir "${ADDITIONALPLUGINSPATH}"

; Uninstaller signing command
!if "${UNINSTALLERSIGNCOMMAND}" != ""
  !uninstfinalize '${UNINSTALLERSIGNCOMMAND}'
!endif

; Handle install mode, `perUser`, `perMachine` or `both`
!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel admin
!endif

!if "${INSTALLMODE}" == "currentUser"
  RequestExecutionLevel user
!endif

!if "${INSTALLMODE}" == "both"
  !define MULTIUSER_MUI
  !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"
  !define MULTIUSER_INSTALLMODE_COMMANDLINE
  !if "${ARCH}" == "x64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !else if "${ARCH}" == "arm64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !endif
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "${UNINSTKEY}"
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "CurrentUser"
  !define MULTIUSER_INSTALLMODEPAGE_SHOWUSERNAME
  !define MULTIUSER_INSTALLMODE_FUNCTION RestorePreviousInstallLocation
  !define MULTIUSER_EXECUTIONLEVEL Highest
  !include MultiUser.nsh
!endif

; Installer icon
!if "${INSTALLERICON}" != ""
  !define MUI_ICON "${INSTALLERICON}"
!endif

; Installer sidebar image
!if "${SIDEBARIMAGE}" != ""
  !define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBARIMAGE}"
!endif

; Enable header images for installer and uninstaller pages when either image is configured.
!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
!else if "${UNINSTALLERHEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
!endif

; Installer header image
!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE_BITMAP "${HEADERIMAGE}"
!endif

; Uninstaller header image
!if "${UNINSTALLERHEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE_UNBITMAP "${UNINSTALLERHEADERIMAGE}"
!endif

; Uninstaller icon
!if "${UNINSTALLERICON}" != ""
  !define MUI_UNICON "${UNINSTALLERICON}"
!endif

; Define registry key to store installer language
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "${MANUPRODUCTKEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "Installer Language"

; ============================================================================
; Interactive Wizard Sequence:
;   Welcome -> Directory -> Shortcut options -> Installation progress -> Finish
; ============================================================================

; 1. Welcome Page
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_WELCOME

; 2. Directory Page
!define MUI_DIRECTORYPAGE_VERIFYONLEAVE
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!define MUI_PAGE_CUSTOMFUNCTION_LEAVE DirectoryPageLeave
!insertmacro MUI_PAGE_DIRECTORY

; 3. Shortcut options Page
Page custom ShortcutOptionsPageCreate ShortcutOptionsPageLeave

; 4. Installation progress Page
!insertmacro MUI_PAGE_INSTFILES

; 5. Finish Page
; Don't auto jump to finish page after installation page,
; because the installation page has useful info that can be used to debug any issues with the installer.
!define MUI_FINISHPAGE_NOAUTOCLOSE
; Present Launch DropCove checked by default on the Finish page.
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Launch ${PRODUCTNAME}"
!define MUI_FINISHPAGE_RUN_FUNCTION RunMainBinary
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_FINISH

Function RunMainBinary
  nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""
FunctionEnd

Function DirectoryPageLeave
  GetInstDirError $0
  ${If} $0 <> 0
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "The selected installation directory is unavailable or invalid:$\r$\n$INSTDIR$\r$\n$\r$\nPlease select a valid, writable directory."
    Abort
  ${EndIf}

  Call ValidateInstallDir
  ${If} $0 <> 1
    MessageBox MB_ICONEXCLAMATION|MB_OK \
      "The selected installation directory is unavailable or not writable:$\r$\n$INSTDIR$\r$\n$\r$\nPlease select a valid, writable directory."
    Abort
  ${EndIf}
FunctionEnd

Function ShortcutOptionsPageCreate
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}

  !insertmacro MUI_HEADER_TEXT "Shortcut options" "Choose how ${PRODUCTNAME} shortcuts are created."
  nsDialogs::Create 1018
  Pop $0
  ${IfThen} $0 == error ${|} Abort ${|}
  ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

  ${NSD_CreateLabel} 0 0 100% 24u "${PRODUCTNAME} always creates a Start Menu shortcut so the application remains discoverable. You can also create an optional Desktop shortcut."
  Pop $0

  ${NSD_CreateCheckbox} 0 32u 100% 12u "Create Start Menu shortcut (required)"
  Pop $StartMenuShortcutCheckbox
  ${NSD_Check} $StartMenuShortcutCheckbox
  EnableWindow $StartMenuShortcutCheckbox 0

  ${NSD_CreateCheckbox} 0 50u 100% 12u "$(createDesktop)"
  Pop $DesktopShortcutCheckbox
  ${If} $DesktopShortcutRequested = 1
    ${NSD_Check} $DesktopShortcutCheckbox
  ${EndIf}

  ${NSD_SetFocus} $DesktopShortcutCheckbox
  nsDialogs::Show
FunctionEnd

Function ShortcutOptionsPageLeave
  ${NSD_GetState} $DesktopShortcutCheckbox $0
  ${If} $0 == ${BST_CHECKED}
    StrCpy $DesktopShortcutRequested 1
  ${Else}
    StrCpy $DesktopShortcutRequested 0
  ${EndIf}
FunctionEnd

; Uninstaller Pages (no application-data / Profile Data deletion checkbox)
; 1. Confirm uninstall page
!define MUI_PAGE_CUSTOMFUNCTION_PRE un.SkipIfPassive
!insertmacro MUI_UNPAGE_CONFIRM

; 2. Uninstalling Page
!insertmacro MUI_UNPAGE_INSTFILES

;Languages
{{#each languages}}
!insertmacro MUI_LANGUAGE "{{this}}"
{{/each}}
!insertmacro MUI_RESERVEFILE_LANGDLL
{{#each language_files}}
  !include "{{this}}"
{{/each}}

Function .onInit
  StrCpy $DesktopShortcutRequested 0

  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/NS" $NoShortcutMode
  ${IfNot} ${Errors}
    StrCpy $NoShortcutMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}

  !if "${DISPLAYLANGUAGESELECTOR}" == "true"
    !insertmacro MUI_LANGDLL_DISPLAY
  !endif

  !insertmacro SetContext

  ; Enforce 64-bit Microsoft Visual C++ Redistributable prerequisite before proceeding
  Call CheckVCRedistributableX64

  ; Record any previously registered install location for shortcut ownership checks
  ReadRegStr $PreviousInstallDir SHCTX "${MANUPRODUCTKEY}" ""
  ReadRegStr $OldMainBinaryName SHCTX "${UNINSTKEY}" "MainBinaryName"

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location: %LOCALAPPDATA%\Programs\DropCove for currentUser
    !if "${INSTALLMODE}" == "perMachine"
      ${If} ${RunningX64}
        !if "${ARCH}" == "x64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else if "${ARCH}" == "arm64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else
          StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
        !endif
      ${Else}
        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
      ${EndIf}
    !else if "${INSTALLMODE}" == "currentUser"
      StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    !endif

    Call RestorePreviousInstallLocation
  ${EndIf}

  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_INIT
  !endif

  ; Reject downgrades across interactive, passive, update, and silent runs
  !if "${ALLOWDOWNGRADES}" == "false"
    ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
    ${If} $R0 != ""
      nsis_tauri_utils::SemverCompare "${VERSION}" $R0
      Pop $R0
      ${If} $R0 = -1
        SetErrorLevel 2
        ${IfNot} ${Silent}
          MessageBox MB_ICONSTOP|MB_OK "A newer version of ${PRODUCTNAME} is already installed. Downgrading is not supported."
        ${EndIf}
        Abort
      ${EndIf}
    ${EndIf}
  !endif
FunctionEnd

Section InstallChecks
  ; Validate prerequisite, target directory, Start Menu shortcut ownership, and resident shutdown
  ; BEFORE WebView2 or any program-file or registry mutations.
  Call CheckVCRedistributableX64

  Call ValidateInstallDir
  ${If} $0 <> 1
    DetailPrint "Invalid or unwritable installation directory: $INSTDIR"
    SetErrorLevel 2
    ${IfNot} ${Silent}
      MessageBox MB_ICONSTOP|MB_OK \
        "The selected installation directory is unavailable or not writable:$\r$\n$INSTDIR"
    ${EndIf}
    Abort
  ${EndIf}

  Call VerifyStartMenuShortcutOwnership

  !ifmacrodef NSIS_HOOK_PREINSTALL
    !insertmacro NSIS_HOOK_PREINSTALL
  !endif

  InitPluginsDir
  ClearErrors
  File "/oname=$PLUGINSDIR\${MAINBINARYNAME}.exe" "${MAINBINARYSRCPATH}"
  ${If} ${Errors}
    DetailPrint "Failed to prepare cooperative shutdown helper."
    SetErrorLevel 2
    Abort
  ${EndIf}
  ClearErrors
  ExecWait '"$PLUGINSDIR\${MAINBINARYNAME}.exe" --shutdown' $0
  ${If} ${Errors}
  ${OrIf} $0 <> 0
    Delete "$PLUGINSDIR\${MAINBINARYNAME}.exe"
    DetailPrint "Aborting: ${PRODUCTNAME} did not exit within 5 seconds."
    SetErrorLevel 2
    ${IfNot} ${Silent}
      MessageBox MB_ICONEXCLAMATION|MB_OK \
        "${PRODUCTNAME} is currently running and did not close within 5 seconds.$\r$\n$\r$\nPlease close ${PRODUCTNAME} from the notification area and run setup again."
    ${EndIf}
    Abort
  ${EndIf}
SectionEnd

Section WebView2
  ; Check if Webview2 is already installed and skip this section
  ${If} ${RunningX64}
    ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${Else}
    ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}
  ${If} $4 == ""
    ReadRegStr $4 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}

  ${If} $4 == ""
    ; Webview2 installation
    ;
    ; Skip if updating
    ${If} $UpdateMode <> 1
      !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        DetailPrint "$(webview2Downloading)"
        NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Pop $0
        ${If} $0 == "success"
          DetailPrint "$(webview2DownloadSuccess)"
        ${Else}
          DetailPrint "$(webview2DownloadError)"
          SetErrorLevel 2
          Abort "$(webview2AbortError)"
        ${EndIf}
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
        Delete "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        Goto install_webview2
      !endif

      Goto webview2_done

      install_webview2:
        DetailPrint "$(installingWebview2)"
        ; $6 holds the path to the webview2 installer
        ExecWait "$6 ${WEBVIEW2INSTALLERARGS} /install" $1
        ${If} $1 = 0
          DetailPrint "$(webview2InstallSuccess)"
        ${Else}
          DetailPrint "$(webview2InstallError)"
          SetErrorLevel 2
          Abort "$(webview2AbortError)"
        ${EndIf}
      webview2_done:
    ${EndIf}
  ${Else}
    !if "${MINIMUMWEBVIEW2VERSION}" != ""
      ${VersionCompare} "${MINIMUMWEBVIEW2VERSION}" "$4" $R0
      ${If} $R0 = 1
        update_webview:
          DetailPrint "$(installingWebview2)"
          ${If} ${RunningX64}
            ReadRegStr $R1 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate" "path"
          ${Else}
            ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 == ""
            ReadRegStr $R1 HKCU "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 != ""
            ; Chromium updater docs: https://source.chromium.org/chromium/chromium/src/+/main:docs/updater/user_manual.md
            ; Modified from "HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView\ModifyPath"
            ExecWait `"$R1" /install appguid=${WEBVIEW2APPGUID}&needsadmin=true` $1
            ${If} $1 = 0
              DetailPrint "$(webview2InstallSuccess)"
            ${Else}
              ${If} ${Silent}
                SetErrorLevel 2
                Abort "$(webview2InstallError)"
              ${EndIf}
              MessageBox MB_ICONEXCLAMATION|MB_ABORTRETRYIGNORE "$(webview2InstallError)" IDIGNORE ignore IDRETRY update_webview
              SetErrorLevel 2
              Quit
              ignore:
            ${EndIf}
          ${EndIf}
      ${EndIf}
    !endif
  ${EndIf}
SectionEnd

Section Install
  ClearErrors
  SetOutPath $INSTDIR

  ; Copy main executable from verified staging plugin directory
  CopyFiles /SILENT "$PLUGINSDIR\${MAINBINARYNAME}.exe" "$INSTDIR\${MAINBINARYNAME}.exe"
  Delete "$PLUGINSDIR\${MAINBINARYNAME}.exe"

  ; Copy resources
  {{#each resources_dirs}}
    CreateDirectory "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources}}
    File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
  {{/each}}

  ; Copy external binaries
  {{#each binaries}}
    File /a "/oname={{this}}" "{{no-escape @key}}"
  {{/each}}

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ${If} ${Errors}
    DetailPrint "Failed to write program files to $INSTDIR."
    SetErrorLevel 2
    Abort
  ${EndIf}

  ; Create file associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
       !insertmacro APP_ASSOCIATE "{{ext}}" "{{or association.name ext}}" "{{association-description association.description ext}}" "$INSTDIR\${MAINBINARYNAME}.exe,0" "Open with ${PRODUCTNAME}" "$INSTDIR\${MAINBINARYNAME}.exe $\"%1$\""
    {{/each}}
  {{/each}}

  ; Register deep links
  {{#each deep_link_protocols as |protocol| ~}}
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "URL Protocol" ""
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "" "URL:${BUNDLEID} protocol"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\DefaultIcon" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
  {{/each}}

  ; Create or update mandatory Start Menu shortcut
  ClearErrors
  Call CreateOrUpdateStartMenuShortcut
  ${If} ${Errors}
    DetailPrint "Failed to create Start Menu shortcut."
    SetErrorLevel 2
    Abort
  ${EndIf}

  ; Create or update optional Desktop shortcut (respects ownership and interactive selection)
  Call CreateOrUpdateDesktopShortcut

  ; Remove old main binary if it doesn't match new main binary name
  ${If} $OldMainBinaryName != ""
  ${AndIf} $OldMainBinaryName != "${MAINBINARYNAME}.exe"
    Delete "$INSTDIR\$OldMainBinaryName"
  ${EndIf}

  ; Register $INSTDIR and Add/Remove Programs metadata only after program files and shortcuts succeed
  ClearErrors
  WriteRegStr SHCTX "${MANUPRODUCTKEY}" "" $INSTDIR

  !if "${INSTALLMODE}" == "both"
    WriteRegStr SHCTX "${UNINSTKEY}" $MultiUser.InstallMode 1
  !endif

  WriteRegStr SHCTX "${UNINSTKEY}" "MainBinaryName" "${MAINBINARYNAME}.exe"
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayIcon" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "${MANUFACTURER}"
  WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoModify" "1"
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"

  ${GetSize} "$INSTDIR" "/M=uninstall.exe /S=0K /G=0" $0 $1 $2
  IntOp $0 $0 + ${ESTIMATEDSIZE}
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${UNINSTKEY}" "EstimatedSize" "$0"

  !if "${HOMEPAGE}" != ""
    WriteRegStr SHCTX "${UNINSTKEY}" "URLInfoAbout" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "URLUpdateInfo" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "HelpLink" "${HOMEPAGE}"
  !endif

  ${If} ${Errors}
    DetailPrint "Failed to register installation metadata."
    SetErrorLevel 2
    Abort
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTINSTALL
    !insertmacro NSIS_HOOK_POSTINSTALL
  !endif

  ; Auto close this page for passive mode
  ${If} $PassiveMode = 1
    SetAutoClose true
  ${EndIf}
SectionEnd

Function un.onInit
  !insertmacro SetContext

  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_UNINIT
  !endif

  !insertmacro MUI_UNGETLANGUAGE

  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}
FunctionEnd

Section Uninstall
  !ifmacrodef NSIS_HOOK_PREUNINSTALL
    !insertmacro NSIS_HOOK_PREUNINSTALL
  !endif

  ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ClearErrors
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --shutdown' $0
    ${If} ${Errors}
    ${OrIf} $0 <> 0
      DetailPrint "Aborting: ${PRODUCTNAME} did not exit within 5 seconds."
      SetErrorLevel 2
      ${IfNot} ${Silent}
        MessageBox MB_ICONEXCLAMATION|MB_OK \
          "${PRODUCTNAME} is currently running and did not close within 5 seconds.$\r$\n$\r$\nPlease close ${PRODUCTNAME} from the notification area and run uninstall again."
      ${EndIf}
      Abort
    ${EndIf}
  ${EndIf}

  ; Delete the app directory and its content from disk (never touches %LOCALAPPDATA%\DropCove)
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"

  ; Delete resources
  {{#each resources}}
    Delete "$INSTDIR\\{{this.[1]}}"
  {{/each}}

  ; Delete external binaries
  {{#each binaries}}
    Delete "$INSTDIR\\{{this}}"
  {{/each}}

  ; Delete app associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
      !insertmacro APP_UNASSOCIATE "{{ext}}" "{{or association.name ext}}"
    {{/each}}
  {{/each}}

  ; Delete deep links
  {{#each deep_link_protocols as |protocol| ~}}
    ReadRegStr $R7 SHCTX "Software\Classes\\{{protocol}}\shell\open\command" ""
    ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
      DeleteRegKey SHCTX "Software\Classes\\{{protocol}}"
    ${EndIf}
  {{/each}}

  ; Delete uninstaller
  Delete "$INSTDIR\uninstall.exe"

  {{#each resources_ancestors}}
  RMDir /REBOOTOK "$INSTDIR\\{{this}}"
  {{/each}}
  RMDir "$INSTDIR"

  ; Remove installer-owned shortcuts if not updating
  ${If} $UpdateMode <> 1
    !insertmacro DeleteAppUserModelId

    ; Remove Start Menu shortcut only if target proves installer ownership
    !insertmacro IsShortcutTarget "${STARTMENUSHORTCUT}" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "${STARTMENUSHORTCUT}"
      Delete "${STARTMENUSHORTCUT}"
    ${EndIf}

    ; Remove Desktop shortcut only if target proves installer ownership
    !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
      Delete "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}
  ${EndIf}

  ; Remove registry information for add/remove programs
  !if "${INSTALLMODE}" == "both"
    DeleteRegKey SHCTX "${UNINSTKEY}"
  !else if "${INSTALLMODE}" == "perMachine"
    DeleteRegKey HKLM "${UNINSTKEY}"
  !else
    DeleteRegKey HKCU "${UNINSTKEY}"
  !endif

  ; Remove installer-location registration and owned Start with Windows registration when not updating
  ${If} $UpdateMode <> 1
    DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty SHCTX "${MANUKEY}"
    DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
    DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty HKCU "${MANUKEY}"

    ; Remove HKCU Run entry only when its complete target and canonical profile arguments match this installation
    ReadRegStr $R8 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
    ${If} $R8 != ""
      StrCpy $R9 "\\?\$LOCALAPPDATA\${PRODUCTNAME}"
      System::Call 'kernel32::CreateFileW(w "$LOCALAPPDATA\${PRODUCTNAME}", i 0, i 7, p 0, i 3, i 0x02000000, p 0) p .r0'
      ${If} $0 P<> -1
      ${AndIf} $0 P<> 0
        System::Call 'kernel32::GetFinalPathNameByHandleW(p r0, w .r1, i ${NSIS_MAX_STRLEN}, i 0) i .r2'
        ${If} $2 > 0
        ${AndIf} $1 != ""
          StrCpy $R9 $1
        ${EndIf}
        System::Call 'kernel32::CloseHandle(p r0)'
      ${EndIf}
      ${If} $R8 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" --autostart --profile $\"$R9$\""
      ${OrIf} $R8 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" --autostart --profile $\"$LOCALAPPDATA\${PRODUCTNAME}$\""
        DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
      ${Else}
        DetailPrint "Preserving non-owned Start with Windows registration: $R8"
        ${IfNot} ${Silent}
          MessageBox MB_ICONEXCLAMATION|MB_OK \
            "An existing Start with Windows entry for ${PRODUCTNAME} points to another target or arguments and was preserved:$\r$\n$R8"
        ${EndIf}
      ${EndIf}
    ${EndIf}
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTUNINSTALL
    !insertmacro NSIS_HOOK_POSTUNINSTALL
  !endif

  ; Auto close if passive mode or updating
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
    SetAutoClose true
  ${EndIf}
SectionEnd

Function CheckVCRedistributableX64
  Push $0
  Push $1
  Push $2
  SetRegView 64
  ClearErrors
  ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ReadRegDWORD $1 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Major"
  StrCpy $2 0
  ${If} ${Errors}
    StrCpy $2 1
  ${EndIf}
  !insertmacro SetContext

  ${If} $2 = 1
  ${OrIf} $0 <> 1
  ${OrIf} $1 <> 14
    SetErrorLevel 2
    ${IfNot} ${Silent}
      MessageBox MB_ICONSTOP|MB_OK \
        "${PRODUCTNAME} requires the Microsoft Visual C++ 2015-2022 Redistributable (x64).$\r$\n$\r$\nPlease download and install it from:$\r$\nhttps://aka.ms/vc14/vc_redist.x64.exe$\r$\n$\r$\nSetup will now exit."
    ${EndIf}
    Pop $2
    Pop $1
    Pop $0
    Abort
  ${EndIf}

  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function ValidateInstallDir
  Push $1
  Push $2
  Push $3
  Push $4
  Push $5
  StrCpy $0 0

  ; 1. Must not be empty or placeholder
  ${If} $INSTDIR == ""
  ${OrIf} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    Goto validate_done
  ${EndIf}

  ; 2. Must have a drive prefix (e.g. C:\...) or UNC prefix (\\...)
  StrLen $1 $INSTDIR
  ${If} $1 < 4
    Goto validate_done
  ${EndIf}
  StrCpy $2 $INSTDIR 2 1
  StrCpy $3 $INSTDIR 2 0
  ${If} $2 != ":\"
  ${AndIf} $3 != "\\"
    Goto validate_done
  ${EndIf}

  ; 3. Check tail after first 2 chars for invalid path characters (: * ? " < > |)
  StrCpy $2 $INSTDIR "" 2
  ${StrLoc} $3 $2 ":" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 "*" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 "?" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 '"' ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 "<" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 ">" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}
  ${StrLoc} $3 $2 "|" ">"
  ${IfThen} $3 != "" ${|} Goto validate_done ${|}

  ; 4. Canonicalize $INSTDIR and %LOCALAPPDATA%\DropCove via GetFullPathNameW (resolves .., ., /, trailing \)
  System::Call 'kernel32::GetFullPathNameW(w "$INSTDIR", i ${NSIS_MAX_STRLEN}, w .r1, p 0) i .r3'
  ${If} $3 <= 0
  ${OrIf} $1 == ""
    Goto validate_done
  ${EndIf}
  strip_instdir_slash:
    StrLen $3 $1
    ${If} $3 > 3
      StrCpy $4 $1 1 -1
      ${If} $4 == "\"
        StrCpy $1 $1 -1
        Goto strip_instdir_slash
      ${EndIf}
    ${EndIf}

  System::Call 'kernel32::GetFullPathNameW(w "$LOCALAPPDATA\${PRODUCTNAME}", i ${NSIS_MAX_STRLEN}, w .r2, p 0) i .r3'
  ${If} $3 <= 0
  ${OrIf} $2 == ""
    StrCpy $2 "$LOCALAPPDATA\${PRODUCTNAME}"
  ${EndIf}
  strip_profile_slash:
    StrLen $3 $2
    ${If} $3 > 3
      StrCpy $4 $2 1 -1
      ${If} $4 == "\"
        StrCpy $2 $2 -1
        Goto strip_profile_slash
      ${EndIf}
    ${EndIf}

  ; Reject if canonical $INSTDIR is Profile Data, a descendant of Profile Data, or an ancestor of Profile Data
  ${If} $1 == $2
    Goto validate_done
  ${EndIf}
  StrLen $3 "$2\"
  StrCpy $4 $1 $3 0
  ${If} $4 == "$2\"
    Goto validate_done
  ${EndIf}
  StrLen $3 "$1\"
  StrCpy $4 $2 $3 0
  ${If} $4 == "$1\"
    Goto validate_done
  ${EndIf}

  StrCpy $5 $1
  ; 5. Walk up from $INSTDIR to find the nearest existing ancestor directory
  find_ancestor_loop:
    ${If} ${FileExists} "$1\*.*"
      Goto found_ancestor
    ${EndIf}
    ${If} ${FileExists} "$1"
      Goto validate_done
    ${EndIf}
    ${GetParent} "$1" $2
    ${If} $2 == ""
    ${OrIf} $2 == "$1"
      Goto validate_done
    ${EndIf}
    StrCpy $1 $2
    Goto find_ancestor_loop
  found_ancestor:
    ; 6. Probe writability at the nearest existing ancestor without leaving directories behind
    ClearErrors
    ${If} $1 == $5
      StrCpy $3 "$1\.dropcove-write-probe.tmp"
      FileOpen $4 "$3" w
      ${If} ${Errors}
      ${OrIf} $4 == ""
        Goto validate_done
      ${EndIf}
      FileClose $4
      Delete "$3"
      ${If} ${Errors}
        Goto validate_done
      ${EndIf}
    ${Else}
      StrCpy $2 "$1\.dropcove-write-probe"
      CreateDirectory "$2"
      ${If} ${Errors}
        Goto validate_done
      ${EndIf}
      StrCpy $3 "$2\probe.tmp"
      FileOpen $4 "$3" w
      ${If} ${Errors}
      ${OrIf} $4 == ""
        RMDir "$2"
        Goto validate_done
      ${EndIf}
      FileClose $4
      Delete "$3"
      RMDir "$2"
    ${EndIf}

    StrCpy $INSTDIR $5
    StrCpy $0 1

  validate_done:
    Pop $5
    Pop $4
    Pop $3
    Pop $2
    Pop $1
FunctionEnd

Function IsShortcutOwnedByInstaller
  !insertmacro IsShortcutTarget "$R9" "$INSTDIR\${MAINBINARYNAME}.exe"
  Pop $0
  ${If} $0 = 1
    Return
  ${EndIf}

  ${If} $PreviousInstallDir != ""
    !insertmacro IsShortcutTarget "$R9" "$PreviousInstallDir\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      Return
    ${EndIf}
  ${EndIf}

  ${If} $OldMainBinaryName != ""
    !insertmacro IsShortcutTarget "$R9" "$INSTDIR\$OldMainBinaryName"
    Pop $0
    ${If} $0 = 1
      Return
    ${EndIf}

    ${If} $PreviousInstallDir != ""
      !insertmacro IsShortcutTarget "$R9" "$PreviousInstallDir\$OldMainBinaryName"
      Pop $0
      ${If} $0 = 1
        Return
      ${EndIf}
    ${EndIf}
  ${EndIf}

  StrCpy $0 0
FunctionEnd

Function VerifyStartMenuShortcutOwnership
  ${If} ${FileExists} "${STARTMENUSHORTCUT}"
    StrCpy $R9 "${STARTMENUSHORTCUT}"
    Call IsShortcutOwnedByInstaller
    ${If} $0 <> 1
      DetailPrint "Conflicting Start Menu shortcut detected: ${STARTMENUSHORTCUT}"
      SetErrorLevel 2
      ${IfNot} ${Silent}
        MessageBox MB_ICONSTOP|MB_OK \
          "Setup cannot continue because an existing Start Menu shortcut at:$\r$\n${STARTMENUSHORTCUT}$\r$\npoints to an unrelated target and will not be overwritten."
      ${EndIf}
      Abort
    ${EndIf}
  ${EndIf}
FunctionEnd

Function RestorePreviousInstallLocation
  ${If} $PreviousInstallDir != ""
    StrCpy $INSTDIR $PreviousInstallDir
  ${EndIf}
FunctionEnd

Function SkipIfPassive
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
FunctionEnd

Function un.SkipIfPassive
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
FunctionEnd

Function CreateOrUpdateStartMenuShortcut
  ${If} $NoShortcutMode = 1
    Return
  ${EndIf}

  !if "${STARTMENUFOLDER}" != ""
    CreateDirectory "$SMPROGRAMS\${STARTMENUFOLDER}"
  !endif
  CreateShortcut "${STARTMENUSHORTCUT}" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "${STARTMENUSHORTCUT}"
FunctionEnd

Function CreateOrUpdateDesktopShortcut
  ${If} $NoShortcutMode = 1
    Return
  ${EndIf}

  ${If} ${FileExists} "$DESKTOP\${PRODUCTNAME}.lnk"
    StrCpy $R9 "$DESKTOP\${PRODUCTNAME}.lnk"
    Call IsShortcutOwnedByInstaller
    ${If} $0 = 1
      CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
      !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
    ${Else}
      DetailPrint "Warning: Preserving conflicting Desktop shortcut at $DESKTOP\${PRODUCTNAME}.lnk"
      ${If} $DesktopShortcutRequested = 1
      ${AndIfNot} ${Silent}
        MessageBox MB_ICONEXCLAMATION|MB_OK \
          "An existing Desktop shortcut at:$\r$\n$DESKTOP\${PRODUCTNAME}.lnk$\r$\npoints to an unrelated target and was preserved without changes."
      ${EndIf}
    ${EndIf}
    Return
  ${EndIf}

  ${If} $DesktopShortcutRequested = 1
  ${AndIf} $PassiveMode <> 1
  ${AndIfNot} ${Silent}
    CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
  ${EndIf}
FunctionEnd
