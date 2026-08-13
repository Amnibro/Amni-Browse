!include "MUI2.nsh"
Name "Amni Browse"
OutFile "..\AmniBrowse-Setup.exe"
InstallDir "$LOCALAPPDATA\AmniBrowse"
RequestExecutionLevel user
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
Section "Install"
  SetOutPath "$INSTDIR"
  nsExec::ExecToLog 'powershell -NoProfile -ExecutionPolicy Bypass -File "$EXEDIR\install.ps1"'
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd
Section "Uninstall"
  nsExec::ExecToLog 'powershell -NoProfile -ExecutionPolicy Bypass -File "$INSTDIR\uninstall.ps1"'
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
