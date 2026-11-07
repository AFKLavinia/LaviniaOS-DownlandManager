import React from "react";
import type { Segment } from "../../lib/eventTypes";

interface SegmentProgressProps {
  segments: Segment[];
  totalBytes: number | null;
  percentage: number;
}

export const SegmentProgress: React.FC<SegmentProgressProps> = ({
  segments,
  totalBytes,
  percentage,
}) => {
  // If no segments or unknown total size, render standard smooth progress bar
  if (!totalBytes || totalBytes <= 0 || segments.length === 0) {
    return (
      <div className="segment-strip-container" title={`${percentage.toFixed(1)}%`}>
        <div
          className="segment-slice"
          style={{
            left: 0,
            width: `${percentage}%`,
            backgroundColor: "var(--accent-sky)",
          }}
        />
      </div>
    );
  }

  return (
    <div
      className="segment-strip-container"
      title={`${percentage.toFixed(1)}% (${segments.length} Parça)`}
    >
      {segments.map((seg, idx) => {
        const downloaded = Math.max(0, seg.current_byte - seg.start_byte);
        const leftPercent = (seg.start_byte / totalBytes) * 100;
        const widthPercent = (downloaded / totalBytes) * 100;

        let color = "var(--segment-filled)";
        if (seg.state === "completed") {
          color = "var(--accent-emerald)";
        } else if (seg.state === "connecting") {
          color = "var(--segment-connecting)";
        }

        return (
          <div
            key={idx}
            className="segment-slice"
            style={{
              left: `${leftPercent}%`,
              width: `${Math.max(0.2, widthPercent)}%`,
              backgroundColor: color,
            }}
          />
        );
      })}
    </div>
  );
};
