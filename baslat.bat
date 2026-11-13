@echo off
setlocal
cd /d "%~dp0"

echo ===================================================
echo   LODM - LaviniaOS Download Manager
echo ===================================================
echo.

rem A running instance locks the exe and makes the build fail.
taskkill /IM lodm-app.exe /F >nul 2>&1

echo [1/3] Building UI frontend...
pushd ui
if not exist node_modules (
    call npm.cmd install
    if errorlevel 1 goto :fail
)
call npm.cmd run build
if errorlevel 1 (
    popd
    goto :fail
)
popd

echo [2/3] Building Desktop Engine...
cargo build --manifest-path src-tauri\Cargo.toml --features custom-protocol
if errorlevel 1 goto :fail

echo [3/3] Launching LODM Desktop...
start "" "%~dp0target\debug\lodm-app.exe"
echo [OK] LODM is running.
endlocal
exit /b 0

:fail
echo.
echo [ERROR] Build failed. See messages above.
pause
endlocal
exit /b 1
