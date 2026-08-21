# ?? End Language Windows One-Line Installer
# Usage: irm https://github.com/IrMaho/End/releases/latest/download/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Version = "v0.4.0-alpha"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "?? Installing End Programming Language ($Version)..." -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan

$InstallDir = "$env:LOCALAPPDATA\EndLanguage"
$BinDir = "$InstallDir\bin"
$ZipUrl = "https://github.com/IrMaho/End/releases/download/$Version/end-$Version-windows-x64.zip"
$ShaUrl = "https://github.com/IrMaho/End/releases/download/$Version/end-$Version-windows-x64.zip.sha256"
$TempZip = "$env:TEMP\end-$Version-windows-x64.zip"

# Step 1: Create Directories
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Step 2: Download Release Payload
Write-Host "[1/3] Downloading End Language $Version binary bundle..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing
    
    # Optional Checksum Verification
    try {
        $expectedSha = (Invoke-WebRequest -Uri $ShaUrl -UseBasicParsing).Content.Trim().Split(" ")[0]
        $actualSha = (Get-FileHash -Path $TempZip -Algorithm SHA256).Hash
        if ($expectedSha -and ($expectedSha.ToLower() -eq $actualSha.ToLower())) {
            Write-Host "? SHA256 Checksum Verified: $actualSha" -ForegroundColor Green
        }
    } catch {
        Write-Host "? Continuing with TLS integrity verification..." -ForegroundColor DarkGray
    }

    Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
    Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    Write-Host "? Downloaded & extracted successfully!" -ForegroundColor Green
} catch {
    Write-Host "? Offline mode / fallback: locating local compiled binaries..." -ForegroundColor Cyan
    $LocalBin = "$PSScriptRoot\endc\target\release\endc.exe"
    if (Test-Path $LocalBin) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
        Copy-Item $LocalBin "$BinDir\endc.exe" -Force
        Copy-Item $LocalBin "$BinDir\end.exe" -Force
        if (Test-Path "$PSScriptRoot\std") {
            Copy-Item "$PSScriptRoot\std" "$InstallDir\std" -Recurse -Force
        }
        Write-Host "? Installed from local release artifact!" -ForegroundColor Green
    } else {
        Write-Host "? Failed to download or locate release binaries: $_" -ForegroundColor Red
        exit 1
    }
}

# Step 3: Register Environment Variable PATH
Write-Host "[2/3] Adding $BinDir to User PATH..." -ForegroundColor Yellow
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$BinDir"
    Write-Host "? Added to User PATH successfully!" -ForegroundColor Green
} else {
    Write-Host "? Already in PATH!" -ForegroundColor Green
}

# Step 4: Verification
Write-Host "[3/3] Verifying installation..." -ForegroundColor Yellow
$InstalledExe = "$BinDir\end.exe"
if (Test-Path $InstalledExe) {
    Write-Host "`n?? SUCCESS! End Programming Language is ready." -ForegroundColor Green
    Write-Host "Run 'end --help' or 'end dev' to get started." -ForegroundColor Cyan
} else {
    Write-Host "? Installation check failed: $InstalledExe not found." -ForegroundColor Red
    exit 1
}
