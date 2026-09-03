param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_phantom2")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class PH2 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
}
"@
[PH2]::SetProcessDPIAware() | Out-Null
$gst='C:\gstreamer\1.0\msvc_x86_64'
$env:PATH="$env:PATH;$gst\bin"; $env:GSTREAMER_1_0_ROOT_MSVC_X86_64="$gst\"; $env:RUST_LOG='info'
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"
if (Test-Path $sess) { Copy-Item $sess "$sess.phbak" -Force }
$Urls=@('https://news.ycombinator.com/','https://example.com/')
$i=0
$objs=@($Urls|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
$j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
[PH2]::SetCursorPos(3300,1300) | Out-Null
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path (Split-Path $Exe)) -PassThru -RedirectStandardOutput "$OutDir\app.log" -RedirectStandardError "$OutDir\app.err.log"
Start-Sleep -Seconds 14
$p.Refresh(); if($p.HasExited){ Write-Host "EXITED"; exit 1 }
$h=$p.MainWindowHandle
$r=New-Object PH2+R; [PH2]::GetWindowRect($h,[ref]$r)|Out-Null
function Shot($n){ $rr=New-Object PH2+R; [PH2]::GetWindowRect($h,[ref]$rr)|Out-Null
  $b=New-Object Drawing.Bitmap(($rr.Rt-$rr.L),($rr.B-$rr.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($rr.L,$rr.T,0,0,$b.Size); $b.Save("$OutDir\$n.png",[Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $b.Dispose(); Write-Host "SHOT $n" }
function Key($k){ [PH2]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 300; [System.Windows.Forms.SendKeys]::SendWait($k); Write-Host "KEY $k $(Get-Date -Format HH:mm:ss.f)" }
function Click($x,$y){ [PH2]::SetCursorPos($x,$y)|Out-Null; Start-Sleep -Milliseconds 120; [PH2]::mouse_event(2,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 60; [PH2]::mouse_event(4,0,0,0,[IntPtr]::Zero); Write-Host "CLICK $x,$y $(Get-Date -Format HH:mm:ss.f)"; Start-Sleep -Milliseconds 300 }
Shot "00_start"
Key "^{TAB}"; Start-Sleep -Seconds 3; Key "^{TAB}"; Start-Sleep -Seconds 5; Shot "E1_ctrltab"
Click ($r.L+400) ($r.T+30); Start-Sleep -Seconds 3; Click ($r.L+130) ($r.T+30); [PH2]::SetCursorPos(3300,1300)|Out-Null; Start-Sleep -Seconds 5; Shot "E2_chipclick"
Key "^2"; Start-Sleep -Seconds 3; Key "^1"; Start-Sleep -Seconds 5; Shot "E3_ctrldigit"
Key "2"; Start-Sleep -Seconds 5; Shot "E4_digit_alone"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
if (Test-Path "$sess.phbak") { Copy-Item "$sess.phbak" $sess -Force }
Write-Host "OUT $OutDir"
