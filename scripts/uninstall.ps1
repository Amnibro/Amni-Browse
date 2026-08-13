$ErrorActionPreference="Stop"
$dest=Join-Path $env:LOCALAPPDATA "AmniBrowse"
Get-Process amni-browse -ErrorAction SilentlyContinue | Stop-Process -Force
foreach($p in @((Join-Path ([Environment]::GetFolderPath("Desktop")) "Amni Browse.lnk"),(Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Amni Browse.lnk"))){
  if(Test-Path $p){ Remove-Item $p -Force }
}
foreach($k in @("HKCU:\Software\Classes\AmniBrowseHTML","HKCU:\Software\Clients\StartMenuInternet\AmniBrowse")){
  if(Test-Path $k){ Remove-Item $k -Recurse -Force }
}
Remove-ItemProperty -Path "HKCU:\Software\RegisteredApplications" -Name "AmniBrowse" -ErrorAction SilentlyContinue
if(Test-Path $dest){ Remove-Item $dest -Recurse -Force }
Write-Host "Amni Browse removed."
