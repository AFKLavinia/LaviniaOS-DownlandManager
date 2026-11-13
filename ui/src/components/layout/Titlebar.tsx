import React, { useEffect, useState } from "react";
import { SpeedIcon, SettingsIcon } from "../../icons";
import { formatSpeed } from "../../lib/formatters";
import { useT } from "../../lib/I18nContext";
import { api } from "../../lib/api";

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
  const t = useT();
  const [isMaximized, setIsMaximized] = useState(false);

  // Derive the maximized state from the real window size (also covers dblclick on the bar).
  useEffect(() => {
    const sync = () =>
      setIsMaximized(
        window.outerWidth >= window.screen.availWidth - 2 &&
          window.outerHeight >= window.screen.availHeight - 2,
      );
    sync();
    window.addEventListener("resize", sync);
    return () => window.removeEventListener("resize", sync);
  }, []);

  const handleMaximize = () => {
    api.maximize().catch(() => undefined);
  };

  return (
    <header className="custom-titlebar" data-tauri-drag-region>
      <div className="titlebar-left" data-tauri-drag-region>
        <span className="lodm-logo-badge">LODM</span>
        <span className="titlebar-divider">/</span>
        <span className="titlebar-title">{t("appName")}</span>
      </div>

      <div className="titlebar-center" data-tauri-drag-region>
        <div className="stat-chip">
          <SpeedIcon size={12} strokeWidth={2} />
          <span style={{ fontSize: "10px", color: "var(--text-muted)" }}>{t("speed")}:</span>
          <strong
            style={{ fontFamily: "var(--font-mono)", fontSize: "10px", color: "var(--text-primary)" }}
          >
            {formatSpeed(totalSpeed)}
          </strong>
        </div>
      </div>

      <div className="titlebar-right no-drag">
        <button
          className="theme-toggle-btn"
          onClick={onToggleTheme}
          title={t("toggleTheme")}
        >
          {theme === "amoled" ? t("themeAmoled") : t("themeLight")}
        </button>

        <button className="titlebar-btn" onClick={onOpenSettings} title={`${t("settings")} (Ctrl+,)`}>
          <SettingsIcon size={13} strokeWidth={2} />
        </button>

        <div className="window-controls">
          <button
            className="win-btn win-min"
            onClick={() => api.minimize().catch(() => undefined)}
            title={t("minimize")}
          >
            <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
              <rect width="10" height="1" />
            </svg>
          </button>

          <button className="win-btn win-max" onClick={handleMaximize} title={t("maximize")}>
            {isMaximized ? (
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1">
                <rect x="2.5" y="0.5" width="7" height="7" />
                <path d="M0.5 2.5v7h7" />
              </svg>
            ) : (
              <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1">
                <rect x="0.5" y="0.5" width="8" height="8" />
              </svg>
            )}
          </button>

          <button
            className="win-btn win-close"
            onClick={() => api.close().catch(() => window.close())}
            title={t("close")}
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
