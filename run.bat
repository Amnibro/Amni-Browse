@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" (
    echo [Amni-Browse] GStreamer runtime DLLs not found at "%GST_ROOT%".
    echo Run ^(elevated^): scripts\install_build_deps.ps1
    pause
    exit /b 1
)
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin;C:\ProgramData\chocolatey\bin"
set "NEED_BUILD=1"
if exist "target\release\amni-browse.exe" (
    for /f %%s in ('powershell -NoProfile -Command "$exe=(Get-Item 'target\release\amni-browse.exe').LastWriteTime; $src=@(Get-ChildItem -Recurse -File src,build.rs,Cargo.toml,Cargo.lock -ErrorAction SilentlyContinue | Measure-Object -Property LastWriteTime -Maximum).Maximum; if ($src -and $src -gt $exe) { 'stale' } else { 'fresh' }"') do set "BUILD_STATE=%%s"
    if "!BUILD_STATE!"=="fresh" set "NEED_BUILD=0"
)
if "%NEED_BUILD%"=="0" (
    echo [Amni-Browse] Prebuilt exe is up to date - skipping rebuild ^(delete target\release\amni-browse.exe to force one^).
) else (
    echo [Amni-Browse] Building full Servo engine ^(first build ~30 min, incremental after^)...
    cargo build --release --features servo-real
    if errorlevel 1 (
        echo [Amni-Browse] Build FAILED - not launching stale binary.
        pause
        exit /b 1
    )
)
echo [Amni-Browse] Staging ANGLE DLLs next to exe...
for /d %%d in (target\release\build\mozangle-*) do (
    if exist "%%d\out\libEGL.dll" copy /y "%%d\out\libEGL.dll" target\release\ >nul
    if exist "%%d\out\libGLESv2.dll" copy /y "%%d\out\libGLESv2.dll" target\release\ >nul
)
if not exist "target\release\assets\chrome" mkdir "target\release\assets\chrome"
copy /y "assets\chrome\toolbar.html" "target\release\assets\chrome\" >nul
start "" target\release\amni-browse.exe
