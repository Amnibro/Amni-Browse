@echo off
setlocal
cd /d "%~dp0.."
set "OUT=%~dp0..\qc_bundle_out.txt"
> "%OUT%" echo === QC BUNDLE %DATE% %TIME% ===

echo [1/9] check.cmd (cargo check first; fail the bundle here)
echo [1/9] check.cmd>> "%OUT%"
call "%~dp0check.cmd" >> "%OUT%" 2>&1
set CERR=%errorlevel%
echo CHECK_EXIT=%CERR% >> "%OUT%"
if not "%CERR%"=="0" (
  echo === FAIL cargo check; remaining steps skipped ===>> "%OUT%"
  type "%OUT%"
  exit /b %CERR%
)

echo [2/9] build_release.cmd>> "%OUT%"
call "%~dp0build_release.cmd" >> "%OUT%" 2>&1
set BERR=%errorlevel%
echo BUILD_EXIT=%BERR% >> "%OUT%"

echo [3/9] test.cmd>> "%OUT%"
call "%~dp0test.cmd" >> "%OUT%" 2>&1
set TERR=%errorlevel%
echo TEST_EXIT=%TERR% >> "%OUT%"

echo [4/9] check_toolbar.js>> "%OUT%"
node "%~dp0check_toolbar.js" >> "%OUT%" 2>&1
set JERR=%errorlevel%
echo TOOLBAR_EXIT=%JERR% >> "%OUT%"

echo [5/9] gateA favicon probe>> "%OUT%"
set PERR=1
if not exist "target\release\amni-browse.exe" echo PROBE_SKIP=no exe>> "%OUT%"
if exist "target\release\amni-browse.exe" call :gatea
echo PROBE_EXIT=%PERR% >> "%OUT%"
goto :after_gates
:gatea
powershell.exe -NoProfile -Command "$ErrorActionPreference='Stop';$Exe='%~dp0..\target\release\amni-browse.exe';$OutDir=$env:TEMP+'\amni_gateA_logonly';Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue;New-Item -ItemType Directory -Force -Path $OutDir|Out-Null;$cfg=$env:APPDATA+'\amni-browse';$sess=$cfg+'\session.json';New-Item -ItemType Directory -Force -Path $cfg|Out-Null;if(Test-Path $sess){Copy-Item $sess ($sess+'.gateAbak') -Force};$urls=@('https://github.com/','https://en.wikipedia.org/wiki/Servo_(software)','https://example.com/');$i=0;$objs=@($urls|ForEach-Object{$o=@{url=$_;title='';is_active=($i -eq 0);history=@($_);history_index=0;engine=''};$i++;$o});$j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString('o');was_clean_exit=$true}|ConvertTo-Json -Depth 6;[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)));$log=$OutDir+'\gateA.log';$elog=$OutDir+'\gateA.err.log';$env:RUST_LOG='info';$p=Start-Process -FilePath $Exe -PassThru -RedirectStandardOutput $log -RedirectStandardError $elog;Start-Sleep -Seconds 30;if(-not $p.HasExited){$p.Kill()};Start-Sleep -Seconds 2;$al=Get-Content $log,$elog -ErrorAction SilentlyContinue;$fav=$al|Select-String -Pattern 'favicon';$cache=@($fav|Select-String -Pattern 'embedder-(cache|net)').Count;$dom=@($fav|Select-String -Pattern 'appendChild').Count;$dup=@($fav|Select-String -Pattern 'servo-net https://example.com has').Count;Write-Host ('GATE_A embedder-cache='+$cache+' appendChild='+$dom+' probe-dup='+$dup);$fav|Select-String -Pattern 'embedder-(cache|net)'|ForEach-Object{Write-Host $_};if($dup -gt 1){Write-Host 'GATE_A FAIL probe-dup';exit 3};if(Test-Path ($sess+'.gateAbak')){Copy-Item ($sess+'.gateAbak') $sess -Force}" >> "%OUT%" 2>&1
set PERR=%errorlevel%
exit /b 0
:gateb
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0gateB_nolic.ps1" >> "%OUT%" 2>&1
set GERR=%errorlevel%
exit /b 0
:after_gates

echo [6/9] gateB no-link-icon local origin>> "%OUT%"
set GERR=1
if not exist "target\release\amni-browse.exe" echo GATEB_SKIP=no exe>> "%OUT%"
if exist "target\release\amni-browse.exe" call :gateb
echo GATEB_EXIT=%GERR% >> "%OUT%"

if exist "target\release\amni-browse.exe" (
  for %%F in ("target\release\amni-browse.exe") do echo EXE_EXISTS=YES EXE_TIME=%%~tF EXE_SIZE=%%~zF>> "%OUT%"
) else (
  echo EXE_EXISTS=NO>> "%OUT%"
)

echo [7/9] gateC badenc probe>> "%OUT%"
set GCERR=1
if not exist "target\release\amni-browse.exe" echo GATEC_SKIP=no exe>> "%OUT%"
if exist "target\release\amni-browse.exe" call :gatec
echo GATEC_EXIT=%GCERR% >> "%OUT%"

echo [8/9] gateD csp connect-src none origin>> "%OUT%"
set GDERR=1
if not exist "target\release\amni-browse.exe" echo GATED_SKIP=no exe>> "%OUT%"
if exist "target\release\amni-browse.exe" call :gated
echo GATED_EXIT=%GDERR% >> "%OUT%"

echo [9/9] gateE https-to-http mixed redirect>> "%OUT%"
set GEERR=1
if not exist "target\release\amni-browse.exe" echo GATEE_SKIP=no exe>> "%OUT%"
if exist "target\release\amni-browse.exe" call :gatee
echo GATEE_EXIT=%GEERR% >> "%OUT%"

set FAIL=0
if not "%CERR%"=="0" set FAIL=1
if not "%BERR%"=="0" set FAIL=1
if not "%TERR%"=="0" set FAIL=1
if not "%JERR%"=="0" set FAIL=1
if not "%PERR%"=="0" set FAIL=1
if not "%GERR%"=="0" set FAIL=1
if not "%GCERR%"=="0" set FAIL=1
if not "%GDERR%"=="0" set FAIL=1
if not "%GEERR%"=="0" set FAIL=1
echo BUNDLE_FAIL=%FAIL% >> "%OUT%"
echo === DONE ===>> "%OUT%"
type "%OUT%"
exit /b %FAIL%
:gatec
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0gateC_badenc.ps1" >> "%OUT%" 2>&1
set GCERR=%errorlevel%
exit /b 0
:gated
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0gateD_csp.ps1" >> "%OUT%" 2>&1
set GDERR=%errorlevel%
exit /b 0
:gatee
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0gateE_mixed.ps1" >> "%OUT%" 2>&1
set GEERR=%errorlevel%
exit /b 0
