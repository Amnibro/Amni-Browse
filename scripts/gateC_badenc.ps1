param([int]$Port = 8918, [int]$Secs = 45)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\amni-browse.exe'
if (-not (Test-Path $exe)) { Write-Host 'GATE_C skip=no exe'; exit 1 }
$fix = Join-Path $root 'test\gateC_badenc'
$mk = Join-Path $fix 'make_fixture.py'
if (Test-Path $mk) { python $mk | Out-Null }
$origin = "http://127.0.0.1:$Port"
$out = Join-Path $env:TEMP 'amni_gateC'
Remove-Item -Recurse -Force $out -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $out | Out-Null
$env:PYTHONUNBUFFERED = '1'
$srvLog = Join-Path $out 'srv.log'
$srvErr = Join-Path $out 'srv.err.log'
$srv = Start-Process -FilePath 'python' -ArgumentList @('-u','-m','http.server',"$Port",'--bind','127.0.0.1','--directory',$fix) -PassThru -WindowStyle Hidden -RedirectStandardOutput $srvLog -RedirectStandardError $srvErr
Start-Sleep -Seconds 2
try {
  $cfg = Join-Path $env:APPDATA 'amni-browse'
  New-Item -ItemType Directory -Force -Path $cfg | Out-Null
  $sess = Join-Path $cfg 'session.json'
  if (Test-Path $sess) { Copy-Item $sess ($sess + '.gateCbak') -Force }
  $tab = @{ url = "$origin/"; title = ''; is_active = $true; history = @("$origin/"); history_index = 0; engine = '' }
  $j = @{ tabs = @($tab); window_width = 1200.0; window_height = 800.0; saved_at = (Get-Date).ToUniversalTime().ToString('o'); was_clean_exit = $true } | ConvertTo-Json -Depth 6
  [IO.File]::WriteAllText($sess, $j, (New-Object Text.UTF8Encoding($false)))
  $log = Join-Path $out 'gateC.log'
  $elog = Join-Path $out 'gateC.err.log'
  $env:RUST_LOG = 'info'
  $p = Start-Process -FilePath $exe -PassThru -RedirectStandardOutput $log -RedirectStandardError $elog
  Start-Sleep -Seconds $Secs
  if (-not $p.HasExited) { $p.Kill() }
  Start-Sleep -Seconds 2
  if (-not $srv.HasExited) { $srv.Kill() }
  Start-Sleep -Seconds 1
  $all = Get-Content $log, $elog -ErrorAction SilentlyContinue
  $fav = @($all | Select-String -Pattern 'favicon')
  $mine = @($fav | Select-String -Pattern ([regex]::Escape("127.0.0.1:$Port")))
  $mine | ForEach-Object { Write-Host ('GATE_C log ' + $_) }
  $badenc = @($mine | Select-String -Pattern '\bbadenc\b|decode-fail').Count
  $cached = @($mine | Select-String -Pattern 'embedder-(cache|fetch) ok').Count
  $pending = @($fav | Select-String -Pattern '\bpending\b').Count
  $dom = @($fav | Select-String -Pattern 'appendChild').Count
  Write-Host ("GATE_C reject=$badenc embedder-cache-ok=$cached pending=$pending appendChild=$dom")
  $ok = ($badenc -ge 1) -and ($cached -eq 0) -and ($pending -eq 0) -and ($dom -eq 0)
  if ($ok) { Write-Host 'GATE_C PASS' } else { Write-Host 'GATE_C FAIL' }
  if (Test-Path ($sess + '.gateCbak')) { Copy-Item ($sess + '.gateCbak') $sess -Force }
  if ($ok) { exit 0 } else { exit 2 }
} finally {
  if ($srv -and -not $srv.HasExited) { $srv.Kill() }
}
