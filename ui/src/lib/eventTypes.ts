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
}
