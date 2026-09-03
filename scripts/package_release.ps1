param(
  [string]$Version = (Select-String -Path (Join-Path $PSScriptRoot "..\Cargo.toml") -Pattern '^version = "(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$rel = Join-Path $root "target\release"
$exe = Join-Path $rel "amni-browse.exe"
if (-not (Test-Path $exe)) { throw "Missing $exe" }
$out = Join-Path $root "target\release\amni-browse-v$Version-win64.zip"
$stage = Join-Path $env:TEMP "amni-browse-pack-$Version"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item $exe (Join-Path $stage "amni-browse.exe") -Force
Get-ChildItem $rel -File -Filter "*.dll" | ForEach-Object { Copy-Item $_.FullName $stage -Force }
if (Test-Path (Join-Path $rel "libEGL.dll")) { Copy-Item (Join-Path $rel "libEGL.dll") $stage -Force }
if (Test-Path (Join-Path $rel "libGLESv2.dll")) { Copy-Item (Join-Path $rel "libGLESv2.dll") $stage -Force }
foreach ($dir in @("gstreamer-1.0", "assets")) {
  $src = Join-Path $rel $dir
  if (Test-Path $src) { Copy-Item $src (Join-Path $stage $dir) -Recurse -Force }
}
if (Test-Path $out) { Remove-Item $out -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $out)
Remove-Item $stage -Recurse -Force
$hash = (Get-FileHash $out -Algorithm SHA256).Hash.ToLower()
$len = (Get-Item $out).Length
Write-Output "packed=$out"
Write-Output "size=$len"
Write-Output "sha256=$hash"
