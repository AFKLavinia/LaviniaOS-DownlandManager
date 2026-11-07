@echo off
chcp 65001 >nul
title LODM - LaviniaOS Download Manager Başlatıcı

echo ===================================================
echo   LODM - LaviniaOS Download Manager Başlatılıyor...
echo ===================================================
echo.

cd /d "%~dp0"

echo [1/3] Arayüz statik dosyaları derleniyor...
cd ui
call npm.cmd run build
if %ERRORLEVEL% NEQ 0 (
    echo [HATA] Arayüz derlemesi başarısız oldu!
    pause
    exit /b %ERRORLEVEL%
)
cd ..

echo [2/3] Tauri v2 yerel masaüstü motoru derleniyor...
cargo build --manifest-path src-tauri\Cargo.toml
if %ERRORLEVEL% NEQ 0 (
    echo [HATA] Tauri motoru derlemesi başarısız oldu!
    pause
    exit /b %ERRORLEVEL%
)

echo [3/3] Arayüz sunucusu hazırlanıyor ve LODM masaüstü penceresi açılıyor...
start /B "" npm.cmd --prefix ui run dev >nul 2>&1
timeout /t 2 >nul

start "" "target\debug\lodm-app.exe"

echo.
echo ===================================================
echo   [BAŞARILI] LODM Masaüstü Uygulaması Açıldı!
echo ===================================================
timeout /t 3 >nul
exit
