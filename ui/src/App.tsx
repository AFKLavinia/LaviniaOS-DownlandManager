import React, { useState } from "react";
import type { DownloadJob } from "./lib/eventTypes";
import { Header } from "./components/layout/Header";
import { Toolbar } from "./components/layout/Toolbar";
import { Sidebar, type FilterCategory } from "./components/layout/Sidebar";
import { DownloadTable } from "./components/downloads/DownloadTable";
import { SegmentInspector } from "./components/inspector/SegmentInspector";
import { AddDownloadModal } from "./components/downloads/AddDownloadModal";

export const App: React.FC = () => {
  const [jobs, setJobs] = useState<DownloadJob[]>([
    {
      id: "job-1",
      url: "https://releases.ubuntu.com/24.04/ubuntu-24.04-desktop-amd64.iso",
      destination_path: "C:\\Downloads",
      filename: "ubuntu-24.04-desktop-amd64.iso",
      total_bytes: 6100000000,
      etag: '"6627a69b-16bb9e000"',
      last_modified: "Tue, 23 Apr 2024 12:00:00 GMT",
      supports_range: true,
      state: "downloading",
      thread_count: 8,
      error_message: null,
      segments: [
        { id: 0, start_byte: 0, current_byte: 650000000, end_byte: 762500000, state: "downloading", speed: 5240000 },
        { id: 1, start_byte: 762500001, current_byte: 1400000000, end_byte: 1525000000, state: "downloading", speed: 4890000 },
        { id: 2, start_byte: 1525000001, current_byte: 2287500000, end_byte: 2287500000, state: "completed", speed: 0 },
        { id: 3, start_byte: 2287500001, current_byte: 2900000000, end_byte: 3050000000, state: "downloading", speed: 6100000 },
        { id: 4, start_byte: 3050000001, current_byte: 3600000000, end_byte: 3812500000, state: "downloading", speed: 3800000 },
        { id: 5, start_byte: 3812500001, current_byte: 4575000001, end_byte: 4575000000, state: "completed", speed: 0 },
        { id: 6, start_byte: 4575000001, current_byte: 5100000000, end_byte: 5337500000, state: "downloading", speed: 4200000 },
        { id: 7, start_byte: 5337500001, current_byte: 5800000000, end_byte: 6100000000, state: "downloading", speed: 5100000 },
      ],
    },
    {
      id: "job-2",
      url: "https://nodejs.org/dist/v20.12.2/node-v20.12.2-x64.msi",
      destination_path: "C:\\Downloads",
      filename: "node-v20.12.2-x64.msi",
      total_bytes: 32500000,
      etag: '"6616a9a0-1efc3e8"',
      last_modified: "Wed, 10 Apr 2024 15:00:00 GMT",
      supports_range: true,
      state: "completed",
      thread_count: 4,
      error_message: null,
      segments: [
        { id: 0, start_byte: 0, current_byte: 8125001, end_byte: 8125000, state: "completed", speed: 0 },
        { id: 1, start_byte: 8125001, current_byte: 16250001, end_byte: 16250000, state: "completed", speed: 0 },
        { id: 2, start_byte: 16250001, current_byte: 24375001, end_byte: 24375000, state: "completed", speed: 0 },
        { id: 3, start_byte: 24375001, current_byte: 32500001, end_byte: 32500000, state: "completed", speed: 0 },
      ],
    },
    {
      id: "job-3",
      url: "https://download.visualstudio.microsoft.com/download/pr/dotnet-sdk-8.0.exe",
      destination_path: "C:\\Downloads",
      filename: "dotnet-sdk-8.0.exe",
      total_bytes: 220000000,
      etag: '"8a7b9c1d-d24000"',
      last_modified: "Mon, 18 Mar 2024 10:00:00 GMT",
      supports_range: true,
      state: "paused",
      thread_count: 8,
      error_message: null,
      segments: [
        { id: 0, start_byte: 0, current_byte: 27500000, end_byte: 27500000, state: "completed", speed: 0 },
        { id: 1, start_byte: 27500001, current_byte: 35000000, end_byte: 55000000, state: "pending", speed: 0 },
      ],
    },
  ]);

  const [selectedId, setSelectedId] = useState<string | null>("job-1");
  const [filter, setFilter] = useState<FilterCategory>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [isModalOpen, setIsModalOpen] = useState(false);

  // Compute total speed across all active downloads
  const totalSpeed = jobs
    .filter((j) => j.state === "downloading")
    .flatMap((j) => j.segments)
    .reduce((sum, seg) => {
      const spd = typeof seg.speed === "object" ? seg.speed[0] : seg.speed;
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
    const id = typeof j.id === "object" ? j.id[0] : j.id;
    return id === selectedId;
  }) || null;

  // Actions
  const handleAddNew = (url: string, destination: string, threads: number) => {
    const filename = url.split("/").filter(Boolean).pop()?.split("?")[0] || "download.bin";
    const newJob: DownloadJob = {
      id: `job-${Date.now()}`,
      url,
      destination_path: destination,
      filename,
      total_bytes: 150000000,
      etag: null,
      last_modified: null,
      supports_range: true,
      state: "downloading",
      thread_count: threads,
      error_message: null,
      segments: Array.from({ length: threads }).map((_, idx) => ({
        id: idx,
        start_byte: idx * 18750000,
        current_byte: idx * 18750000,
        end_byte: (idx + 1) * 18750000 - 1,
        state: "downloading",
        speed: 2500000,
      })),
    };

    setJobs((prev) => [newJob, ...prev]);
    setSelectedId(typeof newJob.id === "object" ? newJob.id[0] : newJob.id);
  };

  const handlePause = () => {
    if (!selectedId) return;
    setJobs((prev) =>
      prev.map((j) => {
        const id = typeof j.id === "object" ? j.id[0] : j.id;
        return id === selectedId ? { ...j, state: "paused" } : j;
      })
    );
  };

  const handleResume = () => {
    if (!selectedId) return;
    setJobs((prev) =>
      prev.map((j) => {
        const id = typeof j.id === "object" ? j.id[0] : j.id;
        return id === selectedId ? { ...j, state: "downloading" } : j;
      })
    );
  };

  const handleDelete = () => {
    if (!selectedId) return;
    setJobs((prev) => {
      const updated = prev.filter((j) => {
        const id = typeof j.id === "object" ? j.id[0] : j.id;
        return id !== selectedId;
      });
      return updated;
    });
    setSelectedId(null);
  };

  return (
    <div className="app-container">
      <Header totalSpeed={totalSpeed} onOpenSettings={() => {}} />

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
