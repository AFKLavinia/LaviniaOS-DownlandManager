import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AddRequest, DownloadJob, Notice, Settings } from "./eventTypes";

export interface ApiInfo {
  port: number;
  token: string;
}

export interface SaveSettingsResult {
  settings: Settings;
  restart_required: boolean;
}

/** Thin typed wrapper over the Tauri commands exposed by src-tauri. */
export const api = {
  getDownloads: () => invoke<DownloadJob[]>("get_downloads"),
  add: (req: AddRequest) => invoke<string>("add_download", { req }),
  pause: (id: string) => invoke<boolean>("pause_download", { id }),
  resume: (id: string) => invoke<void>("resume_download", { id }),
  remove: (id: string, deleteFile: boolean) =>
    invoke<void>("remove_download", { id, deleteFile }),
  pauseAll: () => invoke<void>("pause_all"),
  resumeAll: () => invoke<void>("resume_all"),
  clearCompleted: () => invoke<void>("clear_completed"),
  openFile: (id: string) => invoke<void>("open_file", { id }),
  openFolder: (id?: string) => invoke<void>("open_folder", { id: id ?? null }),
  pickFolder: () => invoke<string | null>("pick_folder"),
  getClipboard: () => invoke<string | null>("get_clipboard_text"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (newSettings: Settings) =>
    invoke<SaveSettingsResult>("save_settings", { newSettings }),
  getApiInfo: () => invoke<ApiInfo>("get_api_info"),
  minimize: () => invoke<void>("win_minimize"),
  maximize: () => invoke<void>("win_maximize"),
  close: () => invoke<void>("win_close"),

  onDownloads: (cb: (jobs: DownloadJob[]) => void): Promise<UnlistenFn> =>
    listen<DownloadJob[]>("lodm://downloads", (e) => cb(e.payload)),
  onNotice: (cb: (n: Notice) => void): Promise<UnlistenFn> =>
    listen<Notice>("lodm://notice", (e) => cb(e.payload)),
};

/** Normalise an unknown thrown value into a readable message. */
export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}
