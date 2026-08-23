# 👑 End DevOps & AI Workspace — PowerShell Launcher

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Definition

$release = Join-Path $root "endc\target\release\endc.exe"
$debug   = Join-Path $root "endc\target\debug\endc.exe"
$app     = Join-Path $root "examples\app\main.end"

$endc = if (Test-Path $release) { $release } elseif (Test-Path $debug) { $debug } else { $null }

if (-not $endc) {
    Write-Host ""
    Write-Host " [ERROR] endc.exe not found." -ForegroundColor Red
    Write-Host " Build first:" -ForegroundColor Yellow
    Write-Host "   cd endc" -ForegroundColor Cyan
    Write-Host "   cargo build --release" -ForegroundColor Cyan
    Write-Host ""
    Read-Host " Press ENTER to exit"
    exit 1
}

Write-Host ""
Write-Host " Starting End DevOps & AI Workspace..." -ForegroundColor Cyan
Write-Host " Runtime: $endc" -ForegroundColor DarkGray
Write-Host ""

& $endc run $app
