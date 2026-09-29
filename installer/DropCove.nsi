!ifndef PUBLISH_DIR
  !error "PUBLISH_DIR must point to the self-contained publish directory."
!endif

!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif

!include "LogicLib.nsh"
!include "WinMessages.nsh"

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
    SendMessage $0 ${WM_CLOSE} 0 0 /TIMEOUT=5000
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

  SendMessage $0 ${WM_CLOSE} 0 0 /TIMEOUT=5000
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
