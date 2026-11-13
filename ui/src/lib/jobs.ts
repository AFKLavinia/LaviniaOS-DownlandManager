import type { DownloadJob, JobState } from "./eventTypes";

/** Rust newtypes may serialise as `{0: x}` or plain `x`; normalise both. */
export function unwrap<T>(v: { 0: T } | T): T {
  return typeof v === "object" && v !== null && 0 in (v as object)
    ? (v as { 0: T })[0]
    : (v as T);
}

export const jobIdOf = (job: DownloadJob): string => unwrap<string>(job.id);

export const isRunning = (s: JobState): boolean =>
  s === "downloading" || s === "probing" || s === "finalizing";

export interface JobStats {
  downloaded: number;
  total: number;
  percentage: number;
  speed: number;
  etaSeconds: number | null;
  completedSegments: number;
}

export function jobStats(job: DownloadJob): JobStats {
  const total = job.total_bytes ?? 0;
  const downloaded = job.segments.reduce(
    (acc, s) => acc + Math.max(0, Math.min(s.current_byte, s.end_byte + 1) - s.start_byte),
    0,
  );
  const speed = job.segments.reduce((acc, s) => acc + (unwrap<number>(s.speed) || 0), 0);
  const done = job.state === "completed";
  const percentage = done ? 100 : total > 0 ? Math.min(100, (downloaded / total) * 100) : 0;
  const remaining = Math.max(0, total - downloaded);
  return {
    downloaded: done && total > 0 ? total : downloaded,
    total,
    percentage,
    speed,
    etaSeconds: speed > 0 && total > 0 ? remaining / speed : null,
    completedSegments: job.segments.filter((s) => s.state === "completed").length,
  };
}

const NON_FILE_EXT = new Set(["html", "htm", "php", "asp", "aspx", "jsp", "cgi", "xhtml"]);

/** First http(s) URL found in a text blob, or null. */
export function extractUrl(text: string | null | undefined): string | null {
  if (!text) return null;
  const m = text.match(/https?:\/\/[^\s<>"']+/i);
  if (!m) return null;
  try {
    return new URL(m[0]).toString();
  } catch {
    return null;
  }
}

/** All distinct http(s) URLs in a text blob (one per line / whitespace separated). */
export function extractUrls(text: string): string[] {
  const found = text.match(/https?:\/\/[^\s<>"']+/gi) ?? [];
  const out: string[] = [];
  for (const raw of found) {
    try {
      const u = new URL(raw).toString();
      if (!out.includes(u)) out.push(u);
    } catch {
      /* ignore malformed */
    }
  }
  return out;
}

/** Heuristic: does this URL look like a direct file download (vs a web page)? */
export function looksDownloadable(url: string): boolean {
  try {
    const last = new URL(url).pathname.split("/").filter(Boolean).pop() ?? "";
    const dot = last.lastIndexOf(".");
    if (dot <= 0) return false;
    const ext = last.slice(dot + 1).toLowerCase();
    return ext.length >= 2 && ext.length <= 5 && !NON_FILE_EXT.has(ext);
  } catch {
    return false;
  }
}

/** Parse `Name: value` lines into header tuples, skipping malformed lines. */
export function parseHeaderLines(text: string): [string, string][] {
  return text
    .split(/\r?\n/)
    .map((line) => {
      const i = line.indexOf(":");
      if (i <= 0) return null;
      const name = line.slice(0, i).trim();
      const value = line.slice(i + 1).trim();
      return name ? ([name, value] as [string, string]) : null;
    })
    .filter((h): h is [string, string] => h !== null);
}
