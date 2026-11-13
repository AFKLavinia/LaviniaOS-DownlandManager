use super::commands::{DownloadManagerApi, NewDownload};
use crate::common::types::JobId;
use crate::domain::request_options::RequestOptions;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const TOKEN_HEADER: &str = "x-lodm-token";

#[derive(Deserialize)]
struct AddDownloadRequest {
    url: String,
    destination: Option<String>,
    threads: Option<u32>,
    filename: Option<String>,
    #[serde(default)]
    headers: Vec<(String, String)>,
    cookie: Option<String>,
    referer: Option<String>,
    user_agent: Option<String>,
    #[serde(default)]
    paused: bool,
}

#[derive(Deserialize)]
struct JobActionRequest {
    id: String,
    #[serde(default)]
    delete_file: bool,
}

struct HttpRequest {
    method: String,
    path: String,
    origin: Option<String>,
    token: Option<String>,
    body: String,
}

/// Lightweight HTTP/REST server providing a local IPC bridge for external clients
/// (browser extension, CLI). Every request must carry the secret `X-LODM-Token`.
pub struct LocalApiServer {
    api: Arc<DownloadManagerApi>,
    port: u16,
    token: Option<String>,
    default_dir: PathBuf,
}

impl LocalApiServer {
    pub fn new(api: Arc<DownloadManagerApi>, port: u16) -> Self {
        Self {
            api,
            port,
            token: None,
            default_dir: PathBuf::from("./downloads"),
        }
    }

    /// Require this secret in the `X-LODM-Token` header.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn with_default_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.default_dir = dir.into();
        self
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        println!("[API SERVER] Listening on http://{}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let api = self.api.clone();
            let token = self.token.clone();
            let default_dir = self.default_dir.clone();
            tokio::spawn(async move {
                Self::handle(socket, api, token, default_dir).await;
            });
        }
    }

    async fn read_request(socket: &mut TcpStream) -> Option<HttpRequest> {
        let mut buf: Vec<u8> = Vec::with_capacity(4096);
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos;
            }
            if buf.len() > MAX_REQUEST_BYTES {
                return None;
            }
            let n = socket.read(&mut chunk).await.ok()?;
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
        };

        let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
        let mut lines = head.lines();
        let mut parts = lines.next()?.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();

        let mut content_length = 0usize;
        let mut origin = None;
        let mut token = None;
        for line in lines {
            if let Some((k, v)) = line.split_once(':') {
                let k = k.trim().to_ascii_lowercase();
                let v = v.trim().to_string();
                match k.as_str() {
                    "content-length" => content_length = v.parse().unwrap_or(0),
                    "origin" => origin = Some(v),
                    TOKEN_HEADER => token = Some(v),
                    _ => {}
                }
            }
        }
        if content_length > MAX_REQUEST_BYTES {
            return None;
        }

        let body_start = header_end + 4;
        while buf.len() < body_start + content_length {
            let n = socket.read(&mut chunk).await.ok()?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let body = String::from_utf8_lossy(&buf[body_start.min(buf.len())..]).to_string();

        Some(HttpRequest {
            method,
            path,
            origin,
            token,
            body,
        })
    }

    /// Only extension pages and the local UI may read responses cross-origin.
    fn cors_origin(origin: &Option<String>) -> Option<String> {
        let o = origin.as_deref()?;
        let allowed = o.starts_with("chrome-extension://")
            || o.starts_with("moz-extension://")
            || o.starts_with("http://tauri.localhost")
            || o.starts_with("tauri://")
            || o.starts_with("http://localhost:5173");
        allowed.then(|| o.to_string())
    }

    fn token_ok(expected: &Option<String>, given: &Option<String>) -> bool {
        match expected {
            None => true,
            Some(exp) => match given {
                Some(g) if g.len() == exp.len() => {
                    g.bytes().zip(exp.bytes()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
                }
                _ => false,
            },
        }
    }

    async fn handle(
        mut socket: TcpStream,
        api: Arc<DownloadManagerApi>,
        token: Option<String>,
        default_dir: PathBuf,
    ) {
        let Some(req) = Self::read_request(&mut socket).await else {
            return;
        };
        let cors = Self::cors_origin(&req.origin);

        let (status, body) = if req.method == "OPTIONS" {
            ("204 No Content", String::new())
        } else if !Self::token_ok(&token, &req.token) {
            ("401 Unauthorized", r#"{"success":false,"error":"invalid token"}"#.to_string())
        } else {
            Self::route(&api, &req, &default_dir).await
        };

        let mut header = format!("HTTP/1.1 {}\r\n", status);
        if let Some(o) = cors {
            header.push_str(&format!(
                "Access-Control-Allow-Origin: {}\r\nVary: Origin\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, X-LODM-Token\r\n",
                o
            ));
        }
        header.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        ));
        let _ = socket.write_all(header.as_bytes()).await;
        let _ = socket.write_all(body.as_bytes()).await;
        let _ = socket.shutdown().await;
    }

    async fn route(
        api: &Arc<DownloadManagerApi>,
        req: &HttpRequest,
        default_dir: &PathBuf,
    ) -> (&'static str, String) {
        let ok = |v: serde_json::Value| ("200 OK", v.to_string());
        let bad = |e: String| {
            (
                "400 Bad Request",
                serde_json::json!({ "success": false, "error": e }).to_string(),
            )
        };

        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/api/ping") => ok(serde_json::json!({ "success": true, "app": "LODM" })),
            ("GET", "/api/downloads") => {
                let jobs = api.list_downloads();
                ("200 OK", serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".into()))
            }
            ("POST", "/api/downloads") => match serde_json::from_str::<AddDownloadRequest>(&req.body) {
                Ok(r) => {
                    let mut nd = NewDownload::simple(
                        r.url,
                        r.destination.map(PathBuf::from).unwrap_or_else(|| default_dir.clone()),
                        r.threads.unwrap_or(8),
                    );
                    nd.filename = r.filename;
                    nd.paused = r.paused;
                    nd.request = RequestOptions {
                        headers: r.headers,
                        cookie: r.cookie,
                        referer: r.referer,
                        user_agent: r.user_agent,
                        basic_auth: None,
                    };
                    match api.add(nd).await {
                        Ok(id) => ok(serde_json::json!({ "success": true, "job_id": id.to_string() })),
                        Err(e) => (
                            "500 Internal Server Error",
                            serde_json::json!({ "success": false, "error": e.to_string() }).to_string(),
                        ),
                    }
                }
                Err(e) => bad(e.to_string()),
            },
            ("POST", "/api/downloads/pause")
            | ("POST", "/api/downloads/resume")
            | ("POST", "/api/downloads/remove") => {
                let Ok(r) = serde_json::from_str::<JobActionRequest>(&req.body) else {
                    return bad("invalid body".into());
                };
                let Ok(uuid) = uuid::Uuid::parse_str(&r.id) else {
                    return bad("invalid id".into());
                };
                let id = JobId(uuid);
                let res = match req.path.as_str() {
                    "/api/downloads/pause" => Ok(api.pause_download(id)),
                    "/api/downloads/resume" => api.start_download(id).await.map(|_| true),
                    _ => api.remove_download(id, r.delete_file).await.map(|_| true),
                };
                match res {
                    Ok(v) => ok(serde_json::json!({ "success": v })),
                    Err(e) => bad(e.to_string()),
                }
            }
            _ => (
                "404 Not Found",
                r#"{"success":false,"error":"Endpoint not found"}"#.to_string(),
            ),
        }
    }
}
