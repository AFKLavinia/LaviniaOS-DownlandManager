import React, { useEffect, useState } from "react";
import { Modal } from "../common/Modal";
import { Button } from "../common/Button";
import { FolderIcon } from "../../icons";
import { useT } from "../../lib/I18nContext";
import { api } from "../../lib/api";
import type { Settings } from "../../lib/eventTypes";

interface SettingsModalProps {
  isOpen: boolean;
  settings: Settings;
  onClose: () => void;
  /** Resolves with an error message, or null on success. */
  onSave: (patch: Partial<Settings>) => Promise<string | null>;
}

const num = (v: string, fallback: number) => {
  const n = Number(v);
  return Number.isFinite(n) ? n : fallback;
};

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  settings,
  onClose,
  onSave,
}) => {
  const t = useT();
  const [draft, setDraft] = useState<Settings>(settings);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setDraft(settings);
      setError(null);
      setCopied(false);
    }
  }, [isOpen, settings]);

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft((d) => ({ ...d, [key]: value }));

  const browse = async () => {
    const dir = await api.pickFolder().catch(() => null);
    if (dir) set("download_dir", dir);
  };

  const copyToken = async () => {
    try {
      await navigator.clipboard.writeText(draft.api_token);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      /* clipboard blocked */
    }
  };

  const save = async () => {
    const err = await onSave({
      download_dir: draft.download_dir,
      threads: draft.threads,
      speed_limit_kbps: Math.max(0, draft.speed_limit_kbps),
      max_concurrent: draft.max_concurrent,
      proxy: draft.proxy.trim(),
      use_categories: draft.use_categories,
      clipboard_watch: draft.clipboard_watch,
      close_to_tray: draft.close_to_tray,
      notifications: draft.notifications,
      language: draft.language,
      theme: draft.theme,
      api_port: draft.api_port,
    });
    if (err) setError(err);
    else onClose();
  };

  const check = (key: keyof Settings, label: string) => (
    <label className="check-row">
      <input
        type="checkbox"
        checked={Boolean(draft[key])}
        onChange={(e) => set(key, e.target.checked as never)}
      />
      <span>{label}</span>
    </label>
  );

  return (
    <Modal
      isOpen={isOpen}
      title={t("settings")}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{t("cancel")}</Button>
          <Button variant="primary" onClick={save}>
            {t("save")}
          </Button>
        </>
      }
    >
      <div className="settings-scroll">
        <div className="settings-section">{t("setGeneral")}</div>
        <div>
          <label className="field-label">{t("setDownloadDir")}</label>
          <div className="form-row">
            <input
              className="input-field"
              style={{ flex: 1 }}
              value={draft.download_dir}
              onChange={(e) => set("download_dir", e.target.value)}
            />
            <Button type="button" icon={<FolderIcon size={14} />} onClick={browse}>
              {t("browse")}
            </Button>
          </div>
        </div>
        <div className="form-row">
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("setThreads")}</label>
            <input
              type="number"
              min={1}
              max={32}
              className="input-field"
              value={draft.threads}
              onChange={(e) => set("threads", num(e.target.value, 8))}
            />
          </div>
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("setConcurrent")}</label>
            <input
              type="number"
              min={1}
              max={16}
              className="input-field"
              value={draft.max_concurrent}
              onChange={(e) => set("max_concurrent", num(e.target.value, 3))}
            />
          </div>
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("setSpeed")}</label>
            <input
              type="number"
              min={0}
              className="input-field"
              value={draft.speed_limit_kbps}
              onChange={(e) => set("speed_limit_kbps", num(e.target.value, 0))}
            />
          </div>
        </div>
        {check("use_categories", t("setCategories"))}
        <div>
          <label className="field-label">{t("setProxy")}</label>
          <input
            className="input-field"
            value={draft.proxy}
            onChange={(e) => set("proxy", e.target.value)}
            placeholder="http://127.0.0.1:8080"
            spellCheck={false}
          />
        </div>

        <div className="settings-section">{t("setBehavior")}</div>
        {check("clipboard_watch", t("setClipboard"))}
        {check("close_to_tray", t("setTray"))}
        {check("notifications", t("setNotifications"))}

        <div className="settings-section">{t("setAppearance")}</div>
        <div className="form-row">
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("setLanguage")}</label>
            <select
              className="input-field"
              value={draft.language}
              onChange={(e) => set("language", e.target.value as Settings["language"])}
            >
              <option value="tr">Türkçe</option>
              <option value="en">English</option>
            </select>
          </div>
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("toggleTheme")}</label>
            <select
              className="input-field"
              value={draft.theme}
              onChange={(e) => set("theme", e.target.value as Settings["theme"])}
            >
              <option value="amoled">{t("themeAmoled")}</option>
              <option value="light">{t("themeLight")}</option>
            </select>
          </div>
        </div>

        <div className="settings-section">{t("setExtension")}</div>
        <div className="hint">{t("setExtensionHint")}</div>
        <div className="form-row">
          <div style={{ width: 90 }}>
            <label className="field-label">{t("setPort")}</label>
            <input
              type="number"
              className="input-field"
              value={draft.api_port}
              onChange={(e) => set("api_port", num(e.target.value, 4141))}
            />
          </div>
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("setToken")}</label>
            <div className="form-row">
              <input
                className="input-field mono"
                style={{ flex: 1 }}
                value={draft.api_token}
                readOnly
                onFocus={(e) => e.currentTarget.select()}
              />
              <Button type="button" onClick={copyToken}>
                {copied ? t("copied") : t("copy")}
              </Button>
            </div>
          </div>
        </div>

        {error && <div className="form-error">{error}</div>}
      </div>
    </Modal>
  );
};
