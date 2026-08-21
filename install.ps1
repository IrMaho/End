# 👑 End Language Windows One-Line Installer
# Usage: irm https://github.com/IrMaho/End/releases/latest/download/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "👑 Installing End Programming Language (v1.0.0)..." -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan

$InstallDir = "$env:LOCALAPPDATA\EndLanguage"
$BinDir = "$InstallDir\bin"
$ZipUrl = "https://github.com/IrMaho/End/releases/download/v1.0.0/end-v1.0.0-windows-x64.zip"
$TempZip = "$env:TEMP\end-v1.0.0-windows-x64.zip"

# Step 1: Create Directories
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Step 2: Download Release Payload
Write-Host "[1/3] Downloading End Language v1.0.0 binary bundle..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing
    Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
    Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    Write-Host "✔ Downloaded & extracted successfully!" -ForegroundColor Green
} catch {
    Write-Host "ℹ Offline mode / fallback: locating local compiled binaries..." -ForegroundColor Cyan
    $LocalBin = "$PSScriptRoot\endc\target\release\endc.exe"
    if (Test-Path $LocalBin) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        Copy-Item $LocalBin "$BinDir\endc.exe" -Force
        Copy-Item $LocalBin "$BinDir\end.exe" -Force
        if (Test-Path "$PSScriptRoot\std") {
            Copy-Item "$PSScriptRoot\std" "$InstallDir\std" -Recurse -Force
        }
        Write-Host "✔ Installed from local release artifact!" -ForegroundColor Green
    } else {
        Write-Host "❌ Failed to download or locate release binaries: $_" -ForegroundColor Red
        exit 1
    }
}

# Step 3: Add to User PATH
Write-Host "[2/3] Registering Environment Variables (PATH)..." -ForegroundColor Yellow
$UserPath = [Environment]::GetEnvironmentVariable("PATH", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$BinDir", [EnvironmentVariableTarget]::User)
    $env:PATH = "$env:PATH;$BinDir"
    Write-Host "✔ Added '$BinDir' to User PATH!" -ForegroundColor Green
} else {
    Write-Host "✔ '$BinDir' already exists in PATH." -ForegroundColor Green
}

# Step 4: Verify Installation
Write-Host "[3/3] Verifying installation..." -ForegroundColor Yellow
$EndExe = "$BinDir\endc.exe"
if (Test-Path $EndExe) {
    & $EndExe eval "15 * 4 + 20"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "👑 SUCCESS: End Language v1.0.0 is installed and ready to use!" -ForegroundColor Green
Write-Host "Open a new terminal and run: `end --help` or `end eval '100 * 5'`" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Cyan
