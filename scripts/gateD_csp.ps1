param([int]$Port = 8919, [int]$Secs = 45)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\amni-browse.exe'
if (-not (Test-Path $exe)) { Write-Host 'GATE_D skip=no exe'; exit 1 }
$fix = Join-Path $root 'test\gateD_csp'
$origin = "http://127.0.0.1:$Port"
$out = Join-Path $env:TEMP 'amni_gateD'
Remove-Item -Recurse -Force $out -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $out | Out-Null
$env:PYTHONUNBUFFERED = '1'
$srvLog = Join-Path $out 'srv.log'
$srvErr = Join-Path $out 'srv.err.log'
function Count-FaviconGets {
  param($paths)
  $n = 0
  foreach ($p in $paths) {
    if (Test-Path $p) {
      $n += @(Get-Content $p -ErrorAction SilentlyContinue | Select-String -Pattern 'GET /favicon\.ico').Count
    }
  }
  return $n
}
$srv = Start-Process -FilePath 'python' -ArgumentList @('-u',(Join-Path $fix 'serve.py'),"$Port") -PassThru -WindowStyle Hidden -RedirectStandardOutput $srvLog -RedirectStandardError $srvErr
Start-Sleep -Seconds 2
try {
  $probe = Invoke-WebRequest -Uri "$origin/favicon.ico" -UseBasicParsing -TimeoutSec 5
  Write-Host ('GATE_D fixture bytes=' + $probe.RawContentLength)
  Start-Sleep -Milliseconds 400
  $hitsAfterProbe = Count-FaviconGets @($srvLog, $srvErr)
  Write-Host ('GATE_D serverFaviconGET after probe=' + $hitsAfterProbe)
  $cfg = Join-Path $env:APPDATA 'amni-browse'
  New-Item -ItemType Directory -Force -Path $cfg | Out-Null
  $sess = Join-Path $cfg 'session.json'
  if (Test-Path $sess) { Copy-Item $sess ($sess + '.gateDbak') -Force }
  $tab = @{ url = "$origin/"; title = ''; is_active = $true; history = @("$origin/"); history_index = 0; engine = '' }
  $j = @{ tabs = @($tab); window_width = 1200.0; window_height = 800.0; saved_at = (Get-Date).ToUniversalTime().ToString('o'); was_clean_exit = $true } | ConvertTo-Json -Depth 6
  [IO.File]::WriteAllText($sess, $j, (New-Object Text.UTF8Encoding($false)))
  $log = Join-Path $out 'gateD.log'
  $elog = Join-Path $out 'gateD.err.log'
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
  $mine | ForEach-Object { Write-Host ('GATE_D log ' + $_) }
  $started = @($mine | Select-String -Pattern '\bstarted\b').Count
  $cached = @($mine | Select-String -Pattern 'embedder-(cache|fetch) ok').Count
  $bad = @($mine | Select-String -Pattern 'decode-fail|badenc|poll-timeout|miss|empty|big|err').Count
  $embnet = @($mine | Select-String -Pattern 'embedder-net').Count
  $pending = @($fav | Select-String -Pattern '\bpending\b').Count
  $dom = @($fav | Select-String -Pattern 'appendChild').Count
  $hitsTotal = Count-FaviconGets @($srvLog, $srvErr)
  $hits = [Math]::Max(0, $hitsTotal - $hitsAfterProbe)
  Write-Host ("GATE_D started=$started embedder-cache-ok=$cached bad=$bad pending=$pending appendChild=$dom serverFaviconGET=$hits serverFaviconGET_total=$hitsTotal embedder-net=$embnet")
  $ok = ($started -ge 1) -and ($cached -ge 1) -and ($pending -eq 0) -and ($dom -eq 0) -and ($hits -ge 1) -and ($embnet -ge 2)
  if ($ok) { Write-Host 'GATE_D PASS' } else { Write-Host 'GATE_D FAIL' }
  if (Test-Path ($sess + '.gateDbak')) { Copy-Item ($sess + '.gateDbak') $sess -Force }
  if ($ok) { exit 0 } else { exit 2 }
} finally {
  if ($srv -and -not $srv.HasExited) { $srv.Kill() }
}
