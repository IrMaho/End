@echo off
title EndLedger Interactive EndUI Dashboard ^& AI DevMode Overlay
color 0A
echo ==================================================================
echo 🎨 Launching EndLedger Interactive EndUI Dashboard ^& DevServer...
echo ==================================================================
echo.
end.exe ui "%~dp0src\main.end" --dev --serve --open --port 3000
pause
