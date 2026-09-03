param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_click")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class PC {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  public struct R { public int L,T,Rt,B; }
}
"@
[PC]::SetProcessDPIAware() | Out-Null
$env:RUST_LOG='info'
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"; if (Test-Path $sess) { Copy-Item $sess "$sess.clickbak" -Force }
$Urls=@('https://www.speedtest.net/','https://github.com/login','https://news.ycombinator.com/','https://en.wikipedia.org/wiki/Servo_(software)')
$i=0; $objs=@($Urls|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
$j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path (Split-Path $Exe)) -PassThru -RedirectStandardOutput "$OutDir\app.log" -RedirectStandardError "$OutDir\app.err.log"
Start-Sleep -Seconds 14
$p.Refresh(); $h=$p.MainWindowHandle
$r=New-Object PC+R; [PC]::GetWindowRect($h,[ref]$r)|Out-Null; Write-Host "WIN $($r.L),$($r.T) $($r.Rt-$r.L)x$($r.B-$r.T)"
function Shot($n){ [PC]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 300; $rr=New-Object PC+R; [PC]::GetWindowRect($h,[ref]$rr)|Out-Null; $b=New-Object Drawing.Bitmap(($rr.Rt-$rr.L),($rr.B-$rr.T)); $g=[Drawing.Graphics]::FromImage($b); $g.CopyFromScreen($rr.L,$rr.T,0,0,$b.Size); $b.Save("$OutDir\$n.png"); $g.Dispose(); $b.Dispose(); $sb=New-Object System.Text.StringBuilder 512; [PC]::GetWindowTextW($h,$sb,512)|Out-Null; Write-Host "SHOT $n :: $($sb.ToString())" }
function Click($x,$y){ [PC]::SetForegroundWindow($h)|Out-Null; [PC]::SetCursorPos($x,$y)|Out-Null; Start-Sleep -Milliseconds 150; [PC]::mouse_event(2,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 60; [PC]::mouse_event(4,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 400 }
Shot "c0_start"
Click ($r.L+420) ($r.T+37); Start-Sleep -Seconds 5; Shot "c1_tab2_github"
Click ($r.L+680) ($r.T+37); Start-Sleep -Seconds 5; Shot "c2_tab3_hn"
Click ($r.L+420) ($r.T+37); Start-Sleep -Seconds 3
Click ($r.L+884) ($r.T+712); Start-Sleep -Seconds 6; Shot "c3_google_button"
Click ($r.L+1042) ($r.T+37); Start-Sleep -Seconds 4; Shot "c5_plus_newtab"
[PC]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 300; [System.Windows.Forms.SendKeys]::SendWait("^1"); Start-Sleep -Seconds 3; Shot "c4_ctrl1"
function RClick($x,$y){ [PC]::SetForegroundWindow($h)|Out-Null; [PC]::SetCursorPos($x,$y)|Out-Null; Start-Sleep -Milliseconds 150; [PC]::mouse_event(8,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 60; [PC]::mouse_event(16,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 500 }
RClick ($r.L+650) ($r.T+37); Shot "c7_tab_menu"; Click ($r.L+690) ($r.T+60); Start-Sleep -Seconds 2; Shot "c8_pinned"
RClick ($r.L+420) ($r.T+37); Start-Sleep -Milliseconds 400; Click ($r.L+460) ($r.T+92); Start-Sleep -Seconds 3; Shot "c9_group_prompt"
[PC]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 300; [System.Windows.Forms.SendKeys]::SendWait("^1"); Start-Sleep -Seconds 2; [System.Windows.Forms.SendKeys]::SendWait("^f"); Start-Sleep -Seconds 1; [System.Windows.Forms.SendKeys]::SendWait("Spectrum{ENTER}"); Start-Sleep -Seconds 2; Shot "c10_find"
function Drag($x1,$y1,$x2,$y2){ [PC]::SetCursorPos($x1,$y1)|Out-Null; Start-Sleep -Milliseconds 200; [PC]::mouse_event(2,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 150; for($i=1;$i -le 10;$i++){ [PC]::SetCursorPos([int]($x1+($x2-$x1)*$i/10),[int]($y1+($y2-$y1)*$i/10))|Out-Null; Start-Sleep -Milliseconds 40 }; Start-Sleep -Milliseconds 150; [PC]::mouse_event(4,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 500 }
Drag ($r.Rt-2) ([int](($r.T+$r.B)/2)) ($r.Rt+150) ([int](($r.T+$r.B)/2)); $r2=New-Object PC+R; [PC]::GetWindowRect($h,[ref]$r2)|Out-Null; Write-Host "RESIZE $($r.Rt-$r.L) -> $($r2.Rt-$r2.L)"
Drag ($r.L+1300) ($r.T+37) ($r.L+1500) ($r.T+240); $r3=New-Object PC+R; [PC]::GetWindowRect($h,[ref]$r3)|Out-Null; Write-Host "MOVE $($r.L),$($r.T) -> $($r3.L),$($r3.T)"
Shot "c6_after_resize_move"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 2
if (Test-Path "$sess.clickbak") { Copy-Item "$sess.clickbak" $sess -Force }
Write-Host "OUT $OutDir"
