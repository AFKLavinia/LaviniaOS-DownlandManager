@echo off
setlocal
cd /d "%~dp0"

echo Building LODM release installer...
taskkill /IM lodm-app.exe /F >nul 2>&1

pushd ui
if not exist node_modules call npm.cmd install
popd

call npx.cmd --yes @tauri-apps/cli build
if errorlevel 1 (
    echo [ERROR] Release build failed.
    pause
    exit /b 1
)

echo.
echo Done. Installer: target\release\bundle\nsis\
endlocal
