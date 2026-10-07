$ErrorActionPreference = "Stop"

cargo build --release

New-Item -ItemType Directory -Force "dist" | Out-Null

Copy-Item `
    "target\release\Meltick.exe" `
    "dist\Meltick.scr" `
    -Force

Write-Host ""
Write-Host "Built: dist\Meltick.scr"