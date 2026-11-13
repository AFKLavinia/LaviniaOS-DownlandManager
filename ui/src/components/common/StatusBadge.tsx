import React from "react";
import type { JobState } from "../../lib/eventTypes";
import type { I18nKey } from "../../lib/i18n";
import { useT } from "../../lib/I18nContext";
import { CheckCircleIcon, AlertCircleIcon, PlayIcon, PauseIcon, ClockIcon } from "../../icons";

interface StatusBadgeProps {
  state: JobState;
}

const LABEL_KEYS: Record<JobState, I18nKey> = {
  queued: "stQueued",
  probing: "stProbing",
  downloading: "stDownloading",
  paused: "stPaused",
  finalizing: "stFinalizing",
  completed: "stCompleted",
  failed: "stFailed",
  cancelled: "stCancelled",
};

export const StatusBadge: React.FC<StatusBadgeProps> = ({ state }) => {
  const t = useT();

  const icon = (() => {
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
  })();

  return (
    <span className={`status-pill ${state}`}>
      {icon}
      <span>{t(LABEL_KEYS[state] ?? "stQueued")}</span>
    </span>
  );
};
