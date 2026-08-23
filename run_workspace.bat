@echo off
chcp 65001 >nul 2>&1
title 👑 End DevOps ^& AI Workspace
cd /d "%~dp0"

:: Try release build first, then debug
if exist "endc\target\release\endc.exe" (
    "endc\target\release\endc.exe" run "examples\app\main.end"
) else if exist "endc\target\debug\endc.exe" (
    "endc\target\debug\endc.exe" run "examples\app\main.end"
) else (
    echo.
    echo  [ERROR] endc.exe not found.
    echo  Build first with: cd endc ^&^& cargo build --release
    echo.
    pause
    exit /b 1
)

pause
