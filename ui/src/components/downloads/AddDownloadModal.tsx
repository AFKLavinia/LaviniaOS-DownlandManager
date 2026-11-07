import React, { useState } from "react";
import { Modal } from "../common/Modal";
import { Button } from "../common/Button";
import { DownloadIcon } from "../../icons";

interface AddDownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (url: string, destination: string, threads: number) => void;
}

export const AddDownloadModal: React.FC<AddDownloadModalProps> = ({
  isOpen,
  onClose,
  onAdd,
}) => {
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState("./downloads");
  const [threads, setThreads] = useState(8);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;
    onAdd(url.trim(), destination.trim(), Number(threads));
    setUrl("");
    onClose();
  };

  return (
    <Modal
      isOpen={isOpen}
      title="Yeni İndirme Ekle"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>İptal</Button>
          <Button variant="primary" icon={<DownloadIcon size={14} />} onClick={handleSubmit}>
            İndirmeyi Başlat
          </Button>
        </>
      }
    >
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        <div>
          <label style={{ display: "block", marginBottom: "4px", fontSize: "11px", color: "var(--text-secondary)" }}>
            İndirme Bağlantısı (URL):
          </label>
          <input
            type="url"
            required
            className="input-field"
            placeholder="https://example.com/file.zip"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            autoFocus
          />
        </div>

        <div>
          <label style={{ display: "block", marginBottom: "4px", fontSize: "11px", color: "var(--text-secondary)" }}>
            Kayıt Dizini:
          </label>
          <input
            type="text"
            className="input-field"
            value={destination}
            onChange={(e) => setDestination(e.target.value)}
          />
        </div>

        <div>
          <label style={{ display: "block", marginBottom: "4px", fontSize: "11px", color: "var(--text-secondary)" }}>
            Eşzamanlı Bağlantı Parçası (Threads):
          </label>
          <select
            className="input-field"
            value={threads}
            onChange={(e) => setThreads(Number(e.target.value))}
            style={{ cursor: "pointer" }}
          >
            <option value={1}>1 Bağlantı (Tek Parça)</option>
            <option value={4}>4 Bağlantı</option>
            <option value={8}>8 Bağlantı (Önerilen)</option>
            <option value={16}>16 Bağlantı (Yüksek Hız)</option>
            <option value={32}>32 Bağlantı (Maksimum)</option>
          </select>
        </div>
      </form>
    </Modal>
  );
};
