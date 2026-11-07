@echo off
setlocal
cd /d "%~dp0"

echo ===================================================
echo   LODM - LaviniaOS Download Manager
echo ===================================================
echo.

echo [1/3] Building UI frontend...
cd ui
call npm.cmd run build
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Frontend build failed!
    pause
    exit /b 1
)
cd ..

echo [2/3] Building Desktop Engine...
cargo build --manifest-path src-tauri\Cargo.toml
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    pause
    exit /b 1
)

echo [3/3] Launching LODM Desktop...
start "" "%~dp0target\debug\lodm-app.exe"
echo [OK] LODM is running.
endlocal
