# 👑 Universal Deployer for Antigravity IDE, VS Code, Cursor & Windsurf

Write-Host "👑 Deploying End Language Extension to Antigravity IDE & All Editors..." -ForegroundColor Green

$Root = "c:\Users\ASUS\Desktop\flutter_project\end"
$VscodeDir = Join-Path $Root "editors\vscode"

Write-Host "[1/2] Compiling TypeScript extension..." -ForegroundColor Yellow
Set-Location $VscodeDir
npm run compile

$TargetDirs = @(
    "C:\Users\ASUS\.antigravity-ide\extensions\endlanguage.end-lang-0.2.0",
    "C:\Users\ASUS\.antigravity\extensions\endlanguage.end-lang-0.2.0",
    "C:\Users\ASUS\.vscode\extensions\endlanguage.end-lang-0.2.0",
    "C:\Users\ASUS\.cursor\extensions\endlanguage.end-lang-0.2.0",
    "C:\Users\ASUS\.windsurf\extensions\endlanguage.end-lang-0.2.0"
)

$Files = @("package.json", "language-configuration.json", "README.md", "icon.png")
$Dirs = @("dist", "syntaxes", "snippets")

Write-Host "[2/2] Copying extension bundles..." -ForegroundColor Yellow

foreach ($Target in $TargetDirs) {
    $parent = Split-Path $Target -Parent
    if (-not (Test-Path $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    if (Test-Path $Target) {
        Remove-Item -Path $Target -Recurse -Force
    }
    New-Item -ItemType Directory -Path $Target -Force | Out-Null

    foreach ($f in $Files) {
        $src = Join-Path $VscodeDir $f
        if (Test-Path $src) {
            Copy-Item -Path $src -Destination (Join-Path $Target $f) -Force
        }
    }

    foreach ($d in $Dirs) {
        $src = Join-Path $VscodeDir $d
        if (Test-Path $src) {
            Copy-Item -Path $src -Destination $Target -Recurse -Force
        }
    }

    Write-Host "  ✔ Successfully installed into: $Target" -ForegroundColor Green
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "👑 SUCCESS: Extension is now installed in Antigravity IDE!" -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan
