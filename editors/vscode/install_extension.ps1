Write-Host "Deploying End Language Extension (v0.4.0)..." -ForegroundColor Green

$CurrentDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $CurrentDir) {
    $CurrentDir = (Get-Location).Path
}

Write-Host "[1/2] Compiling TypeScript extension..." -ForegroundColor Yellow
Set-Location $CurrentDir
npm run compile

Write-Host "[2/2] Installing extension..." -ForegroundColor Yellow
$Target = Join-Path $env:USERPROFILE ".vscode\extensions\endlanguage.end-lang-0.4.0"

if (Test-Path $Target) {
    Remove-Item -Path $Target -Recurse -Force
}
New-Item -ItemType Directory -Path $Target -Force | Out-Null

$Files = @("package.json", "language-configuration.json", "README.md", "icon.png")
foreach ($f in $Files) {
    $src = Join-Path $CurrentDir $f
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination (Join-Path $Target $f) -Force
    }
}

$Dirs = @("dist", "syntaxes", "snippets")
foreach ($d in $Dirs) {
    $src = Join-Path $CurrentDir $d
    if (Test-Path $src) {
        Copy-Item -Path $src -Destination $Target -Recurse -Force
    }
}

Write-Host "SUCCESS: End Language Extension (v0.4.0) Installed into $Target" -ForegroundColor Green
