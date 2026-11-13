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
  // Unknown size / not yet planned: simple bar.
  if (!totalBytes || totalBytes <= 0 || segments.length === 0) {
    return (
      <div className="segment-strip-container" title={`${percentage.toFixed(1)}%`}>
        <div
          className="segment-slice"
          style={{ left: 0, width: `${percentage}%`, backgroundColor: "var(--segment-filled)" }}
        />
      </div>
    );
  }

  return (
    <div className="segment-strip-container" title={`${percentage.toFixed(1)}%`}>
      {segments.map((seg, idx) => {
        const downloaded = Math.max(0, Math.min(seg.current_byte, seg.end_byte + 1) - seg.start_byte);
        const left = (seg.start_byte / totalBytes) * 100;
        const width = (downloaded / totalBytes) * 100;
        const color =
          seg.state === "connecting" ? "var(--segment-connecting)" : "var(--segment-filled)";
        return (
          <div
            key={idx}
            className="segment-slice"
            style={{ left: `${left}%`, width: `${width}%`, backgroundColor: color }}
          />
        );
      })}
    </div>
  );
};
