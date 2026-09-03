param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_hit")
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
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
  public const uint LD=0x0002, LU=0x0004;
}
"@
[PH]::SetProcessDPIAware() | Out-Null
$page=@'
<!DOCTYPE html><html><head><meta charset="utf-8"><title>hit probe</title>
<style>body{margin:0;background:#141821;color:#eee;font:20px system-ui}
#b{position:absolute;left:40px;top:40px;width:260px;height:90px;font:20px system-ui}
#c{position:absolute;left:40px;top:180px;width:260px;height:90px}</style></head>
<body><button id="b">hit me</button><input id="c" type="color" value="#C89B4E">
<script>document.getElementById('b').onclick=function(){document.title='CONTENT-CLICK-OK'}</script>
</body></html>
'@
$pagePath="$OutDir\hit.html"
[IO.File]::WriteAllText($pagePath,$page,(New-Object Text.UTF8Encoding($false)))
$pageUrl="file:///" + ($pagePath -replace '\\','/')
$log="$OutDir\run.log"; $elog="$OutDir\run.err.log"
$env:RUST_LOG="info"
$p=Start-Process -FilePath $Exe -ArgumentList $pageUrl -PassThru -RedirectStandardOutput $log -RedirectStandardError $elog
Start-Sleep -Seconds 14
$h=$p.MainWindowHandle
if($h -eq [IntPtr]::Zero){ Write-Host "NO WINDOW"; $p.Kill(); exit 1 }
[PH]::SetForegroundWindow($h)|Out-Null
Start-Sleep -Seconds 3
$r=New-Object PH+R; [PH]::GetWindowRect($h,[ref]$r)|Out-Null
$sf=1.25
$chrome=[int](84*$sf)
function Click($cx,$cy){ [PH]::SetCursorPos($cx,$cy)|Out-Null; Start-Sleep -Milliseconds 250
  [PH]::mouse_event([PH]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 90
  [PH]::mouse_event([PH]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 1200 }
function Shot($n){ $q=New-Object PH+R; [PH]::GetWindowRect($h,[ref]$q)|Out-Null
  $b=New-Object Drawing.Bitmap(($q.Rt-$q.L),($q.B-$q.T)); $g=[Drawing.Graphics]::FromImage($b)
  $g.CopyFromScreen($q.L,$q.T,0,0,$b.Size); $b.Save("$OutDir\$n.png",[Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $b.Dispose(); Write-Host "SHOT $n" }
$bx=[int]($r.L+170*$sf); $by=[int]($r.T+$chrome+85*$sf)
Write-Host "BUTTON CLICK $bx,$by (window $($r.L),$($r.T))"
Click $bx $by
Shot "01_after_button"
$cx=[int]($r.L+170*$sf); $cy=[int]($r.T+$chrome+225*$sf)
Write-Host "COLOR CLICK $cx,$cy"
Click $cx $cy
Shot "02_after_color"
Start-Sleep -Milliseconds 400
$p.Kill()
Start-Sleep -Milliseconds 800
Write-Host "--- app log ---"
Get-Content $log,$elog -ErrorAction SilentlyContinue | Select-String -Pattern 'amni_browse::platform::servo_real' | Select-Object -Last 25
Write-Host "OUT $OutDir"
