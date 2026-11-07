import React, { useState } from "react";
import { SpeedIcon, SettingsIcon } from "../../icons";
import { formatSpeed } from "../../lib/formatters";

interface TitlebarProps {
  totalSpeed: number;
  theme: "amoled" | "light";
  onToggleTheme: () => void;
  onOpenSettings: () => void;
}

export const Titlebar: React.FC<TitlebarProps> = ({
  totalSpeed,
  theme,
  onToggleTheme,
  onOpenSettings,
}) => {
  const [isMaximized, setIsMaximized] = useState(false);

  const handleMinimize = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("win_minimize");
    } catch {
      // Browser fallback
    }
  };

  const handleMaximize = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("win_maximize");
      setIsMaximized(!isMaximized);
    } catch {
      // Browser fallback
    }
  };

  const handleClose = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("win_close");
    } catch {
      window.close();
    }
  };

  return (
    <header className="custom-titlebar" data-tauri-drag-region>
      {/* Left: LODM Brand Tag (No folder, pure monochrome AMOLED/Light) */}
      <div className="titlebar-left" data-tauri-drag-region>
        <span className="lodm-logo-badge">LODM</span>
        <span className="titlebar-divider">/</span>
        <span className="titlebar-title">LaviniaOS Download Manager</span>
      </div>

      {/* Center: Draggable area & Speed */}
      <div className="titlebar-center" data-tauri-drag-region>
        <div className="stat-chip no-drag">
          <SpeedIcon size={12} strokeWidth={2} />
          <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>Hız:</span>
          <strong style={{ fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-primary)" }}>
            {formatSpeed(totalSpeed)}
          </strong>
        </div>
      </div>

      {/* Right: Theme switcher, settings & native window controls */}
      <div className="titlebar-right no-drag">
        {/* Clean Theme Toggle Button (AMOLED / ACIK) */}
        <button
          className="theme-toggle-btn"
          onClick={onToggleTheme}
          title={`Temayı Değiştir (Şu an: ${theme === "amoled" ? "AMOLED" : "Açık"})`}
        >
          {theme === "amoled" ? "AMOLED" : "AÇIK"}
        </button>

        <button
          className="titlebar-btn"
          onClick={onOpenSettings}
          title="Ayarlar"
        >
          <SettingsIcon size={13} strokeWidth={2} />
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
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1">
              <rect x="0.5" y="0.5" width="8" height="8" />
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
