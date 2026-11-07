import React from "react";
import {
  DownloadIcon,
  PlayIcon,
  CheckCircleIcon,
  PauseIcon,
  AlertCircleIcon,
} from "../../icons";

export type FilterCategory = "all" | "downloading" | "completed" | "paused" | "failed";

interface SidebarProps {
  currentFilter: FilterCategory;
  onFilterChange: (cat: FilterCategory) => void;
  counts: Record<FilterCategory, number>;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentFilter,
  onFilterChange,
  counts,
}) => {
  const items: { key: FilterCategory; label: string; icon: React.ReactNode }[] = [
    { key: "all", label: "Tüm İndirmeler", icon: <DownloadIcon size={15} /> },
    { key: "downloading", label: "İndiriliyor", icon: <PlayIcon size={15} /> },
    { key: "completed", label: "Tamamlandı", icon: <CheckCircleIcon size={15} /> },
    { key: "paused", label: "Duraklatıldı", icon: <PauseIcon size={15} /> },
    { key: "failed", label: "Hatalı", icon: <AlertCircleIcon size={15} /> },
  ];

  return (
    <aside className="app-sidebar">
      <div className="sidebar-heading">Durumlar</div>
      {items.map((item) => (
        <div
          key={item.key}
          className={`sidebar-item ${currentFilter === item.key ? "active" : ""}`}
          onClick={() => onFilterChange(item.key)}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            {item.icon}
            <span>{item.label}</span>
          </div>
          <span className="sidebar-counter">{counts[item.key] || 0}</span>
        </div>
      ))}
    </aside>
  );
};
