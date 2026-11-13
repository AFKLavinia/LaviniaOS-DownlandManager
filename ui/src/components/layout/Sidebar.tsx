import React from "react";
import {
  DownloadIcon,
  PlayIcon,
  CheckCircleIcon,
  PauseIcon,
  AlertCircleIcon,
  TrashIcon,
} from "../../icons";
import { useT } from "../../lib/I18nContext";
import type { I18nKey } from "../../lib/i18n";

export type FilterCategory = "all" | "downloading" | "completed" | "paused" | "failed";

interface SidebarProps {
  currentFilter: FilterCategory;
  onFilterChange: (cat: FilterCategory) => void;
  counts: Record<FilterCategory, number>;
  onResumeAll: () => void;
  onPauseAll: () => void;
  onClearCompleted: () => void;
}

const ITEMS: { key: FilterCategory; label: I18nKey; icon: React.ReactNode }[] = [
  { key: "all", label: "filterAll", icon: <DownloadIcon size={14} /> },
  { key: "downloading", label: "filterDownloading", icon: <PlayIcon size={14} /> },
  { key: "completed", label: "filterCompleted", icon: <CheckCircleIcon size={14} /> },
  { key: "paused", label: "filterPaused", icon: <PauseIcon size={14} /> },
  { key: "failed", label: "filterFailed", icon: <AlertCircleIcon size={14} /> },
];

export const Sidebar: React.FC<SidebarProps> = ({
  currentFilter,
  onFilterChange,
  counts,
  onResumeAll,
  onPauseAll,
  onClearCompleted,
}) => {
  const t = useT();

  return (
    <aside className="app-sidebar">
      <div className="sidebar-heading">{t("statuses")}</div>
      {ITEMS.map((item) => (
        <div
          key={item.key}
          className={`sidebar-item ${currentFilter === item.key ? "active" : ""}`}
          onClick={() => onFilterChange(item.key)}
        >
          <div className="sidebar-label">
            {item.icon}
            <span>{t(item.label)}</span>
          </div>
          <span className="sidebar-counter">{counts[item.key] || 0}</span>
        </div>
      ))}

      <div className="sidebar-heading" style={{ marginTop: 8 }}>
        {t("actions")}
      </div>
      <div className="sidebar-item" onClick={onResumeAll}>
        <div className="sidebar-label">
          <PlayIcon size={14} />
          <span>{t("resumeAll")}</span>
        </div>
      </div>
      <div className="sidebar-item" onClick={onPauseAll}>
        <div className="sidebar-label">
          <PauseIcon size={14} />
          <span>{t("pauseAll")}</span>
        </div>
      </div>
      <div className="sidebar-item" onClick={onClearCompleted}>
        <div className="sidebar-label">
          <TrashIcon size={14} />
          <span>{t("clearCompleted")}</span>
        </div>
      </div>
    </aside>
  );
};
