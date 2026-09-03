param([string[]]$Urls=@('https://www.speedtest.net/','https://developers.cloudflare.com/workers/','https://en.wikipedia.org/wiki/Servo_(software)','https://github.com/servo/servo','https://duckduckgo.com/?q=amni+browse','https://news.ycombinator.com/'),[string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_sites",[int]$Wait=20,[int]$PerTab=6)
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class PS {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  public static IntPtr Best(uint pid){ IntPtr best=IntPtr.Zero; long area=0; EnumWindows(delegate(IntPtr h, IntPtr l){ uint p; GetWindowThreadProcessId(h,out p); if(p!=pid||!IsWindowVisible(h)) return true; R r; GetWindowRect(h,out r); long a=(long)(r.Rt-r.L)*(r.B-r.T); if(a>area){area=a;best=h;} return true; }, IntPtr.Zero); return best; }
  public struct R { public int L,T,Rt,B; }
}
"@
[PS]::SetProcessDPIAware() | Out-Null
$gst='C:\gstreamer\1.0\msvc_x86_64'
$env:PATH="$env:PATH;$gst\bin"
$env:GSTREAMER_1_0_ROOT_MSVC_X86_64="$gst\"
$env:RUST_LOG="info"; $env:AMNI_TRACE_INPUT="1"; if($env:PROBE_JS){ $env:AMNI_PROBE_JS=$env:PROBE_JS }
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"
if (Test-Path $sess) { Copy-Item $sess "$sess.sitesbak" -Force }
$i=0
$objs=@($Urls|ForEach-Object{ $o=@{url=$_;title="";is_active=($i -eq 0);history=@($_);history_index=0;engine=""}; $i++; $o })
$j=@{tabs=$objs;window_width=1400.0;window_height=900.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path (Split-Path $Exe)) -PassThru -RedirectStandardOutput "$OutDir\app.log" -RedirectStandardError "$OutDir\app.err.log"
Start-Sleep -Seconds $Wait
$p.Refresh()
if($p.HasExited){ Write-Host "EXITED $($p.ExitCode)"; exit 1 }
$h=[PS]::Best([uint32]$p.Id); if($h -eq [IntPtr]::Zero){ $h=$p.MainWindowHandle }
function Shot($n){ [PS]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 400
  $r=New-Object PS+R; [PS]::GetWindowRect($h,[ref]$r)|Out-Null
  $b=New-Object Drawing.Bitmap(($r.Rt-$r.L),($r.B-$r.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($r.L,$r.T,0,0,$b.Size); $b.Save("$OutDir\$n.png",[Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $b.Dispose()
  Write-Host "SHOT $n $(($r.Rt-$r.L))x$(($r.B-$r.T))" }
function Front(){ for($t=0;$t -lt 5;$t++){ [PS]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 250; if([PS]::GetForegroundWindow() -eq $h){ return }; [System.Windows.Forms.SendKeys]::SendWait("%"); [PS]::SetForegroundWindow($h)|Out-Null; Start-Sleep -Milliseconds 250 }; Write-Host "FRONT FAILED" }
$r0=New-Object PS+R; [PS]::GetWindowRect($h,[ref]$r0)|Out-Null; Write-Host "WINDOW $($r0.L),$($r0.T) $($r0.Rt-$r0.L)x$($r0.B-$r0.T)"
for($k=1;$k -le $Urls.Count;$k++){ Front; [System.Windows.Forms.SendKeys]::SendWait("^$k"); Start-Sleep -Seconds $PerTab; Shot ("tab{0}" -f $k); $sb=New-Object System.Text.StringBuilder 16000; [PS]::GetWindowTextW($h,$sb,16000)|Out-Null; $t=$sb.ToString(); if($t -like 'AMNIPROBE*'){ [IO.File]::WriteAllText("$OutDir/tab$k.probe.json",$t.Substring(10)); Write-Host "PROBE tab$k $($t.Length) chars" } else { Write-Host "TITLE tab$k $($t.Substring(0,[Math]::Min(80,$t.Length)))" } }
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
if (Test-Path "$sess.sitesbak") { Copy-Item "$sess.sitesbak" $sess -Force }
Write-Host "OUT $OutDir"
