use reqwest::{Client, Proxy};
use std::sync::Arc;
use std::time::Duration;

/// Reusable HTTP client factory with keep-alive connection pooling.
pub struct HttpClientFactory;

impl HttpClientFactory {
    pub fn create_client() -> Arc<Client> {
        Self::create_client_with_proxy(None).expect("Failed to build HTTP client")
    }

    /// Build a client, optionally routed through an HTTP(S)/SOCKS proxy URL
    /// (e.g. `http://127.0.0.1:8080`). An empty string means no proxy.
    pub fn create_client_with_proxy(proxy_url: Option<&str>) -> Result<Arc<Client>, String> {
        let mut builder = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(30));

        if let Some(url) = proxy_url.map(str::trim).filter(|u| !u.is_empty()) {
            let proxy = Proxy::all(url).map_err(|e| format!("Invalid proxy '{}': {}", url, e))?;
            builder = builder.proxy(proxy);
        }

        builder.build().map(Arc::new).map_err(|e| e.to_string())
    }
}
