@echo off
cd /d "%~dp0"
echo [Amni-Browse] Building (incremental, fast if unchanged)...
cargo build --release
if errorlevel 1 (
    echo [Amni-Browse] Build FAILED — not launching stale binary.
    pause
    exit /b 1
)
start "" target\release\amni-browse.exe
