param([string]$Zip,[string]$OutDir="$env:TEMP\amni_gates")
$ErrorActionPreference='Stop'
Remove-Item -Recurse -Force $OutDir -ErrorAction SilentlyContinue
$ext="$OutDir\extract"
New-Item -ItemType Directory -Force -Path $ext | Out-Null
python -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" $Zip $ext
$exe="$ext\amni-browse.exe"
if (!(Test-Path $exe)) { throw "no exe in cold extract" }
Add-Type -AssemblyName System.Windows.Forms,System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  public struct R { public int L,T,Rt,B; }
}
"@
function Shot($proc,$name){
  $h=$proc.MainWindowHandle
  if ($h -eq [IntPtr]::Zero) { $proc.Refresh(); $h=$proc.MainWindowHandle }
  [W]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 400
  $r=New-Object W+R; [W]::GetWindowRect($h,[ref]$r) | Out-Null
  $w=$r.Rt-$r.L; $ht=$r.B-$r.T
  if ($w -le 0 -or $ht -le 0) { throw "zero-size window for $name" }
  $bmp=New-Object System.Drawing.Bitmap($w,$ht)
  $g=[System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.L,$r.T,0,0,$bmp.Size)
  $bmp.Save("$OutDir\$name.png"); $g.Dispose(); $bmp.Dispose()
  Write-Host "SHOT $name ${w}x${ht}"
}
$p=Start-Process -FilePath $exe -WorkingDirectory $ext -PassThru
Start-Sleep -Seconds 12
$p.Refresh()
if ($p.HasExited) { throw "exe exited code $($p.ExitCode)" }
Shot $p "gate1_first_paint"
[W]::SetForegroundWindow($p.MainWindowHandle) | Out-Null; Start-Sleep -Milliseconds 300
[System.Windows.Forms.SendKeys]::SendWait("^t"); Start-Sleep -Seconds 3
[System.Windows.Forms.SendKeys]::SendWait("^t"); Start-Sleep -Seconds 3
Shot $p "gate3_three_tabs"
[System.Windows.Forms.SendKeys]::SendWait("^{TAB}"); Start-Sleep -Seconds 2
Shot $p "gate3_switch_back"
Stop-Process -Id $p.Id -Force
Start-Sleep -Seconds 2
$tj="$env:APPDATA\amni-browse\theme.json"
Copy-Item $tj "$tj.gatebak" -Force
$j=Get-Content $tj -Raw | ConvertFrom-Json
$orig=$j.active_theme_id
$j.active_theme_id="amni-light"
$j | ConvertTo-Json -Depth 8 | Set-Content $tj
$p2=Start-Process -FilePath $exe -WorkingDirectory $ext -PassThru
Start-Sleep -Seconds 12
$p2.Refresh()
if ($p2.HasExited) { throw "relaunch exited code $($p2.ExitCode)" }
Shot $p2 "gate2_theme_light_both_surfaces"
Stop-Process -Id $p2.Id -Force
Copy-Item "$tj.gatebak" $tj -Force
Write-Host "theme restored to $orig"
Write-Host "DONE -> $OutDir"
