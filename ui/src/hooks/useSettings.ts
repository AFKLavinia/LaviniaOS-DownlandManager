import { useCallback, useEffect, useState } from "react";
import { api, errorMessage } from "../lib/api";
import type { Settings } from "../lib/eventTypes";

export const DEFAULT_SETTINGS: Settings = {
  download_dir: "",
  threads: 8,
  speed_limit_kbps: 0,
  max_concurrent: 3,
  proxy: "",
  use_categories: false,
  clipboard_watch: true,
  close_to_tray: true,
  notifications: true,
  language: "tr",
  theme: "amoled",
  api_port: 4141,
  api_token: "",
  autostart: false,
  on_complete_action: "none",
};

const THEME_CACHE_KEY = "lodm-theme";

/** Read the cached theme so the first paint already uses the right palette. */
export function cachedTheme(): Settings["theme"] {
  try {
    return localStorage.getItem(THEME_CACHE_KEY) === "light" ? "light" : "amoled";
  } catch {
    return "amoled";
  }
}

export interface UseSettings {
  settings: Settings;
  loaded: boolean;
  restartRequired: boolean;
  /** Persist a (partial) change; resolves with an error message or null. */
  update: (patch: Partial<Settings>) => Promise<string | null>;
}

/** Loads settings from the backend and applies theme to the document. */
export function useSettings(): UseSettings {
  const [settings, setSettings] = useState<Settings>({
    ...DEFAULT_SETTINGS,
    theme: cachedTheme(),
  });
  const [loaded, setLoaded] = useState(false);
  const [restartRequired, setRestartRequired] = useState(false);

  useEffect(() => {
    api
      .getSettings()
      .then((s) => setSettings(s))
      .catch(() => undefined)
      .finally(() => setLoaded(true));
  }, []);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", settings.theme);
    document.documentElement.lang = settings.language;
    try {
      localStorage.setItem(THEME_CACHE_KEY, settings.theme);
    } catch {
      /* storage unavailable */
    }
  }, [settings.theme, settings.language]);

  const update = useCallback(
    async (patch: Partial<Settings>): Promise<string | null> => {
      setSettings((prev) => ({ ...prev, ...patch }));
      try {
        const res = await api.saveSettings({ ...settings, ...patch });
        setSettings(res.settings);
        if (res.restart_required) setRestartRequired(true);
        return null;
      } catch (e) {
        return errorMessage(e);
      }
    },
    [settings],
  );

  return { settings, loaded, restartRequired, update };
}
