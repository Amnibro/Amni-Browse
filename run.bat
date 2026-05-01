@echo off
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
echo [Amni-Browse] Building full Servo engine ^(first build ~30 min, incremental after^)...
cargo build --release --no-default-features --features servo-real
if errorlevel 1 (
    echo [Amni-Browse] Build FAILED - not launching stale binary.
    pause
    exit /b 1
)
echo [Amni-Browse] Staging ANGLE DLLs next to exe...
for /d %%d in (target\release\build\mozangle-*) do (
    if exist "%%d\out\libEGL.dll" copy /y "%%d\out\libEGL.dll" target\release\ >nul
    if exist "%%d\out\libGLESv2.dll" copy /y "%%d\out\libGLESv2.dll" target\release\ >nul
)
start "" target\release\amni-browse.exe
