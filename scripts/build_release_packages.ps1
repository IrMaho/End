# 👑 Multi-Platform Release Package Builder for End Language v0.4.0-alpha (Windows / Universal)

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "👑 Packaging End Language v0.4.0-alpha Official Release Matrix..." -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan

$Root = Split-Path -Parent $PSScriptRoot
if (-not $Root) { $Root = (Get-Location).Path }

$DistDir = Join-Path $Root "dist"
$StagingDir = Join-Path $DistDir "staging-windows-x64"

if (Test-Path $DistDir) { Remove-Item $DistDir -Recurse -Force }
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null

# 1. Compile Release Binary
Write-Host "[1/4] Compiling optimized endc release binary..." -ForegroundColor Yellow
Set-Location (Join-Path $Root "endc")
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Compilation failed!" -ForegroundColor Red
    exit 1
}

# 2. Stage Assets
Write-Host "[2/4] Assembling release payload..." -ForegroundColor Yellow
$BinDir = Join-Path $StagingDir "bin"
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

$ExeSrc = Join-Path $Root "endc\target\release\endc.exe"
Copy-Item $ExeSrc (Join-Path $BinDir "endc.exe") -Force
Copy-Item $ExeSrc (Join-Path $BinDir "end.exe") -Force

# Copy std library
Copy-Item (Join-Path $Root "std") (Join-Path $StagingDir "std") -Recurse -Force

# Copy Documentation and Configs
Copy-Item (Join-Path $Root "README.md") (Join-Path $StagingDir "README.md") -Force
Copy-Item (Join-Path $Root "LICENSE") (Join-Path $StagingDir "LICENSE") -Force
Copy-Item (Join-Path $Root "Architecture.toml") (Join-Path $StagingDir "Architecture.toml") -Force

# Copy Web Installers
Copy-Item (Join-Path $Root "install.ps1") (Join-Path $StagingDir "install.ps1") -ErrorAction SilentlyContinue
Copy-Item (Join-Path $Root "install.sh") (Join-Path $StagingDir "install.sh") -ErrorAction SilentlyContinue

# 3. Create Windows Zip Archive
Write-Host "[3/4] Creating end-v0.4.0-alpha-windows-x64.zip archive..." -ForegroundColor Yellow
$ZipPath = Join-Path $DistDir "end-v0.4.0-alpha-windows-x64.zip"
Compress-Archive -Path "$StagingDir\*" -DestinationPath $ZipPath -Force

# 4. Generate SHA256 Checksums
Write-Host "[4/4] Generating SHA256 Checksums..." -ForegroundColor Yellow
$Hash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()
$ChecksumFile = Join-Path $DistDir "SHA256SUMS.txt"
Set-Content -Path $ChecksumFile -Value "$Hash  end-v0.4.0-alpha-windows-x64.zip"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "👑 SUCCESS: Release package ready at: $ZipPath" -ForegroundColor Green
Write-Host "SHA256: $Hash" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Cyan
