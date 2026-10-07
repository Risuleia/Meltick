$ErrorActionPreference = "Stop"

Write-Host "Building Meltick..." -ForegroundColor Cyan

cargo build --release

if ($LASTEXITCODE -ne 0) {
    throw "cargo build --release failed."
}

$dist = Join-Path $PSScriptRoot "..\dist"
$dist = [System.IO.Path]::GetFullPath($dist)

Write-Host "Cleaning dist..." -ForegroundColor Cyan

if (Test-Path $dist) {
    Remove-Item $dist -Recurse -Force
}

New-Item -ItemType Directory -Path $dist | Out-Null

$exe = Join-Path $PSScriptRoot "..\target\release\Meltick.exe"
$scr = Join-Path $dist "Meltick.scr"

if (-not (Test-Path $exe)) {
    throw "Release executable not found: $exe"
}

Copy-Item $exe $scr -Force

if (-not (Test-Path $scr)) {
    throw "Failed to create screensaver: $scr"
}

$size = (Get-Item $scr).Length

Write-Host ""
Write-Host "Meltick build complete." -ForegroundColor Green
Write-Host "Artifact: $scr"
Write-Host ("Size: {0:N2} MB" -f ($size / 1MB))