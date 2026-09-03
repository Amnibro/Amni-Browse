param([string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[string]$OutDir="$env:TEMP\amni_probe_color")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class P2 {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint dx,uint dy,uint d,IntPtr e);
  public struct R { public int L,T,Rt,B; }
  public const uint LD=0x0002, LU=0x0004;
}
"@
[P2]::SetProcessDPIAware() | Out-Null
$page=@'
<!DOCTYPE html><html><head><meta charset="utf-8"><title>color probe</title>
<style>body{margin:0;background:#141821;color:#eee;font:16px system-ui}
#c{position:absolute;left:40px;top:40px;width:200px;height:80px}</style></head>
<body><input id="c" type="color" value="#C89B4E"></body></html>
'@
$pagePath="$OutDir\probe.html"
[IO.File]::WriteAllText($pagePath,$page,(New-Object Text.UTF8Encoding($false)))
$pageUrl="file:///" + ($pagePath -replace '\\','/')
$log="$OutDir\run.log"
$env:RUST_LOG="info"
$p=Start-Process -FilePath $Exe -ArgumentList $pageUrl -PassThru -RedirectStandardOutput $log -RedirectStandardError "$OutDir\run.err.log"
Start-Sleep -Seconds 12
$h=$p.MainWindowHandle
if($h -eq [IntPtr]::Zero){ Write-Host "NO WINDOW"; $p.Kill(); exit 1 }
[P2]::SetForegroundWindow($h)|Out-Null
$r=New-Object P2+R
[P2]::GetWindowRect($h,[ref]$r)|Out-Null
$scale=[double]([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width) / [double]([System.Windows.Forms.SystemInformation]::VirtualScreen.Width)
$sf=1.25
$chrome=[int](84*$sf)
$cx=[int]($r.L + 140*$sf)
$cy=[int]($r.T + $chrome + 80*$sf)
Write-Host "WINDOW $($r.L),$($r.T) CLICK $cx,$cy"
[P2]::SetCursorPos($cx,$cy)|Out-Null; Start-Sleep -Milliseconds 300
[P2]::mouse_event([P2]::LD,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 80
[P2]::mouse_event([P2]::LU,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 1500
$bmp=New-Object Drawing.Bitmap(($r.Rt-$r.L),($r.B-$r.T))
$g=[Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.L,$r.T,0,0,$bmp.Size)
$bmp.Save("$OutDir\color_click.png",[Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Start-Sleep -Milliseconds 400
$p.Kill()
Start-Sleep -Milliseconds 800
Write-Host "--- log lines mentioning color/embedder ---"
Get-Content $log,"$OutDir\run.err.log" -ErrorAction SilentlyContinue | Select-String -Pattern 'color|embedder|picker' | Select-Object -Last 20
Write-Host "OUT $OutDir"
