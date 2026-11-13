use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

/// Per-job HTTP request customisation: custom headers, cookies, basic auth.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestOptions {
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub cookie: Option<String>,
    #[serde(default)]
    pub referer: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub basic_auth: Option<(String, String)>,
}

impl RequestOptions {
    /// Apply options onto an outgoing request (single place, used by probe + workers).
    pub fn apply(&self, mut req: RequestBuilder) -> RequestBuilder {
        for (k, v) in &self.headers {
            if !k.trim().is_empty() {
                req = req.header(k.trim(), v.trim());
            }
        }
        if let Some(c) = self.cookie.as_deref().filter(|c| !c.is_empty()) {
            req = req.header("Cookie", c);
        }
        if let Some(r) = self.referer.as_deref().filter(|r| !r.is_empty()) {
            req = req.header("Referer", r);
        }
        if let Some(ua) = self.user_agent.as_deref().filter(|u| !u.is_empty()) {
            req = req.header("User-Agent", ua);
        }
        if let Some((user, pass)) = &self.basic_auth {
            req = req.basic_auth(user, Some(pass));
        }
        req
    }
}
