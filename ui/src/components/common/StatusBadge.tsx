import React from "react";
import type { JobState } from "../../lib/eventTypes";
import { CheckCircleIcon, AlertCircleIcon, PlayIcon, PauseIcon, ClockIcon } from "../../icons";

interface StatusBadgeProps {
  state: JobState;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ state }) => {
  const getIcon = () => {
    switch (state) {
      case "downloading":
        return <PlayIcon size={12} strokeWidth={2.5} />;
      case "completed":
        return <CheckCircleIcon size={12} strokeWidth={2.5} />;
      case "paused":
        return <PauseIcon size={12} strokeWidth={2.5} />;
      case "failed":
        return <AlertCircleIcon size={12} strokeWidth={2.5} />;
      default:
        return <ClockIcon size={12} strokeWidth={2.5} />;
    }
  };

  const labels: Record<JobState, string> = {
    queued: "Sırada",
    probing: "Taranıyor",
    downloading: "İndiriliyor",
    paused: "Duraklatıldı",
    finalizing: "Tamamlanıyor",
    completed: "Tamamlandı",
    failed: "Hata",
    cancelled: "İptal",
  };

  return (
    <span className={`status-pill ${state}`}>
      {getIcon()}
      <span>{labels[state] || state}</span>
    </span>
  );
};
