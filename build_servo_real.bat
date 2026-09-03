@echo off
cd /d "%~dp0"
call "%~dp0scripts\vsenv.cmd" || exit /b 1
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin;C:\ProgramData\chocolatey\bin"
cargo build --release --no-default-features --features servo-real > build_servo_real.log 2>&1
echo EXITCODE=%errorlevel% >> build_servo_real.log
