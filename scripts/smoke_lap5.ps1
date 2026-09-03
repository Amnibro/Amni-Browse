param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_smoke_lap5")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref P p);
  public struct P { public int X,Y; }
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
  public const uint LD=0x0002, LU=0x0004;
}
"@
[W]::SetProcessDPIAware() | Out-Null
$page=@'
<!DOCTYPE html><html><head><meta charset="utf-8"><title>Amni smoke</title>
<style>body{font:16px system-ui;margin:0;background:#141821;color:#eee}
#band{height:120px;display:flex;align-items:center;gap:18px;padding:0 20px;background:#1d2330}
input[type=color]{width:120px;height:56px}</style></head>
<body><div id="band"><input type="color" value="#C89B4E"><input type="file"><button id="b">alert me</button></div>
<h1 style="padding:20px">Amni-Browse smoke page</h1>
<p style="padding:0 20px">Color input, file input, and a scripted alert.</p>
<script>document.getElementById('b').onclick=function(){alert('smoke alert from the page')};
setTimeout(function(){alert('scripted alert on load')},2500);</script>
</body></html>
'@
$pagePath="$OutDir\smoke.html"
[IO.File]::WriteAllText($pagePath,$page,(New-Object Text.UTF8Encoding($false)))
$pageUrl="file:///" + ($pagePath -replace '\\','/')
$cfg="$env:APPDATA\amni-browse"
$sess="$cfg\session.json"
if (Test-Path $sess) { Copy-Item $sess "$sess.lap5bak" -Force }
$W=1400.0; $H=900.0
$state=@{tabs=@(
  @{url=$pageUrl;title="smoke";is_active=$true;history=@($pageUrl);history_index=0;engine="servo"},
  @{url="https://example.com/";title="example";is_active=$false;history=@("https://example.com/");history_index=0;engine="servo"}
);window_width=$W;window_height=$H;saved_at=(Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffffffZ");was_clean_exit=$true}
[IO.File]::WriteAllText($sess,($state | ConvertTo-Json -Depth 6),(New-Object Text.UTF8Encoding($false)))
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path $Exe) -PassThru -RedirectStandardError "$OutDir\app.err.log" -RedirectStandardOutput "$OutDir\app.out.log"
$script:proc=$p
function Rect(){ $h=$p.MainWindowHandle; if($h -eq [IntPtr]::Zero){$p.Refresh();$h=$p.MainWindowHandle}; $r=New-Object W+R; [W]::GetWindowRect($h,[ref]$r)|Out-Null; return $r }
function Front(){ $p.Refresh(); [W]::SetForegroundWindow($p.MainWindowHandle)|Out-Null; Start-Sleep -Milliseconds 350 }
function Shot($name){ Front; $r=Rect; $w=$r.Rt-$r.L; $ht=$r.B-$r.T
  if($w -le 0 -or $ht -le 0){ Write-Host "SKIP $name (zero window)"; return }
  $bmp=New-Object System.Drawing.Bitmap($w,$ht); $g=[System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L,$r.T,0,0,$bmp.Size); $bmp.Save("$OutDir\$name.png"); $g.Dispose(); $bmp.Dispose()
  Write-Host "SHOT $name ${w}x${ht}" }
function Click($x,$y){ [W]::SetCursorPos($x,$y)|Out-Null; Start-Sleep -Milliseconds 120; [W]::mouse_event([W]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 60; [W]::mouse_event([W]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 400 }
function Drag($x1,$y1,$x2,$y2){ [W]::SetCursorPos($x1,$y1)|Out-Null; Start-Sleep -Milliseconds 150; [W]::mouse_event([W]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 200
  for($i=1;$i -le 14;$i++){ [W]::SetCursorPos([int]($x1+($x2-$x1)*$i/14),[int]($y1+($y2-$y1)*$i/14))|Out-Null; Start-Sleep -Milliseconds 45 }
  Start-Sleep -Milliseconds 250 }
function DragEnd(){ [W]::mouse_event([W]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 600 }
Start-Sleep -Seconds 14
$p.Refresh()
if ($p.HasExited) { Write-Host "EXE EXITED code $($p.ExitCode)"; if(Test-Path "$sess.lap5bak"){Copy-Item "$sess.lap5bak" $sess -Force}; exit 1 }
$r=Rect
$h=$p.MainWindowHandle
$cr=New-Object W+R; [W]::GetClientRect($h,[ref]$cr)|Out-Null
$org=New-Object W+P; [W]::ClientToScreen($h,[ref]$org)|Out-Null
$cw=$cr.Rt-$cr.L; $chh=$cr.B-$cr.T
$scale=[math]::Round($cw/$W,3)
Write-Host "WINDOW $($r.L),$($r.T) $(($r.Rt-$r.L))x$(($r.B-$r.T)) CLIENT $($org.X),$($org.Y) ${cw}x${chh} scale~$scale"
$L=$org.X; $T=$org.Y
$chrome=[int](84*$scale)
Shot "01_first_paint"
Start-Sleep -Seconds 4
Shot "02_alert_on_load"
Front; [System.Windows.Forms.SendKeys]::SendWait("{ESC}"); Start-Sleep -Seconds 1
Shot "03_after_dismiss"
Click ([int]($L+80*$scale)) ([int]($T+$chrome+60*$scale))
Shot "04_color_picker"
Front; [System.Windows.Forms.SendKeys]::SendWait("{ESC}"); Start-Sleep -Milliseconds 700
Front; [System.Windows.Forms.SendKeys]::SendWait("^t"); Start-Sleep -Seconds 3
Front; [System.Windows.Forms.SendKeys]::SendWait("^t"); Start-Sleep -Seconds 3
Shot "05_four_tabs"
$ty=[int]($T+20*$scale)
Drag ([int]($L+70*$scale)) $ty ([int]($L+600*$scale)) $ty
Shot "06_mid_drag"
DragEnd
Shot "07_after_drop"
Front; [System.Windows.Forms.SendKeys]::SendWait("^u"); Start-Sleep -Seconds 4
Shot "08_view_source"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
if (Test-Path "$sess.lap5bak") { Copy-Item "$sess.lap5bak" $sess -Force }
Write-Host "OUT $OutDir"
