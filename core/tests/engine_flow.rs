use lavinia_download_core::bridge::{ApiConfig, DownloadManagerApi, NewDownload};
use lavinia_download_core::domain::state::JobState;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Deterministic payload so corruption is detectable.
fn payload(len: usize) -> Arc<Vec<u8>> {
    Arc::new((0..len).map(|i| (i % 251) as u8).collect())
}

/// Minimal range-capable HTTP server. `delay_ms` throttles each 16 KiB write.
async fn spawn_server(data: Arc<Vec<u8>>, delay_ms: u64) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else { return };
            let data = data.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let is_head = req.starts_with("HEAD");
                let range = req
                    .lines()
                    .find(|l| l.to_ascii_lowercase().starts_with("range:"))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|r| r.trim().to_string());

                let total = data.len();
                let (status, start, end) = match range {
                    Some(r) => {
                        let mut it = r.split('-');
                        let s: usize = it.next().unwrap().parse().unwrap();
                        let e: usize = it
                            .next()
                            .and_then(|x| x.parse().ok())
                            .unwrap_or(total - 1)
                            .min(total - 1);
                        ("206 Partial Content", s, e)
                    }
                    None => ("200 OK", 0, total - 1),
                };
                let len = end + 1 - start;
                let mut head = format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nContent-Disposition: attachment; filename=\"served.bin\"\r\nConnection: close\r\n",
                    status, len
                );
                if status.starts_with("206") {
                    head.push_str(&format!("Content-Range: bytes {}-{}/{}\r\n", start, end, total));
                }
                head.push_str("\r\n");
                let _ = sock.write_all(head.as_bytes()).await;
                if is_head {
                    return;
                }
                for chunk in data[start..=end].chunks(16 * 1024) {
                    if sock.write_all(chunk).await.is_err() {
                        return;
                    }
                    if delay_ms > 0 {
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            });
        }
    });
    port
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lodm_it_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

async fn wait_state(api: &DownloadManagerApi, want: JobState, secs: u64) -> bool {
    for _ in 0..(secs * 20) {
        if api.list_downloads().iter().all(|j| j.state == want) && !api.list_downloads().is_empty() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

fn cfg(dir: &Path, max: usize) -> ApiConfig {
    ApiConfig {
        store_path: Some(dir.join("jobs.json")),
        proxy: None,
        max_concurrent: Some(max),
    }
}

#[tokio::test]
async fn downloads_multi_segment_file_intact() {
    let data = payload(5 * 1024 * 1024);
    let port = spawn_server(data.clone(), 0).await;
    let dir = temp_dir("full");
    let api = DownloadManagerApi::with_config(cfg(&dir, 3)).unwrap();

    api.add_download(format!("http://127.0.0.1:{}/f", port), dir.clone(), None, 4)
        .await
        .unwrap();
    assert!(wait_state(&api, JobState::Completed, 20).await, "download did not complete");

    let job = &api.list_downloads()[0];
    assert_eq!(job.filename, "served.bin");
    let written = std::fs::read(job.target_filepath()).unwrap();
    assert_eq!(written.len(), data.len());
    assert_eq!(&written[..], &data[..]);
    assert!(!job.meta_filepath().exists(), "meta should be cleaned up");
}

#[tokio::test]
async fn name_conflict_gets_counter() {
    let data = payload(200_000);
    let port = spawn_server(data, 0).await;
    let dir = temp_dir("conflict");
    let api = DownloadManagerApi::with_config(cfg(&dir, 3)).unwrap();
    let url = format!("http://127.0.0.1:{}/f", port);

    api.add_download(url.clone(), dir.clone(), None, 2).await.unwrap();
    assert!(wait_state(&api, JobState::Completed, 10).await);
    api.add_download(url, dir.clone(), None, 2).await.unwrap();
    assert!(wait_state(&api, JobState::Completed, 10).await);

    assert!(dir.join("served.bin").exists());
    assert!(dir.join("served (1).bin").exists());
}

#[tokio::test]
async fn pause_resume_and_restart_persistence() {
    let data = payload(4 * 1024 * 1024);
    let port = spawn_server(data.clone(), 8).await;
    let dir = temp_dir("resume");
    let url = format!("http://127.0.0.1:{}/f", port);

    {
        let api = DownloadManagerApi::with_config(cfg(&dir, 3)).unwrap();
        let id = api.add_download(url, dir.clone(), None, 2).await.unwrap();
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert!(api.pause_download(id));
        assert!(wait_state(&api, JobState::Paused, 10).await);
        let partial = api.list_downloads()[0].downloaded_bytes();
        assert!(partial > 0 && partial < data.len() as u64, "partial={}", partial);
        api.persist();
    }

    // "Restart": a fresh API instance restores the job from disk, still paused.
    let api = DownloadManagerApi::with_config(cfg(&dir, 3)).unwrap();
    let jobs = api.list_downloads();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].state, JobState::Paused);
    let id = jobs[0].id;

    api.start_download(id).await.unwrap();
    assert!(wait_state(&api, JobState::Completed, 30).await, "resume did not complete");
    let written = std::fs::read(api.list_downloads()[0].target_filepath()).unwrap();
    assert_eq!(&written[..], &data[..], "resumed file is corrupt");
}

#[tokio::test]
async fn concurrency_limit_queues_and_remove_deletes_files() {
    let data = payload(3 * 1024 * 1024);
    let port = spawn_server(data, 10).await;
    let dir = temp_dir("queue");
    let api = DownloadManagerApi::with_config(cfg(&dir, 1)).unwrap();

    let a = api
        .add(NewDownload { filename: Some("a.bin".into()), ..NewDownload::simple(format!("http://127.0.0.1:{}/a", port), dir.clone(), 1) })
        .await
        .unwrap();
    let b = api
        .add(NewDownload { filename: Some("b.bin".into()), ..NewDownload::simple(format!("http://127.0.0.1:{}/b", port), dir.clone(), 1) })
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;
    let jb = api.get_job(b).unwrap();
    assert_eq!(jb.state, JobState::Queued, "second job must wait for a slot");
    assert!(api.get_job(a).unwrap().state.is_active());

    // Removing the running job frees the slot; b starts and the partial a.bin is gone.
    api.remove_download(a, false).await.unwrap();
    assert!(api.get_job(a).is_none());
    assert!(!dir.join("a.bin").exists());
    assert!(!dir.join("a.bin.part.meta").exists());

    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(api.get_job(b).unwrap().state.is_active(), "queued job should start");
    api.remove_download(b, true).await.unwrap();
}

#[tokio::test]
async fn failure_is_recorded_on_job() {
    let dir = temp_dir("fail");
    let api = DownloadManagerApi::with_config(cfg(&dir, 3)).unwrap();
    // Nothing listens on port 1.
    api.add_download("http://127.0.0.1:1/x".into(), dir.clone(), None, 2)
        .await
        .unwrap();
    assert!(wait_state(&api, JobState::Failed, 15).await);
    assert!(api.list_downloads()[0].error_message.is_some());
}

#[tokio::test]
async fn rejects_non_http_urls() {
    let api = DownloadManagerApi::new();
    let r = api
        .add_download("file:///etc/passwd".into(), std::env::temp_dir(), None, 1)
        .await;
    assert!(r.is_err());
}
