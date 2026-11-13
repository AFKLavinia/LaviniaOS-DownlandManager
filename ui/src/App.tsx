import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { AddRequest, DownloadJob, Notice } from "./lib/eventTypes";
import { Titlebar } from "./components/layout/Titlebar";
import { Toolbar } from "./components/layout/Toolbar";
import { Sidebar, type FilterCategory } from "./components/layout/Sidebar";
import { DownloadTable } from "./components/downloads/DownloadTable";
import { SegmentInspector } from "./components/inspector/SegmentInspector";
import { AddDownloadModal } from "./components/downloads/AddDownloadModal";
import { ConfirmRemoveModal } from "./components/downloads/ConfirmRemoveModal";
import { SettingsModal } from "./components/settings/SettingsModal";
import { ToastHost } from "./components/common/ToastHost";
import { ContextMenu, type MenuItem } from "./components/common/ContextMenu";
import { useDownloads } from "./hooks/useDownloads";
import { useSettings, type UseSettings } from "./hooks/useSettings";
import { useToasts } from "./hooks/useToasts";
import { useClipboardWatcher } from "./hooks/useClipboardWatcher";
import { I18nProvider, useT } from "./lib/I18nContext";
import { api, errorMessage } from "./lib/api";
import { extractUrl, jobIdOf, jobStats, isRunning } from "./lib/jobs";

const SettingsContext = createContext<UseSettings | null>(null);

interface MenuState {
  job: DownloadJob;
  x: number;
  y: number;
}

const Shell: React.FC = () => {
  const t = useT();
  const jobs = useDownloads();
  const { settings, restartRequired, update } = useContext(SettingsContext)!;
  const { toasts, push, dismiss } = useToasts();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterCategory>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [addOpen, setAddOpen] = useState(false);
  const [addInitialUrl, setAddInitialUrl] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<DownloadJob | null>(null);
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [dragging, setDragging] = useState(false);
  const searchRef = useRef<HTMLInputElement>(null);

  const knownUrls = useRef<Set<string>>(new Set());
  knownUrls.current = new Set(jobs.map((j) => j.url));

  // Keep a valid selection: pick the first job when none, clear when it vanished.
  useEffect(() => {
    if (selectedId && !jobs.some((j) => jobIdOf(j) === selectedId)) setSelectedId(null);
  }, [jobs, selectedId]);

  // Engine notices -> in-app toasts (native notifications are sent by the backend).
  useEffect(() => {
    let un: (() => void) | undefined;
    let disposed = false;
    api
      .onNotice((n: Notice) =>
        push(
          n.kind === "completed" ? "success" : "error",
          n.kind === "completed" ? t("noticeCompleted") : t("noticeFailed"),
          n.kind === "completed" ? n.body : `${n.title}: ${n.body}`,
        ),
      )
      .then((u) => (disposed ? u() : (un = u)))
      .catch(() => undefined);
    return () => {
      disposed = true;
      un?.();
    };
  }, [push, t]);

  const openAdd = useCallback((url = "") => {
    setAddInitialUrl(url);
    setAddOpen(true);
  }, []);

  useClipboardWatcher(
    settings.clipboard_watch,
    (url) => knownUrls.current.has(url),
    (url) => {
      if (addOpen || settingsOpen) return;
      push("info", t("clipboardFound"), url);
      openAdd(url);
    },
  );

  const totalSpeed = useMemo(
    () =>
      jobs
        .filter((j) => j.state === "downloading")
        .reduce((sum, j) => sum + jobStats(j).speed, 0),
    [jobs],
  );

  const filteredJobs = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    return jobs.filter((job) => {
      if (filter === "downloading") {
        if (!isRunning(job.state) && job.state !== "queued") return false;
      } else if (filter !== "all" && job.state !== filter) return false;
      return q ? job.filename.toLowerCase().includes(q) : true;
    });
  }, [jobs, filter, searchQuery]);

  const counts: Record<FilterCategory, number> = useMemo(
    () => ({
      all: jobs.length,
      downloading: jobs.filter((j) => isRunning(j.state) || j.state === "queued").length,
      completed: jobs.filter((j) => j.state === "completed").length,
      paused: jobs.filter((j) => j.state === "paused").length,
      failed: jobs.filter((j) => j.state === "failed").length,
    }),
    [jobs],
  );

  const selectedJob = jobs.find((j) => jobIdOf(j) === selectedId) ?? null;

  const run = useCallback(
    async (action: () => Promise<unknown>) => {
      try {
        await action();
      } catch (e) {
        push("error", t("tooltipError"), errorMessage(e));
      }
    },
    [push, t],
  );

  const handleAdd = useCallback(
    async (requests: AddRequest[]): Promise<string | null> => {
      for (const req of requests) {
        try {
          await api.add(req);
        } catch (e) {
          return `${t("addFailed")}: ${errorMessage(e)}`;
        }
      }
      return null;
    },
    [t],
  );

  const handleResume = useCallback(
    (job: DownloadJob | null = selectedJob) => job && run(() => api.resume(jobIdOf(job))),
    [run, selectedJob],
  );
  const handlePause = useCallback(
    (job: DownloadJob | null = selectedJob) => job && run(() => api.pause(jobIdOf(job))),
    [run, selectedJob],
  );
  const handleToggle = useCallback(
    (job: DownloadJob | null = selectedJob) => {
      if (!job) return;
      if (isRunning(job.state) || job.state === "queued") handlePause(job);
      else if (job.state !== "completed") handleResume(job);
    },
    [handlePause, handleResume, selectedJob],
  );
  const handleActivate = useCallback(
    (job: DownloadJob) => {
      if (job.state === "completed") run(() => api.openFile(jobIdOf(job)));
      else handleToggle(job);
    },
    [handleToggle, run],
  );
  const confirmRemove = useCallback(
    async (deleteFile: boolean) => {
      const job = removeTarget;
      setRemoveTarget(null);
      if (job) await run(() => api.remove(jobIdOf(job), deleteFile));
    },
    [removeTarget, run],
  );
  const openFolder = useCallback(
    (job: DownloadJob | null = selectedJob) =>
      run(() => api.openFolder(job ? jobIdOf(job) : undefined)),
    [run, selectedJob],
  );

  const toggleTheme = () =>
    update({ theme: settings.theme === "amoled" ? "light" : "amoled" }).then(
      (err) => err && push("error", t("tooltipError"), err),
    );

  const saveSettings = useCallback(
    async (patch: Parameters<typeof update>[0]) => {
      const err = await update(patch);
      if (!err) push("success", t("settingsSaved"));
      return err;
    },
    [push, t, update],
  );

  // Keyboard shortcuts
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      const typing = !!el && /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName);
      const modalOpen = addOpen || settingsOpen || removeTarget !== null;
      const ctrl = e.ctrlKey || e.metaKey;

      if (ctrl && e.key.toLowerCase() === "n") {
        e.preventDefault();
        if (!modalOpen) openAdd();
      } else if (ctrl && e.key === ",") {
        e.preventDefault();
        if (!modalOpen) setSettingsOpen(true);
      } else if (ctrl && e.key.toLowerCase() === "f") {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      } else if (!typing && !modalOpen && selectedJob) {
        if (e.key === "Delete") {
          e.preventDefault();
          setRemoveTarget(selectedJob);
        } else if (e.key === " ") {
          e.preventDefault();
          handleToggle(selectedJob);
        } else if (e.key === "Enter") {
          e.preventDefault();
          handleActivate(selectedJob);
        } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
          e.preventDefault();
          const idx = filteredJobs.findIndex((j) => jobIdOf(j) === selectedId);
          const next = filteredJobs[idx + (e.key === "ArrowDown" ? 1 : -1)];
          if (next) setSelectedId(jobIdOf(next));
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    addOpen,
    settingsOpen,
    removeTarget,
    selectedJob,
    selectedId,
    filteredJobs,
    openAdd,
    handleToggle,
    handleActivate,
  ]);

  // Drag & drop of links / text onto the window.
  useEffect(() => {
    const hasText = (e: DragEvent) =>
      Array.from(e.dataTransfer?.types ?? []).some((x) => x === "text/uri-list" || x === "text/plain");
    const over = (e: DragEvent) => {
      if (!hasText(e)) return;
      e.preventDefault();
      setDragging(true);
    };
    const leave = (e: DragEvent) => {
      if (e.relatedTarget === null) setDragging(false);
    };
    const drop = (e: DragEvent) => {
      if (!hasText(e)) return;
      e.preventDefault();
      setDragging(false);
      const text = e.dataTransfer?.getData("text/uri-list") || e.dataTransfer?.getData("text/plain");
      const url = extractUrl(text);
      if (url) openAdd(url);
    };
    window.addEventListener("dragover", over);
    window.addEventListener("dragleave", leave);
    window.addEventListener("drop", drop);
    return () => {
      window.removeEventListener("dragover", over);
      window.removeEventListener("dragleave", leave);
      window.removeEventListener("drop", drop);
    };
  }, [openAdd]);

  const menuItems = (job: DownloadJob): MenuItem[] => {
    const done = job.state === "completed";
    const running = isRunning(job.state) || job.state === "queued";
    return [
      { label: t("openFile"), onClick: () => run(() => api.openFile(jobIdOf(job))), disabled: !done },
      { label: t("openFolder"), onClick: () => openFolder(job) },
      {
        label: job.state === "failed" ? t("ctxRetry") : t("resume"),
        onClick: () => handleResume(job),
        disabled: done || running,
        separatorBefore: true,
      },
      { label: t("pause"), onClick: () => handlePause(job), disabled: !running },
      {
        label: t("ctxCopyUrl"),
        onClick: () => void navigator.clipboard.writeText(job.url).catch(() => undefined),
        separatorBefore: true,
      },
      { label: t("remove"), onClick: () => setRemoveTarget(job), danger: true, separatorBefore: true },
    ];
  };

  return (
    <div className="app-container">
      <Titlebar
        totalSpeed={totalSpeed}
        theme={settings.theme}
        onToggleTheme={toggleTheme}
        onOpenSettings={() => setSettingsOpen(true)}
      />

      <Toolbar
        onAddNew={() => openAdd()}
        onResume={() => handleResume()}
        onPause={() => handlePause()}
        onDelete={() => selectedJob && setRemoveTarget(selectedJob)}
        onOpenFolder={() => openFolder()}
        selectedId={selectedId}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        searchRef={searchRef}
      />

      <div className="main-body">
        <Sidebar
          currentFilter={filter}
          onFilterChange={setFilter}
          counts={counts}
          onResumeAll={() => run(api.resumeAll)}
          onPauseAll={() => run(api.pauseAll)}
          onClearCompleted={() => run(api.clearCompleted)}
        />

        <main className="content-area">
          <DownloadTable
            jobs={filteredJobs}
            filtered={jobs.length > 0}
            selectedId={selectedId}
            onSelect={setSelectedId}
            onActivate={handleActivate}
            onContextMenu={(job, x, y) => setMenu({ job, x, y })}
          />
          <SegmentInspector job={selectedJob} />
        </main>
      </div>

      <AddDownloadModal
        isOpen={addOpen}
        initialUrl={addInitialUrl}
        defaultDir={settings.download_dir}
        defaultThreads={settings.threads}
        onClose={() => setAddOpen(false)}
        onAdd={handleAdd}
      />

      <SettingsModal
        isOpen={settingsOpen}
        settings={settings}
        onClose={() => setSettingsOpen(false)}
        onSave={saveSettings}
      />

      <ConfirmRemoveModal
        job={removeTarget}
        onCancel={() => setRemoveTarget(null)}
        onConfirm={confirmRemove}
      />

      {menu && (
        <ContextMenu
          x={menu.x}
          y={menu.y}
          items={menuItems(menu.job)}
          onClose={() => setMenu(null)}
        />
      )}

      {restartRequired && <div className="restart-banner">{t("restartRequired")}</div>}
      {dragging && <div className="drop-overlay">{t("dropHint")}</div>}
      <ToastHost toasts={toasts} onDismiss={dismiss} />
    </div>
  );
};

export const App: React.FC = () => {
  // Language lives in settings; Shell reads it through the provider below.
  const settingsState = useSettings();
  return (
    <SettingsContext.Provider value={settingsState}>
      <I18nProvider lang={settingsState.settings.language}>
        <Shell />
      </I18nProvider>
    </SettingsContext.Provider>
  );
};

export default App;
