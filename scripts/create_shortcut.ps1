$ErrorActionPreference = "Stop"
$projDir = Split-Path -Parent $PSScriptRoot
$targetExe = Join-Path $projDir "target\release\amni-browse.exe"
$runBat = Join-Path $projDir "run.bat"
$iconPath = Join-Path $projDir "assets\amni-browse.ico"
$desktop = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktop "Amni Browse.lnk"
if (-not (Test-Path $targetExe)) {
    Write-Host "Building Amni Browse (release)..."
    Push-Location $projDir
    cargo build --release
    Pop-Location
    if (-not (Test-Path $targetExe)) { Write-Error "Build failed: $targetExe not found"; exit 1 }
}
$shell = New-Object -ComObject WScript.Shell
$sc = $shell.CreateShortcut($shortcutPath)
$sc.TargetPath = "$env:ComSpec"
$sc.Arguments = "/c `"`"$runBat`"`""
$sc.WorkingDirectory = $projDir
$sc.Description = "Amni Browse - Privacy-First Web Browser (auto-rebuilds on launch)"
if (Test-Path $iconPath) { $sc.IconLocation = "$iconPath,0" }
$sc.WindowStyle = 1
$sc.Save()
Write-Host "Desktop shortcut created: $shortcutPath"
Write-Host "Launcher: $runBat (runs 'cargo build --release' then starts exe)"
Write-Host "Window style: Normal (visible build console so errors are legible)"
