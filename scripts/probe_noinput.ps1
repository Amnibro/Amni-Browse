param([string]$Url='https://www.speedtest.net/',[string]$Exe="$PSScriptRoot\..\target\release\amni-browse.exe",[int]$Wait=16)
Add-Type @"
using System;using System.Runtime.InteropServices;
public class PN { [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n); }
"@
$cfg="$env:APPDATA\amni-browse"; $sess="$cfg\session.json"; if (Test-Path $sess) { Copy-Item $sess "$sess.nibak" -Force }
$j=@{tabs=@(@{url=$Url;title="";is_active=$true;history=@($Url);history_index=0;engine=""});window_width=1200.0;window_height=800.0;window_x=2200.0;window_y=100.0;saved_at=(Get-Date).ToUniversalTime().ToString("o");was_clean_exit=$true} | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText($sess,$j,(New-Object Text.UTF8Encoding($false)))
$env:RUST_LOG='info'
$p=Start-Process -FilePath $Exe -WorkingDirectory (Split-Path $Exe) -PassThru -RedirectStandardOutput "$env:TEMP\ni.log" -RedirectStandardError "$env:TEMP\ni.err.log"
Start-Sleep -Seconds $Wait
$p.Refresh(); $sb=New-Object System.Text.StringBuilder 8000; [PN]::GetWindowTextW($p.MainWindowHandle,$sb,8000)|Out-Null; Write-Host "TITLE $($sb.ToString())"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 1
if (Test-Path "$sess.nibak") { Copy-Item "$sess.nibak" $sess -Force }
