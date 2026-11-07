use super::commands::DownloadManagerApi;
use crate::common::types::JobId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct AddDownloadRequest {
    url: String,
    destination: Option<String>,
    threads: Option<u32>,
}

#[derive(Deserialize)]
struct JobActionRequest {
    id: String,
}

/// Lightweight HTTP/REST server providing local IPC bridge between UI and Core Engine.
pub struct LocalApiServer {
    api: Arc<DownloadManagerApi>,
    port: u16,
}

impl LocalApiServer {
    pub fn new(api: Arc<DownloadManagerApi>, port: u16) -> Self {
        Self { api, port }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        println!("[API SERVER] Listening on http://{}", addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let api = self.api.clone();

            tokio::spawn(async move {
                let mut buffer = [0u8; 8192];
                let bytes_read = match socket.read(&mut buffer).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
                let mut lines = request_str.lines();
                let request_line = match lines.next() {
                    Some(l) => l,
                    None => return,
                };

                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() < 2 {
                    return;
                }

                let method = parts[0];
                let path = parts[1];

                // Handle CORS preflight
                if method == "OPTIONS" {
                    let response = "HTTP/1.1 204 No Content\r\n\
                        Access-Control-Allow-Origin: *\r\n\
                        Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                        Access-Control-Allow-Headers: Content-Type\r\n\
                        Content-Length: 0\r\n\r\n";
                    let _ = socket.write_all(response.as_bytes()).await;
                    return;
                }

                // Extract body for POST requests
                let body = if let Some(idx) = request_str.find("\r\n\r\n") {
                    &request_str[idx + 4..]
                } else {
                    ""
                };

                let (status, content_type, response_body) = if method == "GET" && path == "/api/downloads" {
                    let jobs = api.list_downloads();
                    let json = serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".to_string());
                    ("200 OK", "application/json", json)
                } else if method == "POST" && path == "/api/downloads" {
                    match serde_json::from_str::<AddDownloadRequest>(body) {
                        Ok(req) => {
                            let dest = req.destination.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("./downloads"));
                            let threads = req.threads.unwrap_or(8);
                            match api.add_download(req.url, dest, None, threads).await {
                                Ok(job_id) => {
                                    let resp = serde_json::json!({ "success": true, "job_id": job_id.to_string() });
                                    ("200 OK", "application/json", resp.to_string())
                                }
                                Err(e) => {
                                    let resp = serde_json::json!({ "success": false, "error": e.to_string() });
                                    ("500 Internal Server Error", "application/json", resp.to_string())
                                }
                            }
                        }
                        Err(e) => {
                            let resp = serde_json::json!({ "success": false, "error": e.to_string() });
                            ("400 Bad Request", "application/json", resp.to_string())
                        }
                    }
                } else if method == "POST" && path == "/api/downloads/pause" {
                    if let Ok(req) = serde_json::from_str::<JobActionRequest>(body) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(&req.id) {
                            api.pause_download(JobId(uuid));
                        }
                    }
                    ("200 OK", "application/json", r#"{"success": true}"#.to_string())
                } else if method == "POST" && path == "/api/downloads/resume" {
                    if let Ok(req) = serde_json::from_str::<JobActionRequest>(body) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(&req.id) {
                            let _ = api.start_download(JobId(uuid)).await;
                        }
                    }
                    ("200 OK", "application/json", r#"{"success": true}"#.to_string())
                } else {
                    ("404 Not Found", "application/json", r#"{"error": "Endpoint not found"}"#.to_string())
                };

                let response_header = format!(
                    "HTTP/1.1 {}\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type\r\n\
                    Content-Type: {}\r\n\
                    Content-Length: {}\r\n\r\n",
                    status,
                    content_type,
                    response_body.len()
                );

                let _ = socket.write_all(response_header.as_bytes()).await;
                let _ = socket.write_all(response_body.as_bytes()).await;
            });
        }
    }
}
