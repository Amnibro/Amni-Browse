@echo off
setlocal
cd /d "%~dp0.."
call "%~dp0vsenv.cmd" || exit /b 1
set "CARGO_TARGET_DIR=%CD%\target"
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin;C:\ProgramData\chocolatey\bin"
echo Building Amni-Browse %CD% target=%CARGO_TARGET_DIR% gstreamer=%GST_ROOT%
cargo build --release --no-default-features --features servo-real
set "EC=%ERRORLEVEL%"
if not "%EC%"=="0" exit /b %EC%
for /d %%d in ("%CARGO_TARGET_DIR%\release\build\mozangle-*") do (
  if exist "%%d\out\libEGL.dll" copy /y "%%d\out\libEGL.dll" "%CARGO_TARGET_DIR%\release\" >nul
  if exist "%%d\out\libGLESv2.dll" copy /y "%%d\out\libGLESv2.dll" "%CARGO_TARGET_DIR%\release\" >nul
)
if not exist "%CARGO_TARGET_DIR%\release\assets\chrome" mkdir "%CARGO_TARGET_DIR%\release\assets\chrome"
copy /y "assets\chrome\toolbar.html" "%CARGO_TARGET_DIR%\release\assets\chrome\" >nul
findstr /c:"Real Servo (libservo)" "%CARGO_TARGET_DIR%\release\amni-browse.exe" >nul || (
  echo FATAL: exe is not a servo-real build
  exit /b 1
)
echo OK: %CARGO_TARGET_DIR%\release\amni-browse.exe
exit /b 0
