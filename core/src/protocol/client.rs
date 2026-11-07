use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

/// Reusable HTTP client factory with keep-alive connection pooling.
pub struct HttpClientFactory;

impl HttpClientFactory {
    pub fn create_client() -> Arc<Client> {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Arc::new(client)
    }
}
