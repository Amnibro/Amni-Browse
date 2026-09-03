param([switch]$Local,[string]$Feed="https://amni-scient.com/browse/latest.json",[string]$GitHub="https://api.github.com/repos/Amnibro/Amni-Browse/releases/latest")
$ErrorActionPreference="Stop"
$dest=Join-Path $env:LOCALAPPDATA "AmniBrowse"
New-Item -ItemType Directory -Force -Path $dest|Out-Null
function Get-Remote {
  try {
    $j=Invoke-RestMethod -Uri $Feed -Headers @{ "User-Agent"="AmniBrowse-Setup" }
    if($j.version -and $j.url){ return @{ Version=[string]$j.version; Url=[string]$j.url; Source="site" } }
  } catch {}
  $g=Invoke-RestMethod -Uri $GitHub -Headers @{ "User-Agent"="AmniBrowse-Setup" }
  $tag=([string]$g.tag_name).TrimStart("v")
  $asset=$g.assets|Where-Object { $_.name -like "amni-browse-*-win64.zip" }|Select-Object -First 1
  if(-not $asset){ throw "No win64 zip on GitHub release $tag" }
  return @{ Version=$tag; Url=[string]$asset.browser_download_url; Source="github" }
}
if($Local){
  $root=Split-Path -Parent $PSScriptRoot
  $exe=Join-Path $root "target\release\amni-browse.exe"
  if(-not (Test-Path $exe)){ throw "Local build missing: $exe" }
  Copy-Item $exe (Join-Path $dest "amni-browse.exe") -Force
  $assets=Join-Path $root "assets"
  if(Test-Path $assets){ Copy-Item $assets (Join-Path $dest "assets") -Recurse -Force }
  Write-Host "Installed local build to $dest"
} else {
  $rel=Get-Remote
  Write-Host "Downloading Amni Browse v$($rel.Version) from $($rel.Source)..."
  $zip=Join-Path $dest "setup.zip"
  Invoke-WebRequest -Uri $rel.Url -OutFile $zip -UseBasicParsing
  $stage=Join-Path $dest "stage"
  if(Test-Path $stage){ Remove-Item $stage -Recurse -Force }
  Expand-Archive -LiteralPath $zip -DestinationPath $stage -Force
  Get-ChildItem $stage -Recurse -File | ForEach-Object {
    $relPath=$_.FullName.Substring($stage.Length).TrimStart("\","/")
    $out=Join-Path $dest $relPath
    New-Item -ItemType Directory -Force -Path (Split-Path $out)|Out-Null
    Copy-Item $_.FullName $out -Force
  }
  Remove-Item $zip -Force -ErrorAction SilentlyContinue
}
$launch=Join-Path $dest "amni-browse.exe"
if(-not (Test-Path $launch)){
  $found=Get-ChildItem $dest -Recurse -Filter "amni-browse.exe"|Select-Object -First 1
  if($found){ $launch=$found.FullName }
}
if(-not (Test-Path $launch)){ throw "amni-browse.exe not found after install" }
$w=New-Object -ComObject WScript.Shell
$desk=[Environment]::GetFolderPath("Desktop")
$start=Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
foreach($p in @((Join-Path $desk "Amni Browse.lnk"),(Join-Path $start "Amni Browse.lnk"))){
  $s=$w.CreateShortcut($p)
  $s.TargetPath=$launch
  $s.WorkingDirectory=(Split-Path $launch)
  $s.Description="Amni Browse"
  $s.IconLocation="$launch,0"
  $s.Save()
}
$exeEsc=$launch.Replace("\","\\")
$cmd="`"$launch`" `"%1`""
$pairs=@(
  @("HKCU:\Software\Classes\AmniBrowseHTML","(default)","Amni Browse HTML"),
  @("HKCU:\Software\Classes\AmniBrowseHTML\shell\open\command","(default)",$cmd),
  @("HKCU:\Software\Clients\StartMenuInternet\AmniBrowse","(default)","Amni Browse"),
  @("HKCU:\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities","ApplicationName","Amni Browse"),
  @("HKCU:\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\URLAssociations","http","AmniBrowseHTML"),
  @("HKCU:\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\URLAssociations","https","AmniBrowseHTML"),
  @("HKCU:\Software\RegisteredApplications","AmniBrowse","Software\Clients\StartMenuInternet\AmniBrowse\Capabilities")
)
foreach($row in $pairs){
  if(-not (Test-Path $row[0])){ New-Item -Path $row[0] -Force|Out-Null }
  if($row[1] -eq "(default)"){ Set-ItemProperty -Path $row[0] -Name "(default)" -Value $row[2] }
  else { New-ItemProperty -Path $row[0] -Name $row[1] -Value $row[2] -PropertyType String -Force|Out-Null }
}
if($PSScriptRoot){ $un=Join-Path $PSScriptRoot "uninstall.ps1"; if(Test-Path $un){ Copy-Item $un (Join-Path $dest "uninstall.ps1") -Force } }
Write-Host "Installed. Launching..."
Start-Process $launch
Start-Process "ms-settings:defaultapps"
