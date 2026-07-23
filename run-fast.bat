@echo off
rem Launch-only: starts the already-built Amni-Browse without recompiling Servo.
rem Use run.bat for the first build; use this for instant launches afterward.
cd /d "%~dp0"
set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin" (
    echo [Amni-Browse] GStreamer not found at "%GST_ROOT%".
    echo Run ^(elevated^): scripts\install_build_deps.ps1
    pause
    exit /b 1
)
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin;C:\ProgramData\chocolatey\bin"
if not exist "target\release\amni-browse.exe" (
    echo [Amni-Browse] No prebuilt exe found - run run.bat once to build it.
    pause
    exit /b 1
)
for /d %%d in (target\release\build\mozangle-*) do (
    if exist "%%d\out\libEGL.dll" copy /y "%%d\out\libEGL.dll" target\release\ >nul
    if exist "%%d\out\libGLESv2.dll" copy /y "%%d\out\libGLESv2.dll" target\release\ >nul
)
echo [Amni-Browse] Launching prebuilt amni-browse.exe (no rebuild)...
start "" target\release\amni-browse.exe
