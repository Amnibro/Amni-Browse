@echo off
setlocal
cd /d "%~dp0.."
set "GST_ROOT=C:\gstreamer\1.0\msvc_x86_64"
if not exist "%GST_ROOT%\bin\gstreamer-1.0-0.dll" set "GST_ROOT=C:\Program Files\gstreamer\1.0\msvc_x86_64"
set "GSTREAMER_1_0_ROOT_MSVC_X86_64=%GST_ROOT%\"
set "PATH=%PATH%;%GST_ROOT%\bin"
set "RUST_LOG=info"
start "" /b cmd /c "target\release\amni-browse.exe %* > test\run_out.log 2> test\run_err.log"
exit /b 0
