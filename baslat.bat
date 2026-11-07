@echo off
setlocal
cd /d "%~dp0"

echo ===================================================
echo   LODM - LaviniaOS Download Manager
echo ===================================================
echo.

echo [1/2] Building UI frontend...
cd ui
call npm.cmd run build
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Frontend build failed!
    pause
    exit /b 1
)
cd ..

echo [2/2] Launching Tauri v2 Desktop Engine...
call npx.cmd @tauri-apps/cli dev

endlocal
