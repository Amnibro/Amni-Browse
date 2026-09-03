@echo off
cd /d C:\Users\antho\Documents\ai\Amni-Browse
set RUST_LOG=info
set RUST_BACKTRACE=1
"target\debug\amni-browse.exe" https://example.com/ > "%TEMP%\amni-persist.log" 2>&1
