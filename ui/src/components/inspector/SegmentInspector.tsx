import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { formatBytes, formatSpeed } from "../../lib/formatters";
import { unwrap } from "../../lib/jobs";
import { NetworkSegmentsIcon } from "../../icons";
import { useT } from "../../lib/I18nContext";

interface SegmentInspectorProps {
  job: DownloadJob | null;
}

export const SegmentInspector: React.FC<SegmentInspectorProps> = ({ job }) => {
  const t = useT();

  if (!job) {
    return (
      <div className="inspector-drawer">
        <div className="inspector-header">
          <span>{t("inspectorTitle")}</span>
        </div>
        <div className="inspector-empty">{t("inspectorEmpty")}</div>
      </div>
    );
  }

  return (
    <div className="inspector-drawer">
      <div className="inspector-header">
        <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
          <NetworkSegmentsIcon size={13} />
          <span className="cell-name">
            {job.filename} - {job.segments.length} {t("segmentsCount")}
          </span>
        </div>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "10px" }}>
          {formatBytes(job.total_bytes)}
        </span>
      </div>

      <div className="inspector-grid">
        {job.segments.map((seg, idx) => {
          const segId = unwrap<number>(seg.id);
          const segTotal = Math.max(0, seg.end_byte - seg.start_byte + 1);
          const downloaded = Math.max(0, Math.min(seg.current_byte, seg.end_byte + 1) - seg.start_byte);
          const ratio = segTotal > 0 ? Math.min(100, (downloaded / segTotal) * 100) : 0;
          const spd = unwrap<number>(seg.speed);

          return (
            <div key={idx} className="segment-card">
              <div className="segment-card-header">
                <span>
                  {t("connection")} #{segId + 1}
                </span>
                <span>{seg.state}</span>
              </div>
              <div className="segment-bar">
                <div className="segment-bar-fill" style={{ width: `${ratio}%` }} />
              </div>
              <div className="cell-sub">
                <span>{ratio.toFixed(0)}%</span>
                <span>{formatBytes(downloaded)}</span>
                <span>{formatSpeed(spd)}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
