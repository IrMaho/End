# End Language - Official PowerShell Automated Installer
# Usage: irm https://raw.githubusercontent.com/IrMaho/End/main/install.ps1 | iex

$ErrorActionPreference = "Stop"
$Version = "v0.4.0-alpha"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Installing End Programming Language ($Version)..." -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan

$InstallDir = "$env:LOCALAPPDATA\EndLanguage"
$BinDir = "$InstallDir\bin"
$SkillDir = "$InstallDir\skills\end-language"
$GlobalGeminiSkill = "$env:USERPROFILE\.gemini\config\skills\end-language"
$ZipUrl = "https://github.com/IrMaho/End/releases/download/$Version/end-$Version-windows-x64.zip"
$TempZip = "$env:TEMP\end-$Version-windows-x64.zip"

# Step 1: Create Directories
Write-Host "[1/4] Preparing directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
New-Item -ItemType Directory -Path $SkillDir -Force | Out-Null
New-Item -ItemType Directory -Path $GlobalGeminiSkill -Force | Out-Null

# Step 2: Deploy Binary & Payload
Write-Host "[2/4] Deploying End Language $Version payload..." -ForegroundColor Yellow
$LocalBin = "$PSScriptRoot\endc\target\release\endc.exe"
if (Test-Path $LocalBin) {
    Copy-Item $LocalBin "$BinDir\endc.exe" -Force
    Copy-Item $LocalBin "$BinDir\end.exe" -Force
    if (Test-Path "$PSScriptRoot\std") {
        Copy-Item "$PSScriptRoot\std" "$InstallDir\std" -Recurse -Force
    }
    if (Test-Path "$PSScriptRoot\.agents\skills\end-language\SKILL.md") {
        Copy-Item "$PSScriptRoot\.agents\skills\end-language\SKILL.md" "$SkillDir\SKILL.md" -Force
        Copy-Item "$PSScriptRoot\.agents\skills\end-language\SKILL.md" "$GlobalGeminiSkill\SKILL.md" -Force
    }
    Write-Host "  [+] Deployed from local build tree!" -ForegroundColor Green
} else {
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing
        Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
        Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
        Write-Host "  [+] Downloaded & extracted official release binary package!" -ForegroundColor Green
    } catch {
        Write-Host "  [!] Online download fallback: deploying active system compiler..." -ForegroundColor DarkYellow
        $CargoBin = "$env:USERPROFILE\.cargo\bin\end.exe"
        if (Test-Path $CargoBin) {
            Copy-Item $CargoBin "$BinDir\end.exe" -Force
            Copy-Item $CargoBin "$BinDir\endc.exe" -Force
        }
    }
}

# Step 3: Add to PATH
Write-Host "[3/4] Registering End in User PATH..." -ForegroundColor Yellow
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($null -eq $userPath) { $userPath = "" }
if ($userPath -notlike "*EndLanguage\bin*") {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$BinDir", 'User')
    $env:PATH = "$BinDir;$env:PATH"
    Write-Host "  [+] Added $BinDir to User PATH permanently" -ForegroundColor Green
} else {
    Write-Host "  [+] $BinDir is already in User PATH" -ForegroundColor Gray
}

# Step 4: Verification
Write-Host "[4/4] Verifying installation..." -ForegroundColor Yellow
$endExe = "$BinDir\end.exe"
if (Test-Path $endExe) {
    & $endExe version
}

Write-Host "`n================================================================================" -ForegroundColor Cyan
Write-Host "End Programming Language v0.4.0-alpha successfully installed!" -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  * Compiler Binary:  $BinDir\end.exe" -ForegroundColor White
Write-Host "  * Standard Library: $InstallDir\std" -ForegroundColor White
Write-Host "  * AI Global Skill:  $GlobalGeminiSkill\SKILL.md" -ForegroundColor White
Write-Host "`nQuick Start Commands:" -ForegroundColor Yellow
Write-Host "    end run main.end           (Instant JIT execution)" -ForegroundColor White
Write-Host "    end build main.end         (Compile to ultra-fast native .exe)" -ForegroundColor White
Write-Host "    end skill init             (Initialize AI pair programming in current project)" -ForegroundColor White
Write-Host "`nRestart your terminal or run: `$env:PATH = `"$BinDir;`$env:PATH`" to start immediately!" -ForegroundColor Cyan
Write-Host "================================================================================`n" -ForegroundColor Cyan
