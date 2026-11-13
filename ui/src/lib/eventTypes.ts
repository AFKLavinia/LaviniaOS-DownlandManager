export type JobState =
  | "queued"
  | "probing"
  | "downloading"
  | "paused"
  | "finalizing"
  | "completed"
  | "failed"
  | "cancelled";

export type SegmentState =
  | "pending"
  | "connecting"
  | "downloading"
  | "completed"
  | "failed";

export interface Segment {
  id: { 0: number } | number;
  start_byte: number;
  current_byte: number;
  end_byte: number;
  state: SegmentState;
  speed: { 0: number } | number;
}

export interface DownloadJob {
  id: { 0: string } | string;
  url: string;
  destination_path: string;
  filename: string;
  total_bytes: number | null;
  etag: string | null;
  last_modified: string | null;
  supports_range: boolean;
  state: JobState;
  segments: Segment[];
  thread_count: number;
  error_message: string | null;
  created_at?: number;
}

/** Payload sent to the `add_download` command. */
export interface AddRequest {
  url: string;
  destination?: string;
  threads?: number;
  filename?: string;
  headers?: [string, string][];
  cookie?: string;
  referer?: string;
  user_agent?: string;
  username?: string;
  password?: string;
  paused?: boolean;
}

export interface Settings {
  download_dir: string;
  threads: number;
  speed_limit_kbps: number;
  max_concurrent: number;
  proxy: string;
  use_categories: boolean;
  clipboard_watch: boolean;
  close_to_tray: boolean;
  notifications: boolean;
  language: "tr" | "en";
  theme: "amoled" | "light";
  api_port: number;
  api_token: string;
}

export interface Notice {
  kind: "completed" | "failed";
  title: string;
  body: string;
}
