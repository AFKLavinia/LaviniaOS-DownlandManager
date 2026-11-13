import React from "react";
import type { Toast } from "../../hooks/useToasts";
import { CheckCircleIcon, AlertCircleIcon, DownloadIcon, CloseIcon } from "../../icons";

interface ToastHostProps {
  toasts: Toast[];
  onDismiss: (id: number) => void;
}

export const ToastHost: React.FC<ToastHostProps> = ({ toasts, onDismiss }) => (
  <div className="toast-host" aria-live="polite">
    {toasts.map((t) => (
      <div key={t.id} className={`toast toast-${t.kind}`} role="status">
        <span className="toast-icon">
          {t.kind === "success" ? (
            <CheckCircleIcon size={14} />
          ) : t.kind === "error" ? (
            <AlertCircleIcon size={14} />
          ) : (
            <DownloadIcon size={14} />
          )}
        </span>
        <div className="toast-text">
          <strong>{t.title}</strong>
          {t.body && <span>{t.body}</span>}
        </div>
        <button className="toast-close" onClick={() => onDismiss(t.id)} aria-label="close">
          <CloseIcon size={11} />
        </button>
      </div>
    ))}
  </div>
);
