# 👑 Universal One-Click Installer for End Language VS Code & Modern IDE Extension

Write-Host "Deploying End Language Extension (v0.2.0)..." -ForegroundColor Green

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $Root) {
    $Root = (Get-Location).Path
}

$VscodeDir = Join-Path $Root "editors\vscode"

Write-Host "[1/2] Compiling TypeScript extension..." -ForegroundColor Yellow
Set-Location $VscodeDir
npm run compile

Write-Host "[2/2] Installing extension..." -ForegroundColor Yellow
$Target = Join-Path $env:USERPROFILE ".vscode\extensions\endlanguage.end-lang-0.2.0"

if (Test-Path $Target) {
    Remove-Item -Path $Target -Recurse -Force
}
New-Item -ItemType Directory -Path $Target -Force | Out-Null

$Files = @("package.json", "language-configuration.json", "README.md", "icon.png")
foreach ($f in $Files) {
    $src = Join-Path $VscodeDir $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination (Join-Path $Target $f) -Force
    }
}

$Dirs = @("dist", "syntaxes", "snippets")
foreach ($d in $Dirs) {
    $src = Join-Path $VscodeDir $d
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $Target -Recurse -Force
    }
}

Write-Host "SUCCESS: End Language Extension (v0.2.0) Installed into $Target" -ForegroundColor Green
