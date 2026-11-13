import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { DownloadJob } from "../lib/eventTypes";

/** Live download list: initial fetch + backend push events (no polling). */
export function useDownloads(): DownloadJob[] {
  const [jobs, setJobs] = useState<DownloadJob[]>([]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    api
      .getDownloads()
      .then((list) => !disposed && setJobs(list))
      .catch(() => undefined);

    api
      .onDownloads((list) => !disposed && setJobs(list))
      .then((un) => {
        if (disposed) un();
        else unlisten = un;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return jobs;
}
