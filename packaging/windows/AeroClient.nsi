; SPDX-License-Identifier: GPL-3.0-only
Unicode true
!include "MUI2.nsh"
!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef APP_SOURCE
  !error "APP_SOURCE must point to the staged portable directory"
!endif

Name "Aero Client"
OutFile "..\..\target\AeroClient-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Aero Client"
RequestExecutionLevel user
SetCompressor /SOLID lzma
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Aero Client"
VIAddVersionKey "FileDescription" "Free and open-source Minecraft launcher"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Aero Client contributors"

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\aero-launcher.exe"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Aero Client (required)" Core
  SectionIn RO
  SetOutPath "$INSTDIR"
  File /r "${APP_SOURCE}\*"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Aero Client"
  CreateShortcut "$SMPROGRAMS\Aero Client\Aero Client.lnk" "$INSTDIR\aero-launcher.exe"
  CreateShortcut "$SMPROGRAMS\Aero Client\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Aero Client" "DisplayName" "Aero Client"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Aero Client" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Aero Client" "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
SectionEnd

Section /o "Desktop shortcut" Desktop
  CreateShortcut "$DESKTOP\Aero Client.lnk" "$INSTDIR\aero-launcher.exe"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\Aero Client.lnk"
  RMDir /r "$SMPROGRAMS\Aero Client"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Aero Client"
  RMDir /r "$INSTDIR"
SectionEnd
