param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_lap15")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class P15 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref PT p);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
  public struct PT { public int X,Y; }
  public const uint LD=0x0002, LU=0x0004;
}
"@
[P15]::SetProcessDPIAware() | Out-Null
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"
New-Item -ItemType Directory -Force -Path $cfg | Out-Null
if (Test-Path $sess) { Copy-Item $sess "$sess.lap15bak" -Force }
function Seed($urls){
  $i=0
  $objs=@($urls|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
  $j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
  [IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
}
function Shot($h,$n,$dir){
  $r=New-Object P15+R; [P15]::GetWindowRect($h,[ref]$r)|Out-Null
  $b=New-Object Drawing.Bitmap(($r.Rt-$r.L),($r.B-$r.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($r.L,$r.T,0,0,$b.Size); $b.Save("$dir\$n.png",[Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $b.Dispose(); Write-Host "SHOT $n $(($r.Rt-$r.L))x$(($r.B-$r.T))"
}
function Launch($tag,$wait){
  $log="$OutDir\$tag.log"; $elog="$OutDir\$tag.err.log"
  $env:RUST_LOG="info"
  $p=Start-Process -FilePath $Exe -PassThru -RedirectStandardOutput $log -RedirectStandardError $elog
  Start-Sleep -Seconds $wait
  $p.Refresh()
  if($p.HasExited){ throw "$tag exited code $($p.ExitCode)" }
  if($p.MainWindowHandle -eq [IntPtr]::Zero){ $p.Kill(); throw "$tag no window" }
  [P15]::SetForegroundWindow($p.MainWindowHandle)|Out-Null
  Start-Sleep -Seconds 4
  ,$p
}
Write-Host "=== GATE A: favicons through servo net ==="
Seed @('https://github.com/','https://en.wikipedia.org/wiki/Servo_(software)','https://example.com/')
$pa=Launch "gateA" 26
Start-Sleep -Seconds 10
Shot $pa.MainWindowHandle "A_tabstrip_favicons" $OutDir
$pa.Kill(); Start-Sleep -Seconds 2
$al=Get-Content "$OutDir\gateA.log","$OutDir\gateA.err.log" -ErrorAction SilentlyContinue
$fav=$al | Select-String -Pattern 'favicon'
Write-Host "--- favicon lines ---"; $fav | Select-Object -Last 25 | ForEach-Object { Write-Host $_ }
$cache=@($fav | Select-String -Pattern 'embedder-cache').Count
$dom=@($fav | Select-String -Pattern 'appendChild').Count
Write-Host "GATE_A embedder-cache=$cache appendChild=$dom"
Write-Host "=== GATE B: three same-title tabs, slow pointer drag ==="
Seed @('about:blank','about:blank','about:blank')
$pb=Launch "gateB" 22
$h=$pb.MainWindowHandle
$dpi=[P15]::GetDpiForWindow($h); if($dpi -eq 0){$dpi=96}
$s=$dpi/96.0
Write-Host "dpi=$dpi scale=$s"
$o=New-Object P15+PT; [P15]::ClientToScreen($h,[ref]$o)|Out-Null
function TabX($i){ [int]($o.X + $s*(8 + $i*122 + 60)) }
$ty=[int]($o.Y + $s*20)
Shot $h "B_before_drag" $OutDir
$from=TabX 0; $to=TabX 2
Write-Host "DRAG $from,$ty -> $to,$ty"
[P15]::SetCursorPos($from,$ty)|Out-Null; Start-Sleep -Milliseconds 400
[P15]::mouse_event([P15]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 350
$steps=24
for($i=1;$i -le $steps;$i++){
  $x=[int]($from + ($to-$from)*$i/$steps)
  [P15]::SetCursorPos($x,$ty)|Out-Null
  Start-Sleep -Milliseconds 70
}
Start-Sleep -Milliseconds 400
Shot $h "B_mid_drag" $OutDir
[P15]::mouse_event([P15]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 1200
Shot $h "B_after_drop" $OutDir
Start-Sleep -Milliseconds 600
$pb.Kill(); Start-Sleep -Seconds 2
$bl=Get-Content "$OutDir\gateB.log","$OutDir\gateB.err.log" -ErrorAction SilentlyContinue
Write-Host "--- tab move lines ---"
$bl | Select-String -Pattern 'tab_move|move_tab|applyMove|reorder' | Select-Object -Last 20 | ForEach-Object { Write-Host $_ }
Write-Host "--- panic/error lines ---"
$bl | Select-String -Pattern 'panic|ERROR' | Select-Object -Last 15 | ForEach-Object { Write-Host $_ }
if (Test-Path "$sess.lap15bak") { Copy-Item "$sess.lap15bak" $sess -Force }
Write-Host "DONE -> $OutDir"
