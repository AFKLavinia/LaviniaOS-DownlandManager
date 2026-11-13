import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { DownloadRow } from "./DownloadRow";
import { DownloadIcon } from "../../icons";
import { jobIdOf } from "../../lib/jobs";
import { useT } from "../../lib/I18nContext";

interface DownloadTableProps {
  jobs: DownloadJob[];
  /** True when the list is empty only because of the filter/search. */
  filtered: boolean;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onActivate: (job: DownloadJob) => void;
  onContextMenu: (job: DownloadJob, x: number, y: number) => void;
}

export const DownloadTable: React.FC<DownloadTableProps> = ({
  jobs,
  filtered,
  selectedId,
  onSelect,
  onActivate,
  onContextMenu,
}) => {
  const t = useT();

  if (jobs.length === 0) {
    return (
      <div className="empty-state">
        <DownloadIcon size={34} strokeWidth={1.5} />
        <span style={{ fontSize: "13px" }}>{filtered ? t("noMatch") : t("emptyList")}</span>
        {!filtered && <span style={{ fontSize: "11px" }}>{t("emptyHint")}</span>}
      </div>
    );
  }

  return (
    <div className="table-container">
      <table className="download-table">
        <thead>
          <tr>
            <th>{t("colName")}</th>
            <th>{t("colSize")}</th>
            <th>{t("colStatus")}</th>
            <th>{t("colProgress")}</th>
            <th>{t("colSpeed")}</th>
            <th>{t("colEta")}</th>
            <th>{t("colConn")}</th>
          </tr>
        </thead>
        <tbody>
          {jobs.map((job) => {
            const id = jobIdOf(job);
            return (
              <DownloadRow
                key={id}
                job={job}
                isSelected={selectedId === id}
                onSelect={onSelect}
                onActivate={onActivate}
                onContextMenu={onContextMenu}
              />
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
