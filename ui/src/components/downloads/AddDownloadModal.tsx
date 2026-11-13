import React, { useEffect, useState } from "react";
import { Modal } from "../common/Modal";
import { Button } from "../common/Button";
import { DownloadIcon, FolderIcon } from "../../icons";
import { useT } from "../../lib/I18nContext";
import { api } from "../../lib/api";
import { extractUrls, parseHeaderLines } from "../../lib/jobs";
import type { AddRequest } from "../../lib/eventTypes";

interface AddDownloadModalProps {
  isOpen: boolean;
  initialUrl?: string;
  defaultDir: string;
  defaultThreads: number;
  onClose: () => void;
  /** Resolves with an error message, or null on success. */
  onAdd: (requests: AddRequest[]) => Promise<string | null>;
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({
  isOpen,
  initialUrl = "",
  defaultDir,
  defaultThreads,
  onClose,
  onAdd,
}) => {
  const t = useT();
  const [urls, setUrls] = useState("");
  const [filename, setFilename] = useState("");
  const [destination, setDestination] = useState("");
  const [threads, setThreads] = useState(8);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [referer, setReferer] = useState("");
  const [cookie, setCookie] = useState("");
  const [userAgent, setUserAgent] = useState("");
  const [headers, setHeaders] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [paused, setPaused] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // Reset the form every time the dialog opens.
  useEffect(() => {
    if (!isOpen) return;
    setUrls(initialUrl);
    setFilename("");
    setDestination(defaultDir);
    setThreads(defaultThreads);
    setShowAdvanced(false);
    setReferer("");
    setCookie("");
    setUserAgent("");
    setHeaders("");
    setUsername("");
    setPassword("");
    setPaused(false);
    setError(null);
    setBusy(false);
  }, [isOpen, initialUrl, defaultDir, defaultThreads]);

  const parsedUrls = extractUrls(urls);

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (parsedUrls.length === 0) {
      setError(t("invalidUrl"));
      return;
    }
    setBusy(true);
    const common: Omit<AddRequest, "url"> = {
      destination: destination.trim() || undefined,
      threads: Math.min(32, Math.max(1, Number(threads) || 8)),
      headers: parseHeaderLines(headers),
      referer: referer.trim() || undefined,
      cookie: cookie.trim() || undefined,
      user_agent: userAgent.trim() || undefined,
      username: username.trim() || undefined,
      password: password || undefined,
      paused,
    };
    const requests: AddRequest[] = parsedUrls.map((url) => ({
      ...common,
      url,
      // A custom name only makes sense for a single link.
      filename: parsedUrls.length === 1 ? filename.trim() || undefined : undefined,
    }));
    const err = await onAdd(requests);
    setBusy(false);
    if (err) setError(err);
    else onClose();
  };

  const browse = async () => {
    const dir = await api.pickFolder().catch(() => null);
    if (dir) setDestination(dir);
  };

  return (
    <Modal
      isOpen={isOpen}
      title={t("addTitle")}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{t("cancel")}</Button>
          <Button
            variant="primary"
            icon={<DownloadIcon size={14} />}
            onClick={() => handleSubmit()}
            disabled={busy}
          >
            {t("startDownload")}
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} className="form-col">
        <div>
          <label className="field-label">{t("urlLabel")}</label>
          <textarea
            className="input-field"
            rows={3}
            placeholder="https://example.com/file.zip"
            value={urls}
            onChange={(e) => {
              setUrls(e.target.value);
              setError(null);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) handleSubmit();
            }}
            autoFocus
            spellCheck={false}
          />
        </div>

        <div className="form-row">
          <div style={{ flex: 1 }}>
            <label className="field-label">{t("filenameLabel")}</label>
            <input
              className="input-field"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              disabled={parsedUrls.length > 1}
            />
          </div>
          <div style={{ width: 90 }}>
            <label className="field-label">{t("threadsLabel")}</label>
            <input
              type="number"
              min={1}
              max={32}
              className="input-field"
              value={threads}
              onChange={(e) => setThreads(Number(e.target.value))}
            />
          </div>
        </div>

        <div>
          <label className="field-label">{t("destLabel")}</label>
          <div className="form-row">
            <input
              className="input-field"
              style={{ flex: 1 }}
              value={destination}
              onChange={(e) => setDestination(e.target.value)}
            />
            <Button type="button" icon={<FolderIcon size={14} />} onClick={browse}>
              {t("browse")}
            </Button>
          </div>
        </div>

        <button type="button" className="link-btn" onClick={() => setShowAdvanced((v) => !v)}>
          {showAdvanced ? "-" : "+"} {t("advanced")}
        </button>

        {showAdvanced && (
          <div className="form-col advanced-box">
            <div className="form-row">
              <div style={{ flex: 1 }}>
                <label className="field-label">{t("referer")}</label>
                <input
                  className="input-field"
                  value={referer}
                  onChange={(e) => setReferer(e.target.value)}
                />
              </div>
              <div style={{ flex: 1 }}>
                <label className="field-label">{t("userAgent")}</label>
                <input
                  className="input-field"
                  value={userAgent}
                  onChange={(e) => setUserAgent(e.target.value)}
                />
              </div>
            </div>
            <div>
              <label className="field-label">{t("cookie")}</label>
              <input
                className="input-field"
                value={cookie}
                onChange={(e) => setCookie(e.target.value)}
                autoComplete="off"
              />
            </div>
            <div>
              <label className="field-label">{t("headersLabel")}</label>
              <textarea
                className="input-field"
                rows={2}
                value={headers}
                onChange={(e) => setHeaders(e.target.value)}
                placeholder={"X-Api-Key: abc123"}
                spellCheck={false}
              />
            </div>
            <div className="form-row">
              <div style={{ flex: 1 }}>
                <label className="field-label">{t("username")}</label>
                <input
                  className="input-field"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  autoComplete="off"
                />
              </div>
              <div style={{ flex: 1 }}>
                <label className="field-label">{t("password")}</label>
                <input
                  type="password"
                  className="input-field"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  autoComplete="new-password"
                />
              </div>
            </div>
            <label className="check-row">
              <input type="checkbox" checked={paused} onChange={(e) => setPaused(e.target.checked)} />
              <span>{t("startPaused")}</span>
            </label>
          </div>
        )}

        {error && <div className="form-error">{error}</div>}
        {/* Enables Enter-to-submit inside single-line inputs */}
        <button type="submit" style={{ display: "none" }} />
      </form>
    </Modal>
  );
};
