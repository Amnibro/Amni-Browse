param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_lap9")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class P9 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
  public const uint LD=0x0002, LU=0x0004;
}
"@
[P9]::SetProcessDPIAware() | Out-Null
$cfg="$env:APPDATA\amni-browse"
$sess="$cfg\session.json"
if (Test-Path $sess) { Copy-Item $sess "$sess.lap9bak" -Force }
New-Item -ItemType Directory -Force -Path $cfg | Out-Null
$tabs=@('https://example.com/','https://github.com/','https://en.wikipedia.org/wiki/Servo_(software)')
$i=0
$tabObjs=@($tabs|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
$json=@{tabs=$tabObjs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$json,(New-Object Text.UTF8Encoding($false)))
$log="$OutDir\run.log"; $elog="$OutDir\run.err.log"
$env:RUST_LOG="info"
$p=Start-Process -FilePath $Exe -PassThru -RedirectStandardOutput $log -RedirectStandardError $elog
Start-Sleep -Seconds 22
$h=$p.MainWindowHandle
if($h -eq [IntPtr]::Zero){ Write-Host "NO WINDOW"; $p.Kill(); exit 1 }
[P9]::SetForegroundWindow($h)|Out-Null
Start-Sleep -Seconds 8
function Shot($n){ $r=New-Object P9+R; [P9]::GetWindowRect($h,[ref]$r)|Out-Null
  $b=New-Object Drawing.Bitmap(($r.Rt-$r.L),($r.B-$r.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($r.L,$r.T,0,0,$b.Size); $b.Save("$OutDir\$n.png",[Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $b.Dispose(); Write-Host "SHOT $n $(($r.Rt-$r.L))x$(($r.B-$r.T))" }
Shot "01_tabstrip_favicons"
$r0=New-Object P9+R; [P9]::GetWindowRect($h,[ref]$r0)|Out-Null
$ty=$r0.T+3
$tx=[int](($r0.L+$r0.Rt)/2)
Write-Host "NORTH DRAG from $tx,$ty  rect T=$($r0.T) B=$($r0.B)"
[P9]::SetCursorPos($tx,$ty)|Out-Null; Start-Sleep -Milliseconds 300
[P9]::mouse_event([P9]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 250
for($i=1;$i -le 12;$i++){ [P9]::SetCursorPos($tx,[int]($ty+$i*10))|Out-Null; Start-Sleep -Milliseconds 45 }
[P9]::mouse_event([P9]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 900
$r1=New-Object P9+R; [P9]::GetWindowRect($h,[ref]$r1)|Out-Null
Write-Host "AFTER  rect T=$($r1.T) B=$($r1.B)  dT=$($r1.T-$r0.T) dB=$($r1.B-$r0.B)"
if(($r1.T-$r0.T) -ge 60 -and [math]::Abs($r1.B-$r0.B) -le 6){ Write-Host "NORTH_RESIZE=PASS" }
elseif([math]::Abs($r1.T-$r0.T) -ge 60 -and [math]::Abs($r1.B-$r0.B) -ge 60){ Write-Host "NORTH_RESIZE=FAIL (window MOVED, not resized)" }
else { Write-Host "NORTH_RESIZE=FAIL (no geometry change)" }
Shot "02_after_north_drag"
Start-Sleep -Milliseconds 400
$p.Kill()
Start-Sleep -Milliseconds 800
if (Test-Path "$sess.lap9bak") { Copy-Item "$sess.lap9bak" $sess -Force }
Write-Host "--- favicon log ---"
Get-Content $log,$elog -ErrorAction SilentlyContinue | Select-String -Pattern 'favicon' | Select-Object -Last 20
Write-Host "OUT $OutDir"
