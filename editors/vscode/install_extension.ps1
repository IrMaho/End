Write-Host "Deploying End Language Extension (v0.4.0) to all IDEs..." -ForegroundColor Green

$CurrentDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $CurrentDir) {
    $CurrentDir = (Get-Location).Path
}

Write-Host "[1/2] Compiling TypeScript extension..." -ForegroundColor Yellow
Set-Location $CurrentDir
npm run compile

Write-Host "[2/2] Installing extension to all IDE extension directories..." -ForegroundColor Yellow

$Targets = @(
    (Join-Path $env:USERPROFILE ".antigravity-ide\extensions\endlanguage.end-lang-0.4.0"),
    (Join-Path $env:USERPROFILE ".antigravity\extensions\endlanguage.end-lang-0.4.0"),
    (Join-Path $env:USERPROFILE ".vscode\extensions\endlanguage.end-lang-0.4.0"),
    (Join-Path $env:USERPROFILE ".cursor\extensions\endlanguage.end-lang-0.4.0"),
    (Join-Path $env:USERPROFILE ".windsurf\extensions\endlanguage.end-lang-0.4.0")
)

$Files = @("package.json", "language-configuration.json", "README.md", "icon.png")
$Dirs = @("dist", "syntaxes", "snippets")

foreach ($Target in $Targets) {
    $ParentDir = Split-Path -Parent $Target
    if (-not (Test-Path $ParentDir)) {
        New-Item -ItemType Directory -Path $ParentDir -Force | Out-Null
    }

    # Clean old versions in parent
    Get-ChildItem -Path $ParentDir -Filter "*endlanguage.end-lang*" | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

    New-Item -ItemType Directory -Path $Target -Force | Out-Null

    foreach ($f in $Files) {
        $src = Join-Path $CurrentDir $f
        if (Test-Path $src) {
            Copy-Item -Path $src -Destination (Join-Path $Target $f) -Force
        }
    }

    foreach ($d in $Dirs) {
        $src = Join-Path $CurrentDir $d
        if (Test-Path $src) {
            Copy-Item -Path $src -Destination $Target -Recurse -Force
        }
    }

    Write-Host "  ✔ Deployed to: $Target" -ForegroundColor Green
}

Write-Host "`nSUCCESS: End Language Extension (v0.4.0) Installed everywhere!" -ForegroundColor Green
