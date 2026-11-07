"""
LODM (LaviniaOS Download Manager) - Native Desktop Application Runner
PySide6 / Qt6 WebEngine Runtime & High-Performance Rust Core Engine
"""

import sys
import os
import subprocess
import time
from pathlib import Path

from PySide6.QtWidgets import QApplication, QMainWindow, QVBoxLayout, QWidget
from PySide6.QtCore import QUrl, Qt
from PySide6.QtWebEngineWidgets import QWebEngineView

ROOT_DIR = Path(__file__).resolve().parent
CORE_EXE = ROOT_DIR / "target" / "debug" / "lavinia_downloader.exe"
UI_DIR = ROOT_DIR / "ui"

class LodmMainWindow(QMainWindow):
    def __init__(self, target_url: str):
        super().__init__()
        self.setWindowTitle("LODM — LaviniaOS Download Manager")
        self.resize(1180, 760)
        self.setMinimumSize(960, 600)

        # Frameless window support for custom titlebar
        self.setWindowFlags(Qt.Window | Qt.FramelessWindowHint | Qt.WindowMinMaxButtonsHint)
        self.setAttribute(Qt.WA_TranslucentBackground, False)

        # WebEngine View
        self.browser = QWebEngineView(self)
        self.browser.setUrl(QUrl(target_url))

        # Central layout
        central_widget = QWidget(self)
        layout = QVBoxLayout(central_widget)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)
        layout.addWidget(self.browser)
        self.setCentralWidget(central_widget)

def main():
    print("=== LODM: Starting Native Desktop Application ===")

    # 1. Ensure Rust Core Engine is running
    try:
        import urllib.request
        urllib.request.urlopen("http://127.0.0.1:4141/api/downloads", timeout=0.5)
        print("[LODM] Rust Core Engine is already active on port 4141.")
    except Exception:
        print("[LODM] Launching Rust Core Engine...")
        if CORE_EXE.exists():
            subprocess.Popen([str(CORE_EXE)], cwd=str(ROOT_DIR), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            time.sleep(0.8)
        else:
            print("[LODM] Compiling Rust Core...")
            subprocess.run(["cargo", "build", "--manifest-path", "core/Cargo.toml"], cwd=str(ROOT_DIR))
            subprocess.Popen([str(CORE_EXE)], cwd=str(ROOT_DIR), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    # 2. Check UI server
    ui_url = "http://localhost:5173"
    try:
        import urllib.request
        urllib.request.urlopen(ui_url, timeout=0.5)
        print("[LODM] UI dev server active on", ui_url)
    except Exception:
        print("[LODM] Starting UI server...")
        subprocess.Popen(["npm.cmd", "run", "dev"], cwd=str(UI_DIR), shell=True)
        time.sleep(1.5)

    # 3. Launch PySide6 Native Desktop Window
    app = QApplication(sys.argv)
    app.setApplicationName("LODM — LaviniaOS Download Manager")

    window = LodmMainWindow(ui_url)
    window.show()
    window.raise_()
    window.activateWindow()

    print("[LODM] Native Desktop Window opened on screen.")
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
