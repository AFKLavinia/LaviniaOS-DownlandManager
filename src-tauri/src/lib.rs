use lavinia_download_core::bridge::DownloadManagerApi;
use lavinia_download_core::common::types::JobId;
use lavinia_download_core::domain::job::DownloadJob;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
fn get_downloads(api: State<Arc<DownloadManagerApi>>) -> Vec<DownloadJob> {
    api.list_downloads()
}

#[tauri::command]
async fn add_download(
    api: State<'_, Arc<DownloadManagerApi>>,
    url: String,
    destination: Option<String>,
    threads: Option<u32>,
) -> Result<String, String> {
    let dest = destination
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./downloads"));
    let count = threads.unwrap_or(8);

    api.add_download(url, dest, None, count)
        .await
        .map(|id| id.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn pause_download(api: State<Arc<DownloadManagerApi>>, id: String) -> bool {
    if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        api.pause_download(JobId(uuid))
    } else {
        false
    }
}

#[tauri::command]
async fn resume_download(api: State<'_, Arc<DownloadManagerApi>>, id: String) -> Result<(), String> {
    if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
        api.start_download(JobId(uuid))
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Invalid Job ID".to_string())
    }
}

#[tauri::command]
fn win_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn win_maximize(window: tauri::Window) {
    if let Ok(is_max) = window.is_maximized() {
        if is_max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn win_close(window: tauri::Window) {
    let _ = window.close();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let api = Arc::new(DownloadManagerApi::new());

    tauri::Builder::default()
        .manage(api)
        .invoke_handler(tauri::generate_handler![
            get_downloads,
            add_download,
            pause_download,
            resume_download,
            win_minimize,
            win_maximize,
            win_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running LODM tauri application");
}
