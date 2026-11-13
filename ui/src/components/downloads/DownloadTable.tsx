import React, { useMemo, useState } from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { DownloadRow } from "./DownloadRow";
import { DownloadIcon } from "../../icons";
import { jobIdOf, jobStats } from "../../lib/jobs";
import { useT } from "../../lib/I18nContext";

type SortKey = "name" | "size" | "status" | "progress" | "speed";

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
  const [sortKey, setSortKey] = useState<SortKey | null>(null);
  const [sortAsc, setSortAsc] = useState(true);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortAsc((a) => !a);
    else {
      setSortKey(key);
      setSortAsc(true);
    }
  };

  const sortedJobs = useMemo(() => {
    if (!sortKey) return jobs;
    const copy = [...jobs];
    copy.sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case "name":
          cmp = a.filename.localeCompare(b.filename);
          break;
        case "size":
          cmp = (a.total_bytes ?? 0) - (b.total_bytes ?? 0);
          break;
        case "status":
          cmp = a.state.localeCompare(b.state);
          break;
        case "progress":
          cmp = jobStats(a).percentage - jobStats(b).percentage;
          break;
        case "speed":
          cmp = jobStats(a).speed - jobStats(b).speed;
          break;
      }
      return sortAsc ? cmp : -cmp;
    });
    return copy;
  }, [jobs, sortKey, sortAsc]);

  const th = (key: SortKey, label: string) => (
    <th onClick={() => toggleSort(key)} style={{ cursor: "pointer" }}>
      {label}
      {sortKey === key ? (sortAsc ? " ^" : " v") : ""}
    </th>
  );

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
            {th("name", t("colName"))}
            {th("size", t("colSize"))}
            {th("status", t("colStatus"))}
            {th("progress", t("colProgress"))}
            {th("speed", t("colSpeed"))}
            <th>{t("colEta")}</th>
            <th>{t("colConn")}</th>
          </tr>
        </thead>
        <tbody>
          {sortedJobs.map((job) => {
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
