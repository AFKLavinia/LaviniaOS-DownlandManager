import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { StatusBadge } from "../common/StatusBadge";
import { SegmentProgress } from "./SegmentProgress";
import { FileIcon, NetworkSegmentsIcon } from "../../icons";
import { formatBytes, formatSpeed, formatEta } from "../../lib/formatters";
import { jobIdOf, jobStats } from "../../lib/jobs";

interface DownloadRowProps {
  job: DownloadJob;
  isSelected: boolean;
  onSelect: (id: string) => void;
  onActivate: (job: DownloadJob) => void;
  onContextMenu: (job: DownloadJob, x: number, y: number) => void;
}

export const DownloadRow: React.FC<DownloadRowProps> = React.memo(
  ({ job, isSelected, onSelect, onActivate, onContextMenu }) => {
    const id = jobIdOf(job);
    const stats = jobStats(job);
    const running = job.state === "downloading";

    return (
      <tr
        className={`download-row ${isSelected ? "selected" : ""}`}
        onClick={() => onSelect(id)}
        onDoubleClick={() => onActivate(job)}
        onContextMenu={(e) => {
          e.preventDefault();
          onSelect(id);
          onContextMenu(job, e.clientX, e.clientY);
        }}
      >
        <td style={{ minWidth: "200px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <FileIcon size={15} />
            <div style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
              <span className="cell-name" title={job.filename}>
                {job.filename}
              </span>
              {job.state === "failed" && job.error_message && (
                <span className="cell-error" title={job.error_message}>
                  {job.error_message}
                </span>
              )}
            </div>
          </div>
        </td>

        <td className="mono" style={{ width: "76px" }}>
          {formatBytes(job.total_bytes)}
        </td>

        <td style={{ width: "104px" }}>
          <StatusBadge state={job.state} />
        </td>

        <td style={{ minWidth: "150px" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: "3px" }}>
            <SegmentProgress
              segments={job.segments}
              totalBytes={job.total_bytes}
              percentage={stats.percentage}
            />
            <div className="cell-sub">
              <span>{stats.percentage.toFixed(1)}%</span>
              <span>
                {formatBytes(stats.downloaded)} / {formatBytes(job.total_bytes)}
              </span>
            </div>
          </div>
        </td>

        <td className="mono" style={{ width: "76px" }}>
          {running ? formatSpeed(stats.speed) : "--"}
        </td>

        <td className="mono" style={{ width: "62px" }}>
          {running ? formatEta(stats.etaSeconds) : "--"}
        </td>

        <td style={{ width: "62px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "11px" }}>
            <NetworkSegmentsIcon size={12} />
            <span>
              {stats.completedSegments}/{job.segments.length || job.thread_count}
            </span>
          </div>
        </td>
      </tr>
    );
  },
);
DownloadRow.displayName = "DownloadRow";
