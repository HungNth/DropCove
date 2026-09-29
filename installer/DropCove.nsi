!ifndef PUBLISH_DIR
  !error "PUBLISH_DIR must point to the self-contained publish directory."
!endif

!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif

!include "LogicLib.nsh"
!include "WinMessages.nsh"
!define WM_DROPCOVE_EXIT 0x8002
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\DropCove"

Name "DropCove"
OutFile "..\artifacts\DropCove-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\DropCove"
InstallDirRegKey HKCU "Software\DropCove" "InstallLocation"
RequestExecutionLevel user
SetCompressor /SOLID lzma
Unicode True

Section "Install"
  SetOutPath "$INSTDIR"
  File /r "${PUBLISH_DIR}\*"
  SetRegView 64
  WriteRegStr HKCU "Software\DropCove" "InstallLocation" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "DropCove"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "DropCove"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\DropCove.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
  CreateDirectory "$SMPROGRAMS\DropCove"
  CreateShortcut "$SMPROGRAMS\DropCove\DropCove.lnk" "$INSTDIR\DropCove.exe"
SectionEnd

Section "Uninstall"
  SetRegView 64
  Delete "$SMPROGRAMS\DropCove\DropCove.lnk"
  RMDir "$SMPROGRAMS\DropCove"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir /r "$INSTDIR"
  DeleteRegKey HKCU "Software\DropCove"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "DropCove"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd

Function .onInit
  SetRegView 64
  ReadRegDWORD $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ${If} $0 != 1
    MessageBox MB_ICONSTOP "DropCove requires the x64 Microsoft Visual C++ Redistributable. Install it, then run this setup again."
    Abort
  ${EndIf}

  ; Check if DropCove is running and prompt/wait for it to close
  FindWindow $0 "" "DropCove"
  ${If} $0 != 0
    SendMessage $0 ${WM_DROPCOVE_EXIT} 0 0 /TIMEOUT=5000
    StrCpy $1 0
    ${DoWhile} $1 < 25
      Sleep 200
      FindWindow $0 "" "DropCove"
      ${If} $0 == 0
        Sleep 500
        ${Break}
      ${EndIf}
      IntOp $1 $1 + 1
    ${Loop}
    FindWindow $0 "" "DropCove"
    ${If} $0 != 0
      MessageBox MB_ICONSTOP "Please close DropCove before installing an update."
      Abort
    ${EndIf}
  ${EndIf}
FunctionEnd

Function un.onInit
  SetRegView 64
  FindWindow $0 "" "DropCove"
  ${If} $0 == 0
    Return
  ${EndIf}

  SendMessage $0 ${WM_DROPCOVE_EXIT} 0 0 /TIMEOUT=5000
  StrCpy $1 0
  ${DoWhile} $1 < 25
    Sleep 200
    FindWindow $0 "" "DropCove"
    ${If} $0 == 0
      Sleep 500
      Return
    ${EndIf}
    IntOp $1 $1 + 1
  ${Loop}

  MessageBox MB_ICONSTOP "Close DropCove before uninstalling."
  Abort
FunctionEnd
