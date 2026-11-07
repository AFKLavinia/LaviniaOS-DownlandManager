import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { DownloadRow } from "./DownloadRow";
import { DownloadIcon } from "../../icons";

interface DownloadTableProps {
  jobs: DownloadJob[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export const DownloadTable: React.FC<DownloadTableProps> = ({
  jobs,
  selectedId,
  onSelect,
}) => {
  if (jobs.length === 0) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--text-muted)",
          gap: "12px",
        }}
      >
        <DownloadIcon size={36} strokeWidth={1.5} />
        <span style={{ fontSize: "13px" }}>Henüz bir indirme görevi bulunmuyor.</span>
      </div>
    );
  }

  return (
    <div className="table-container">
      <table className="download-table">
        <thead>
          <tr>
            <th>Dosya Adı</th>
            <th>Boyut</th>
            <th>Durum</th>
            <th>İlerleme & Segmentler</th>
            <th>Hız</th>
            <th>Kalan Süre</th>
            <th>Bağlantılar</th>
          </tr>
        </thead>
        <tbody>
          {jobs.map((job) => {
            const idStr = typeof job.id === "object" ? job.id[0] : job.id;
            return (
              <DownloadRow
                key={idStr}
                job={job}
                isSelected={selectedId === idStr}
                onSelect={onSelect}
              />
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
