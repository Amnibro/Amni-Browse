param(
  [string]$ProfileDir = "",
  [string]$OutFile = ""
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if (-not $root) { $root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path }
$py = Join-Path $root "scripts\export_chrome_amni.py"
if (-not $ProfileDir) {
  $ProfileDir = Join-Path $env:LOCALAPPDATA "Google\Chrome\User Data\Default"
}
if (-not $OutFile) {
  $OutFile = Join-Path $env:USERPROFILE "Documents\amni-chrome-import.json"
}
$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) { $python = Get-Command py -ErrorAction SilentlyContinue }
if (-not $python) { throw "python required for export-chrome-amni" }
& $python.Source $py $ProfileDir $OutFile
if ($LASTEXITCODE -ne 0) { throw "export failed" }
Write-Host "Copy $OutFile to the phone and Import in AmniBrowse."
