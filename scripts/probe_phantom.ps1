param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_phantom")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class PH {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
}
"@
[PH]::SetProcessDPIAware() | Out-Null
$gst='C:\gstreamer\1.0\msvc_x86_64'
$env:PATH="$env:PATH;$gst\bin"; $env:GSTREAMER_1_0_ROOT_MSVC_X86_64="$gst\"; $env:RUST_LOG="info"; $env:AMNI_TRACE_INPUT="1"
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"
if (Test-Path $sess) { Copy-Item $sess "$sess.phbak" -Force }
$Urls=@('https://en.wikipedia.org/wiki/Servo_(software)','https://news.ycombinator.com/','https://developers.cloudflare.com/workers/')
$i=0
$objs=@($Urls|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
$j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
[PH]::SetCursorPos(3300,1300) | Out-Null
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path (Split-Path $Exe)) -PassThru -RedirectStandardOutput "$OutDir\app.log" -RedirectStandardError "$OutDir\app.err.log"
Start-Sleep -Seconds 18
$p.Refresh(); if($p.HasExited){ Write-Host "EXITED"; exit 1 }
$h=$p.MainWindowHandle
$r=New-Object PH+R; [PH]::GetWindowRect($h,[ref]$r)|Out-Null
Write-Host "WIN $($r.L),$($r.T) $($r.Rt-$r.L)x$($r.B-$r.T)"
function Shot($n){ $rr=New-Object PH+R; [PH]::GetWindowRect($h,[ref]$rr)|Out-Null
  $b=New-Object Drawing.Bitmap(($rr.Rt-$rr.L),($rr.B-$rr.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($rr.L,$rr.T,0,0,$b.Size); $b.Save("$OutDir\$n.png",[Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $b.Dispose(); Write-Host "SHOT $n" }
function Front(){ for($t=0;$t -lt 5;$t++){ [PH]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 250; if([PH]::GetForegroundWindow() -eq $h){ return } ; [System.Windows.Forms.SendKeys]::SendWait("%"); [PH]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 250 }; Write-Host "FRONT FAILED" }
function Key($k){ Front; [System.Windows.Forms.SendKeys]::SendWait($k); Write-Host "KEY $k $(Get-Date -Format HH:mm:ss.f)" }
function ClickContent(){ [PH]::SetCursorPos($r.L+1500,$r.B-40)|Out-Null; Start-Sleep -Milliseconds 150; [PH]::mouse_event(2,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 60; [PH]::mouse_event(4,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 300; [PH]::SetCursorPos(3300,1300)|Out-Null; Write-Host "CLICKCONTENT $(Get-Date -Format HH:mm:ss.f)" }
Write-Host "=== PHASE A cursor off-window ==="
Front; ClickContent; Start-Sleep -Seconds 2; Shot "A0_after_content_click"
Key "^2"; Start-Sleep -Seconds 5; Shot "A_hn_off"
Key "^3"; Start-Sleep -Seconds 5; Shot "A_cf_off"
Key "^1"; Start-Sleep -Seconds 5; Shot "A_wiki_off"
Write-Host "=== PHASE B cursor over content (HN 2nd headline area) ==="
[PH]::SetCursorPos($r.L+400,$r.T+230) | Out-Null
Start-Sleep -Seconds 2
Key "^2"; Start-Sleep -Seconds 6; Shot "B_hn_cursor"
Key "^3"; Start-Sleep -Seconds 6; Shot "B_cf_cursor"
Key "^1"; Start-Sleep -Seconds 6; Shot "B_wiki_cursor"
Write-Host "=== PHASE C cursor over content, switch via mouseless wait only ==="
Start-Sleep -Seconds 8; Shot "C_wiki_idle"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
if (Test-Path "$sess.phbak") { Copy-Item "$sess.phbak" $sess -Force }
Write-Host "OUT $OutDir"
