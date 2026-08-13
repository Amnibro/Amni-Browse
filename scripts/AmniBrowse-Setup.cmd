@echo off
setlocal
cd /d "%~dp0"
echo Amni Browse Setup
echo Pulls the latest zip from amni-scient.com or GitHub and installs to %%LOCALAPPDATA%%\AmniBrowse
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*
if errorlevel 1 (
  echo Install failed.
  pause
  exit /b 1
)
