$ErrorActionPreference = "Stop"
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "ERROR: Must run from an ELEVATED PowerShell." -ForegroundColor Red
    Write-Host "  1. Press Win+X" -ForegroundColor Yellow
    Write-Host "  2. Click 'Terminal (Admin)' or 'Windows PowerShell (Admin)'" -ForegroundColor Yellow
    Write-Host "  3. Paste the command again" -ForegroundColor Yellow
    Read-Host "Press Enter to close"
    exit 1
}
Write-Host "=== Amni-Browse build prereqs ===" -ForegroundColor Cyan
$candidates = @("C:\Program Files\gstreamer\1.0\msvc_x86_64", "C:\gstreamer\1.0\msvc_x86_64")
$gstRoot = $null
foreach ($c in $candidates) { if (Test-Path "$c\lib\pkgconfig") { $gstRoot = $c; break } }
if ($null -eq $gstRoot) {
    Write-Host "GStreamer not found at known paths. Installing via Chocolatey..." -ForegroundColor Yellow
    choco install gstreamer -y --version=1.24.12
    choco install gstreamer-devel -y --version=1.24.12
    $gstRoot = "C:\gstreamer\1.0\msvc_x86_64"
    if (-not (Test-Path "$gstRoot\lib\pkgconfig")) { Write-Host "GStreamer install failed." -ForegroundColor Red; Read-Host "Press Enter to close"; exit 1 }
} else {
    Write-Host ("GStreamer detected at: {0}" -f $gstRoot) -ForegroundColor Green
}
if (-not (Get-Command pkg-config -ErrorAction SilentlyContinue)) {
    Write-Host "pkg-config not found. Installing pkgconfiglite via Chocolatey..." -ForegroundColor Yellow
    choco install pkgconfiglite -y
} else {
    Write-Host ("pkg-config already on PATH: {0}" -f (Get-Command pkg-config).Source) -ForegroundColor Green
}
Write-Host "Setting machine environment variables..." -ForegroundColor Cyan
[Environment]::SetEnvironmentVariable("GSTREAMER_1_0_ROOT_MSVC_X86_64", "$gstRoot\", "Machine")
$pkgConfigDir = "$gstRoot\lib\pkgconfig"
$curPkg = [Environment]::GetEnvironmentVariable("PKG_CONFIG_PATH", "Machine")
if ([string]::IsNullOrEmpty($curPkg)) {
    [Environment]::SetEnvironmentVariable("PKG_CONFIG_PATH", $pkgConfigDir, "Machine")
} elseif ($curPkg -notlike "*$pkgConfigDir*") {
    [Environment]::SetEnvironmentVariable("PKG_CONFIG_PATH", "$curPkg;$pkgConfigDir", "Machine")
}
$gstBin = "$gstRoot\bin"
$curPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
if ($curPath -notlike "*$gstBin*") { [Environment]::SetEnvironmentVariable("PATH", "$curPath;$gstBin", "Machine") }
Write-Host ""
Write-Host "=== DONE ===" -ForegroundColor Green
Write-Host ("GSTREAMER_1_0_ROOT_MSVC_X86_64 = {0}\" -f $gstRoot)
Write-Host ("PKG_CONFIG_PATH now includes   = {0}" -f $pkgConfigDir)
Write-Host ("PATH now includes              = {0}" -f $gstBin)
Write-Host ""
Write-Host "Verifying pkg-config can find gstreamer-1.0..." -ForegroundColor Cyan
$env:PKG_CONFIG_PATH = $pkgConfigDir
$env:PATH = "$env:PATH;$gstBin;C:\ProgramData\chocolatey\bin"
$verify = & pkg-config --modversion gstreamer-1.0 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host ("  -> gstreamer-1.0 version {0} OK" -f $verify) -ForegroundColor Green
} else {
    Write-Host ("  -> pkg-config verify failed: {0}" -f $verify) -ForegroundColor Red
}
Write-Host ""
Write-Host "IMPORTANT: Close ALL shells and the desktop-shortcut console." -ForegroundColor Yellow
Write-Host "Open a NEW shell (fresh env), then double-click the desktop shortcut." -ForegroundColor Yellow
Read-Host "Press Enter to close"
