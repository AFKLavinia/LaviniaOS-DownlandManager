import React from "react";
import { DownloadIcon, SpeedIcon, SettingsIcon } from "../../icons";
import { formatSpeed } from "../../lib/formatters";

interface HeaderProps {
  totalSpeed: number;
  onOpenSettings: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  totalSpeed,
  onOpenSettings,
}) => {
  return (
    <header className="app-header">
      <div className="brand-section">
        <DownloadIcon size={20} className="text-sky-400" strokeWidth={2.5} />
        <h1 className="brand-title">LaviniaOS Downloader</h1>
        <span className="brand-badge">IDM ENGINE</span>
      </div>

      <div className="header-stats">
        <div className="stat-chip">
          <SpeedIcon size={16} strokeWidth={2} />
          <span>Toplam Hız:</span>
          <strong style={{ fontFamily: "var(--font-mono)", color: "var(--text-primary)" }}>
            {formatSpeed(totalSpeed)}
          </strong>
        </div>

        <button
          className="btn"
          style={{ padding: "6px" }}
          onClick={onOpenSettings}
          title="Ayarlar"
        >
          <SettingsIcon size={16} strokeWidth={2} />
        </button>
      </div>
    </header>
  );
};
