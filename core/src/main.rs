use lavinia_download_core::bridge::DownloadManagerApi;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LaviniaOS Download Manager Engine (IDM-Grade Core) ===");
    let api = DownloadManagerApi::new();

    // Subscribe to telemetry
    let mut rx = api.dispatcher().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("[EVENT] {:?}", event);
        }
    });

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let url = &args[1];
        let dest = PathBuf::from("./downloads");
        let threads = if args.len() > 2 {
            args[2].parse().unwrap_or(8)
        } else {
            8
        };

        println!("Initiating multi-segment download: {}", url);
        let job_id = api.add_download(url.clone(), dest, None, threads).await?;
        println!("Job spawned with ID: {}", job_id);

        // Keep running until complete
        loop {
            sleep(Duration::from_secs(1)).await;
            let jobs = api.list_downloads();
            if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                if !job.state.is_active() {
                    println!("Job reached final state: {:?}", job.state);
                    break;
                }
            }
        }
    } else {
        println!("Usage: lavinia_downloader <URL> [thread_count]");
        println!("Engine initialized successfully in idle daemon mode.");
    }

    Ok(())
}
