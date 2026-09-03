@echo off
setlocal
cd /d "%~dp0.."
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" (
  echo FATAL: GStreamer runtime missing
  exit /b 1
)
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PKG_CONFIG_PATH=%GST_ROOT%\lib\pkgconfig"
set "PATH=%PATH%;%GST_ROOT%\bin;C:\ProgramData\chocolatey\bin"
echo GST=%GST_ROOT%
echo PKG_CONFIG_PATH=%PKG_CONFIG_PATH%
if not exist "%GST_ROOT%\lib\pkgconfig\glib-2.0.pc" (
  echo FATAL: glib-2.0.pc missing under GST
  exit /b 1
)
cargo build --release --features servo-real
if errorlevel 1 exit /b 1
echo BUILD_OK
exit /b 0
