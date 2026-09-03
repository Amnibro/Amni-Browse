@echo off
setlocal
cd /d "%~dp0.."
call "%~dp0vsenv.cmd" || exit /b 1
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\lib\pkgconfig" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin"
cargo check --release --features servo-real --message-format short %*
exit /b %errorlevel%
