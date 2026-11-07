import React, { useState } from "react";
import { SpeedIcon, SettingsIcon } from "../../icons";
import { formatSpeed } from "../../lib/formatters";

interface TitlebarProps {
  totalSpeed: number;
  onOpenSettings: () => void;
}

export const Titlebar: React.FC<TitlebarProps> = ({
  totalSpeed,
  onOpenSettings,
}) => {
  const [isMaximized, setIsMaximized] = useState(false);

  const getTauriWindow = async () => {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      return getCurrentWindow();
    } catch {
      return null;
    }
  };

  const handleMinimize = async () => {
    const win = await getTauriWindow();
    if (win) {
      await win.minimize();
    }
  };

  const handleMaximize = async () => {
    const win = await getTauriWindow();
    if (win) {
      await win.toggleMaximize();
      setIsMaximized(!isMaximized);
    }
  };

  const handleClose = async () => {
    const win = await getTauriWindow();
    if (win) {
      await win.close();
    } else {
      window.close();
    }
  };

  return (
    <header className="custom-titlebar" data-tauri-drag-region>
      {/* Left: LODM Brand Tag (No folder icon, clean typography) */}
      <div className="titlebar-left" data-tauri-drag-region>
        <span className="lodm-logo-badge">LODM</span>
        <span className="titlebar-divider">/</span>
        <span className="titlebar-title">LaviniaOS Download Manager</span>
      </div>

      {/* Center: Draggable area */}
      <div className="titlebar-center" data-tauri-drag-region>
        <div className="stat-chip no-drag">
          <SpeedIcon size={14} strokeWidth={2} />
          <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Hız:</span>
          <strong style={{ fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--accent-sky)" }}>
            {formatSpeed(totalSpeed)}
          </strong>
        </div>
      </div>

      {/* Right: Window Controls */}
      <div className="titlebar-right no-drag">
        <button
          className="titlebar-btn"
          onClick={onOpenSettings}
          title="Ayarlar"
        >
          <SettingsIcon size={14} strokeWidth={2} />
        </button>

        <div className="window-controls">
          <button
            className="win-btn win-min"
            onClick={handleMinimize}
            title="Simge Durumuna Küçült"
          >
            <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
              <rect width="10" height="1" />
            </svg>
          </button>

          <button
            className="win-btn win-max"
            onClick={handleMaximize}
            title={isMaximized ? "Geri Yükle" : "Ekranı Kapla"}
          >
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1">
              <rect x="0.5" y="0.5" width="9" height="9" />
            </svg>
          </button>

          <button
            className="win-btn win-close"
            onClick={handleClose}
            title="Kapat"
          >
            <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" strokeWidth="1.2">
              <line x1="0" y1="0" x2="10" y2="10" />
              <line x1="10" y1="0" x2="0" y2="10" />
            </svg>
          </button>
        </div>
      </div>
    </header>
  );
};
