import React, { useState, useEffect } from "react";
import type { DownloadJob } from "./lib/eventTypes";
import { Titlebar } from "./components/layout/Titlebar";
import { Toolbar } from "./components/layout/Toolbar";
import { Sidebar, type FilterCategory } from "./components/layout/Sidebar";
import { DownloadTable } from "./components/downloads/DownloadTable";
import { SegmentInspector } from "./components/inspector/SegmentInspector";
import { AddDownloadModal } from "./components/downloads/AddDownloadModal";

const API_BASE = "http://127.0.0.1:4141";

export const App: React.FC = () => {
  const [jobs, setJobs] = useState<DownloadJob[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterCategory>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Poll real-time raw data downloads from the Rust Core Engine every 350ms
  useEffect(() => {
    let isMounted = true;
    const fetchJobs = async () => {
      try {
        const res = await fetch(`${API_BASE}/api/downloads`);
        if (res.ok) {
          const data: DownloadJob[] = await res.json();
          if (isMounted && Array.isArray(data)) {
            setJobs(data);
            if (!selectedId && data.length > 0) {
              const firstId = typeof data[0].id === "object" ? (data[0].id as any)[0] : data[0].id;
              setSelectedId(firstId);
            }
          }
        }
      } catch {
        // Backend connecting
      }
    };

    fetchJobs();
    const interval = setInterval(fetchJobs, 350);
    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, [selectedId]);

  // Compute total speed across all active downloads
  const totalSpeed = jobs
    .filter((j) => j.state === "downloading")
    .flatMap((j) => j.segments)
    .reduce((sum, seg) => {
      const spd = typeof seg.speed === "object" ? (seg.speed as any)[0] : seg.speed;
      return sum + (spd || 0);
    }, 0);

  // Filter jobs
  const filteredJobs = jobs.filter((job) => {
    if (filter !== "all" && job.state !== filter) return false;
    if (searchQuery.trim()) {
      return job.filename.toLowerCase().includes(searchQuery.toLowerCase());
    }
    return true;
  });

  // Calculate filter counts
  const counts: Record<FilterCategory, number> = {
    all: jobs.length,
    downloading: jobs.filter((j) => j.state === "downloading").length,
    completed: jobs.filter((j) => j.state === "completed").length,
    paused: jobs.filter((j) => j.state === "paused").length,
    failed: jobs.filter((j) => j.state === "failed").length,
  };

  const selectedJob = jobs.find((j) => {
    const id = typeof j.id === "object" ? (j.id as any)[0] : j.id;
    return id === selectedId;
  }) || null;

  // Actions connecting directly to raw data Rust core
  const handleAddNew = async (url: string, destination: string, threads: number) => {
    try {
      await fetch(`${API_BASE}/api/downloads`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url, destination, threads }),
      });
    } catch (e) {
      console.error("Failed to add download to Rust core:", e);
    }
  };

  const handlePause = async () => {
    if (!selectedId) return;
    try {
      await fetch(`${API_BASE}/api/downloads/pause`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: selectedId }),
      });
    } catch (e) {
      console.error(e);
    }
  };

  const handleResume = async () => {
    if (!selectedId) return;
    try {
      await fetch(`${API_BASE}/api/downloads/resume`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: selectedId }),
      });
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = () => {
    if (!selectedId) return;
    handlePause();
    setJobs((prev) => {
      const updated = prev.filter((j) => {
        const id = typeof j.id === "object" ? (j.id as any)[0] : j.id;
        return id !== selectedId;
      });
      return updated;
    });
    setSelectedId(null);
  };

  return (
    <div className="app-container">
      <Titlebar totalSpeed={totalSpeed} onOpenSettings={() => {}} />

      <Toolbar
        onAddNew={() => setIsModalOpen(true)}
        onResume={handleResume}
        onPause={handlePause}
        onDelete={handleDelete}
        onOpenFolder={() => {}}
        selectedId={selectedId}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
      />

      <div className="main-body">
        <Sidebar currentFilter={filter} onFilterChange={setFilter} counts={counts} />

        <main className="content-area">
          <DownloadTable
            jobs={filteredJobs}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
          <SegmentInspector job={selectedJob} />
        </main>
      </div>

      <AddDownloadModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onAdd={handleAddNew}
      />
    </div>
  );
};

export default App;
