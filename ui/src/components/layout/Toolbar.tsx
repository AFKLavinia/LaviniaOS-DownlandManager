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
import { useT } from "../../lib/I18nContext";

interface ToolbarProps {
  onAddNew: () => void;
  onResume: () => void;
  onPause: () => void;
  onDelete: () => void;
  onOpenFolder: () => void;
  selectedId: string | null;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  searchRef?: React.Ref<HTMLInputElement>;
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
  searchRef,
}) => {
  const t = useT();
  const hasSelection = selectedId !== null;

  return (
    <div className="app-toolbar">
      <div className="toolbar-group">
        <Button variant="primary" icon={<AddIcon size={13} />} onClick={onAddNew} title="Ctrl+N">
          {t("addNew")}
        </Button>
        <Button
          icon={<PlayIcon size={13} />}
          onClick={onResume}
          disabled={!hasSelection}
          title={t("resumeTip")}
        >
          {t("resume")}
        </Button>
        <Button
          icon={<PauseIcon size={13} />}
          onClick={onPause}
          disabled={!hasSelection}
          title={t("pauseTip")}
        >
          {t("pause")}
        </Button>
        <Button
          icon={<TrashIcon size={13} />}
          onClick={onDelete}
          disabled={!hasSelection}
          title={`${t("removeTip")} (Del)`}
        >
          {t("remove")}
        </Button>
        <Button
          icon={<FolderIcon size={13} />}
          onClick={onOpenFolder}
          title={t("openFolderTip")}
        >
          {t("openFolder")}
        </Button>
      </div>

      <div className="toolbar-group">
        <div className="search-box">
          <span className="search-icon">
            <SearchIcon size={13} />
          </span>
          <input
            ref={searchRef}
            type="text"
            className="input-field search-input"
            placeholder={t("search")}
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            title="Ctrl+F"
          />
        </div>
      </div>
    </div>
  );
};
