@echo off
setlocal enabledelayedexpansion

echo ================================================================================
echo    👑 End Programming Language — Windows Official Automated Installer
echo ================================================================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\EndLanguage"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "SKILL_DIR=%INSTALL_DIR%\skills\end-language"
set "GLOBAL_GEMINI_SKILL=%USERPROFILE%\.gemini\config\skills\end-language"

echo [1/4] Setting up installation directories...
if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%SKILL_DIR%" mkdir "%SKILL_DIR%"
if not exist "%GLOBAL_GEMINI_SKILL%" mkdir "%GLOBAL_GEMINI_SKILL%"

echo [2/4] Copying End Language binaries and standard library...
set "SRC_DIR=%~dp0"
if exist "%SRC_DIR%bin\end.exe" (
    copy /Y "%SRC_DIR%bin\end.exe" "%BIN_DIR%\end.exe" >nul
    copy /Y "%SRC_DIR%bin\endc.exe" "%BIN_DIR%\endc.exe" >nul
) else if exist "%SRC_DIR%endc\target\release\endc.exe" (
    copy /Y "%SRC_DIR%endc\target\release\endc.exe" "%BIN_DIR%\end.exe" >nul
    copy /Y "%SRC_DIR%endc\target\release\endc.exe" "%BIN_DIR%\endc.exe" >nul
) else (
    echo [!] Downloading latest End Language release...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri 'https://github.com/IrMaho/End/releases/download/v0.4.0-alpha/end-v0.4.0-alpha-windows-x64.zip' -OutFile '%TEMP%\end.zip'; Expand-Archive -Path '%TEMP%\end.zip' -DestinationPath '%INSTALL_DIR%' -Force; Remove-Item '%TEMP%\end.zip' -Force"
)

if exist "%SRC_DIR%std" (
    xcopy /E /I /Y "%SRC_DIR%std" "%INSTALL_DIR%\std" >nul
)
if exist "%SRC_DIR%.agents\skills\end-language\SKILL.md" (
    copy /Y "%SRC_DIR%.agents\skills\end-language\SKILL.md" "%SKILL_DIR%\SKILL.md" >nul
    copy /Y "%SRC_DIR%.agents\skills\end-language\SKILL.md" "%GLOBAL_GEMINI_SKILL%\SKILL.md" >nul
)

echo [3/4] Adding End Language to permanent User PATH...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$path = [Environment]::GetEnvironmentVariable('Path', 'User'); " ^
    "if ($path -notlike '*EndLanguage\bin*') { " ^
    "    [Environment]::SetEnvironmentVariable('Path', $path + ';%BIN_DIR%', 'User'); " ^
    "    Write-Host '  ✔ Added %BIN_DIR% to User PATH' -ForegroundColor Green " ^
    "} else { " ^
    "    Write-Host '  ✔ %BIN_DIR% is already in PATH' -ForegroundColor Gray " ^
    "}"

echo [4/4] Verifying installation...
set "PATH=%BIN_DIR%;%PATH%"
"%BIN_DIR%\end.exe" --version

echo.
echo ================================================================================
echo  🎉 End Programming Language v0.4.0-alpha successfully installed!
echo ================================================================================
echo  • Compiler Binary:  %BIN_DIR%\end.exe
echo  • Standard Library: %INSTALL_DIR%\std
echo  • Global AI Skill:  %GLOBAL_GEMINI_SKILL%\SKILL.md
echo.
echo  🚀 Quick Start Commands:
echo    end run main.end           (Instant JIT execution)
echo    end build main.end         (Compile to ultra-fast native .exe)
echo    end skill init             (Initialize AI pair programming in any folder)
echo.
echo  Restart your terminal to use 'end' from anywhere!
echo ================================================================================
pause
