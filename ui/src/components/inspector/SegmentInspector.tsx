import React from "react";
import type { DownloadJob } from "../../lib/eventTypes";
import { formatBytes, formatSpeed } from "../../lib/formatters";
import { NetworkSegmentsIcon } from "../../icons";

interface SegmentInspectorProps {
  job: DownloadJob | null;
}

export const SegmentInspector: React.FC<SegmentInspectorProps> = ({ job }) => {
  if (!job) {
    return (
      <div className="inspector-drawer">
        <div className="inspector-header">
          <span>Parça Ayrıntıları (Segment Inspector)</span>
        </div>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            height: "100%",
            color: "var(--text-muted)",
            fontSize: "12px",
          }}
        >
          Ayrıntılı bağlantı ve soket dökümünü görmek için yukarıdan bir indirme seçin.
        </div>
      </div>
    );
  }

  return (
    <div className="inspector-drawer">
      <div className="inspector-header">
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <NetworkSegmentsIcon size={14} />
          <span>
            {job.filename} — {job.segments.length} Aktif Bağlantı Parçası
          </span>
        </div>
        <span style={{ fontFamily: "var(--font-mono)", fontSize: "11px" }}>
          Ofset Aralığı: 0 - {formatBytes(job.total_bytes)}
        </span>
      </div>

      <div className="inspector-grid">
        {job.segments.map((seg, idx) => {
          const segId = typeof seg.id === "object" ? seg.id[0] : seg.id;
          const segTotal = Math.max(0, seg.end_byte - seg.start_byte + 1);
          const downloaded = Math.max(0, seg.current_byte - seg.start_byte);
          const ratio = segTotal > 0 ? (downloaded / segTotal) * 100 : 0;
          const spd = typeof seg.speed === "object" ? seg.speed[0] : seg.speed;

          return (
            <div key={idx} className="segment-card">
              <div className="segment-card-header">
                <span>Bağlantı #{segId + 1}</span>
                <span style={{ color: seg.state === "completed" ? "var(--accent-emerald)" : "var(--accent-sky)" }}>
                  {seg.state}
                </span>
              </div>

              <div
                style={{
                  height: "4px",
                  backgroundColor: "var(--segment-track)",
                  borderRadius: "2px",
                  overflow: "hidden",
                  margin: "2px 0",
                }}
              >
                <div
                  style={{
                    height: "100%",
                    width: `${ratio}%`,
                    backgroundColor:
                      seg.state === "completed" ? "var(--accent-emerald)" : "var(--accent-sky)",
                  }}
                />
              </div>

              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  fontSize: "10px",
                  fontFamily: "var(--font-mono)",
                  color: "var(--text-muted)",
                }}
              >
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
