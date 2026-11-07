import React from "react";
import { Button } from "../common/Button";
import {
  AddIcon,
  PlayIcon,
  PauseIcon,
  TrashIcon,
  FolderIcon,
  SearchIcon,
} from "../../icons";

interface ToolbarProps {
  onAddNew: () => void;
  onResume: () => void;
  onPause: () => void;
  onDelete: () => void;
  onOpenFolder: () => void;
  selectedId: string | null;
  searchQuery: string;
  onSearchChange: (q: string) => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  onAddNew,
  onResume,
  onPause,
  onDelete,
  onOpenFolder,
  selectedId,
  searchQuery,
  onSearchChange,
}) => {
  const hasSelection = selectedId !== null;

  return (
    <div className="app-toolbar">
      <div className="toolbar-group">
        <Button variant="primary" icon={<AddIcon size={14} />} onClick={onAddNew}>
          Yeni İndirme
        </Button>

        <Button
          icon={<PlayIcon size={14} />}
          onClick={onResume}
          disabled={!hasSelection}
          title="Seçili indirmeyi başlat veya devam ettir"
        >
          Devam Et
        </Button>

        <Button
          icon={<PauseIcon size={14} />}
          onClick={onPause}
          disabled={!hasSelection}
          title="Seçili indirmeyi duraklat"
        >
          Duraklat
        </Button>

        <Button
          icon={<TrashIcon size={14} />}
          onClick={onDelete}
          disabled={!hasSelection}
          title="Seçili görevi kaldır"
        >
          Kaldır
        </Button>

        <Button
          icon={<FolderIcon size={14} />}
          onClick={onOpenFolder}
          disabled={!hasSelection}
          title="Dosya dizinini aç"
        >
          Klasörü Aç
        </Button>
      </div>

      <div className="toolbar-group">
        <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
          <span style={{ position: "absolute", left: "8px", color: "var(--text-muted)" }}>
            <SearchIcon size={14} />
          </span>
          <input
            type="text"
            className="input-field"
            placeholder="İndirmelerde ara..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            style={{ paddingLeft: "28px", width: "180px", height: "28px" }}
          />
        </div>
      </div>
    </div>
  );
};
