import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { StatusBadge } from "../common/StatusBadge";
import { SegmentProgress } from "./SegmentProgress";
import { FileIcon, NetworkSegmentsIcon } from "../../icons";
import { formatBytes, formatSpeed, formatEta } from "../../lib/formatters";

interface DownloadRowProps {
  job: DownloadJob;
  isSelected: boolean;
  onSelect: (id: string) => void;
}

export const DownloadRow: React.FC<DownloadRowProps> = ({
  job,
  isSelected,
  onSelect,
}) => {
  const jobIdStr = typeof job.id === "object" ? job.id[0] : job.id;

  // Calculate sum of speed across active segments
  const currentSpeed = job.segments.reduce((acc, s) => {
    const spd = typeof s.speed === "object" ? s.speed[0] : s.speed;
    return acc + (spd || 0);
  }, 0);

  // Calculate downloaded bytes
  const downloadedBytes = job.segments.reduce((acc, s) => {
    return acc + Math.max(0, s.current_byte - s.start_byte);
  }, 0);

  const total = job.total_bytes || 0;
  const percentage = total > 0 ? (downloadedBytes / total) * 100 : 0;
  const remainingBytes = Math.max(0, total - downloadedBytes);
  const etaSeconds = currentSpeed > 0 ? remainingBytes / currentSpeed : null;

  const completedSegmentsCount = job.segments.filter((s) => s.state === "completed").length;

  return (
    <tr
      className={`download-row ${isSelected ? "selected" : ""}`}
      onClick={() => onSelect(jobIdStr)}
    >
      <td style={{ minWidth: "220px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <FileIcon size={16} className="text-secondary" />
          <span style={{ fontWeight: 500, color: "var(--text-primary)" }}>
            {job.filename}
          </span>
        </div>
      </td>

      <td style={{ width: "90px", fontFamily: "var(--font-mono)" }}>
        {formatBytes(job.total_bytes)}
      </td>

      <td style={{ width: "110px" }}>
        <StatusBadge state={job.state} />
      </td>

      <td style={{ minWidth: "160px" }}>
        <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          <SegmentProgress
            segments={job.segments}
            totalBytes={job.total_bytes}
            percentage={percentage}
          />
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              fontSize: "10px",
              fontFamily: "var(--font-mono)",
              color: "var(--text-muted)",
            }}
          >
            <span>{percentage.toFixed(1)}%</span>
            <span>
              {formatBytes(downloadedBytes)} / {formatBytes(job.total_bytes)}
            </span>
          </div>
        </div>
      </td>

      <td style={{ width: "90px", fontFamily: "var(--font-mono)" }}>
        {job.state === "downloading" ? formatSpeed(currentSpeed) : "--"}
      </td>

      <td style={{ width: "80px", fontFamily: "var(--font-mono)" }}>
        {job.state === "downloading" ? formatEta(etaSeconds) : "--"}
      </td>

      <td style={{ width: "90px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "11px" }}>
          <NetworkSegmentsIcon size={13} />
          <span>
            {completedSegmentsCount}/{job.segments.length || job.thread_count}
          </span>
        </div>
      </td>
    </tr>
  );
};
