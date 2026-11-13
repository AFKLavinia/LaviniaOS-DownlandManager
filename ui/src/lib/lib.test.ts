import { describe, expect, it } from "vitest";
import { formatBytes, formatEta, formatSpeed } from "./formatters";
import {
  extractUrl,
  extractUrls,
  jobStats,
  looksDownloadable,
  parseHeaderLines,
  unwrap,
} from "./jobs";
import { dictionaries, translate, type I18nKey } from "./i18n";
import type { DownloadJob } from "./eventTypes";

function job(partial: Partial<DownloadJob>): DownloadJob {
  return {
    id: "1",
    url: "http://x/f.bin",
    destination_path: ".",
    filename: "f.bin",
    total_bytes: 1000,
    etag: null,
    last_modified: null,
    supports_range: true,
    state: "downloading",
    segments: [],
    thread_count: 2,
    error_message: null,
    ...partial,
  };
}

describe("formatters", () => {
  it("formats bytes", () => {
    expect(formatBytes(null)).toBe("--");
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });
  it("formats speed and eta", () => {
    expect(formatSpeed(0)).toBe("0 B/s");
    expect(formatSpeed(2048)).toBe("2.0 KB/s");
    expect(formatEta(null)).toBe("--");
    expect(formatEta(75)).toBe("01:15");
    expect(formatEta(3725)).toBe("1h 02m");
  });
});

describe("jobs helpers", () => {
  it("unwraps newtype wrappers", () => {
    expect(unwrap<number>({ 0: 5 })).toBe(5);
    expect(unwrap<number>(7)).toBe(7);
  });

  it("computes stats from segments", () => {
    const j = job({
      segments: [
        { id: 0, start_byte: 0, current_byte: 250, end_byte: 499, state: "downloading", speed: 100 },
        { id: 1, start_byte: 500, current_byte: 750, end_byte: 999, state: "downloading", speed: { 0: 50 } },
      ],
    });
    const s = jobStats(j);
    expect(s.downloaded).toBe(500);
    expect(s.percentage).toBe(50);
    expect(s.speed).toBe(150);
    expect(s.etaSeconds).toBeCloseTo(500 / 150);
  });

  it("completed jobs are always 100%", () => {
    expect(jobStats(job({ state: "completed", segments: [] })).percentage).toBe(100);
  });

  it("does not exceed 100% on overshoot", () => {
    const j = job({
      segments: [{ id: 0, start_byte: 0, current_byte: 5000, end_byte: 999, state: "completed", speed: 0 }],
    });
    expect(jobStats(j).percentage).toBe(100);
  });

  it("extracts urls", () => {
    expect(extractUrl("see https://a.com/x.zip now")).toBe("https://a.com/x.zip");
    expect(extractUrl("no link")).toBeNull();
    expect(extractUrls("http://a.com/1.zip\nhttps://b.com/2.zip\nhttp://a.com/1.zip")).toEqual([
      "http://a.com/1.zip",
      "https://b.com/2.zip",
    ]);
  });

  it("detects direct file links", () => {
    expect(looksDownloadable("https://a.com/files/setup.exe?token=1")).toBe(true);
    expect(looksDownloadable("https://a.com/index.html")).toBe(false);
    expect(looksDownloadable("https://a.com/watch")).toBe(false);
  });

  it("parses header lines", () => {
    expect(parseHeaderLines("X-A: 1\nbad line\n  Y-B :  two:parts ")).toEqual([
      ["X-A", "1"],
      ["Y-B", "two:parts"],
    ]);
  });
});

describe("i18n", () => {
  it("tr and en define exactly the same keys", () => {
    const tr = Object.keys(dictionaries.tr).sort();
    const en = Object.keys(dictionaries.en).sort();
    expect(en).toEqual(tr);
  });
  it("has no empty strings", () => {
    for (const lang of ["tr", "en"] as const) {
      for (const [k, v] of Object.entries(dictionaries[lang])) {
        expect(v.trim().length, `${lang}.${k}`).toBeGreaterThan(0);
      }
    }
  });
  it("substitutes params and falls back to key", () => {
    expect(translate("en", "resume")).toBe("Resume");
    expect(translate("tr", "resume")).toBe("Devam Et");
    expect(translate("en", "nonexistent" as I18nKey)).toBe("nonexistent");
  });
});
