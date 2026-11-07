use lavinia_download_core::bridge::{DownloadManagerApi, LocalApiServer};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== LODM — LaviniaOS Download Manager Core ===");
    let api = Arc::new(DownloadManagerApi::new());

    // Subscribe to telemetry and print events
    let mut rx = api.dispatcher().subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            println!("[EVENT] {:?}", event);
        }
    });

    // Start local REST API bridge server on port 4141 for the UI
    let server = LocalApiServer::new(api.clone(), 4141);
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("[API SERVER ERROR] {}", e);
        }
    });

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let url = &args[1];
        let dest = std::path::PathBuf::from("./downloads");
        let threads = if args.len() > 2 {
            args[2].parse().unwrap_or(8)
        } else {
            8
        };

        println!("Initiating real multi-segment download: {}", url);
        let job_id = api.add_download(url.clone(), dest, None, threads).await?;
        println!("Job spawned with ID: {}", job_id);
    } else {
        println!("Engine initialized with local IPC bridge on port 4141.");
        println!("Ready to accept raw data download jobs from UI.");
    }

    // Keep running forever in daemon mode
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
