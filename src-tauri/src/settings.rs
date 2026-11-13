use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// User-facing application settings, persisted as JSON in the app data dir.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub download_dir: String,
    pub threads: u32,
    /// 0 = unlimited
    pub speed_limit_kbps: u64,
    pub max_concurrent: usize,
    /// Empty = no proxy. Applied on next launch.
    pub proxy: String,
    pub use_categories: bool,
    pub clipboard_watch: bool,
    pub close_to_tray: bool,
    pub notifications: bool,
    /// "tr" | "en"
    pub language: String,
    /// "amoled" | "light"
    pub theme: String,
    pub api_port: u16,
    pub api_token: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_dir: String::new(),
            threads: 8,
            speed_limit_kbps: 0,
            max_concurrent: 3,
            proxy: String::new(),
            use_categories: false,
            clipboard_watch: true,
            close_to_tray: true,
            notifications: true,
            language: "tr".into(),
            theme: "amoled".into(),
            api_port: 4141,
            api_token: String::new(),
        }
    }
}

impl Settings {
    /// Load settings, filling in defaults and generating a secret API token on first run.
    pub fn load(path: &Path, default_dir: PathBuf) -> Self {
        let mut s: Settings = fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default();
        if s.download_dir.trim().is_empty() {
            s.download_dir = default_dir.display().to_string();
        }
        if s.api_token.len() < 16 {
            s.api_token = format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            );
        }
        s.sanitize();
        let _ = s.save(path);
        s
    }

    /// Clamp values into safe ranges.
    pub fn sanitize(&mut self) {
        self.threads = self.threads.clamp(1, 32);
        self.max_concurrent = self.max_concurrent.clamp(1, 16);
        if self.language != "en" {
            self.language = "tr".into();
        }
        if self.theme != "light" {
            self.theme = "amoled".into();
        }
        if self.api_port == 0 {
            self.api_port = 4141;
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, path)
    }

    pub fn speed_limit_bytes(&self) -> Option<u64> {
        (self.speed_limit_kbps > 0).then(|| self.speed_limit_kbps * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_load_generates_token_and_dir() {
        let dir = std::env::temp_dir().join(format!("lodm_settings_{}", std::process::id()));
        let path = dir.join("settings.json");
        let s = Settings::load(&path, dir.join("dl"));
        assert!(s.api_token.len() >= 32);
        assert!(s.download_dir.ends_with("dl"));
        // token is stable across loads
        let s2 = Settings::load(&path, dir.join("other"));
        assert_eq!(s.api_token, s2.api_token);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sanitize_clamps() {
        let mut s = Settings { threads: 999, max_concurrent: 0, language: "xx".into(), ..Default::default() };
        s.sanitize();
        assert_eq!((s.threads, s.max_concurrent, s.language.as_str()), (32, 1, "tr"));
    }
}
