mod settings;

use lavinia_download_core::bridge::{ApiConfig, DownloadManagerApi, LocalApiServer, NewDownload};
use lavinia_download_core::common::types::JobId;
use lavinia_download_core::domain::job::DownloadJob;
use lavinia_download_core::domain::request_options::RequestOptions;
use lavinia_download_core::domain::state::JobState;
use lavinia_download_core::events::events::DownloadEvent;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use settings::Settings;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tauri_plugin_notification::NotificationExt;

/// Shared application state managed by Tauri.
struct AppState {
    api: Arc<DownloadManagerApi>,
    settings: Mutex<Settings>,
    settings_path: PathBuf,
}

type Shared<'a> = State<'a, Arc<AppState>>;

fn parse_id(id: &str) -> Result<JobId, String> {
    uuid::Uuid::parse_str(id)
        .map(JobId)
        .map_err(|_| "Invalid job id".to_string())
}

// ---------------------------------------------------------------- downloads

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AddRequest {
    url: String,
    destination: Option<String>,
    threads: Option<u32>,
    filename: Option<String>,
    headers: Vec<(String, String)>,
    cookie: Option<String>,
    referer: Option<String>,
    user_agent: Option<String>,
    username: Option<String>,
    password: Option<String>,
    paused: bool,
    checksum: Option<String>,
}

#[tauri::command]
fn get_downloads(state: Shared) -> Vec<DownloadJob> {
    state.api.list_downloads()
}

#[tauri::command]
async fn add_download(state: Shared<'_>, req: AddRequest) -> Result<String, String> {
    let (default_dir, default_threads, categorize) = {
        let s = state.settings.lock();
        (PathBuf::from(&s.download_dir), s.threads, s.use_categories)
    };
    let dest = req
        .destination
        .filter(|d| !d.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or(default_dir);

    let basic_auth = match (req.username, req.password) {
        (Some(u), p) if !u.is_empty() => Some((u, p.unwrap_or_default())),
        _ => None,
    };

    let mut nd = NewDownload::simple(req.url, dest, req.threads.unwrap_or(default_threads));
    nd.filename = req.filename;
    nd.categorize = categorize;
    nd.paused = req.paused;
    nd.expected_checksum = req.checksum;
    nd.request = RequestOptions {
        headers: req.headers,
        cookie: req.cookie,
        referer: req.referer,
        user_agent: req.user_agent,
        basic_auth,
    };

    state
        .api
        .add(nd)
        .await
        .map(|id| id.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn pause_download(state: Shared, id: String) -> Result<bool, String> {
    Ok(state.api.pause_download(parse_id(&id)?))
}

#[tauri::command]
async fn resume_download(state: Shared<'_>, id: String) -> Result<(), String> {
    state
        .api
        .start_download(parse_id(&id)?)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_download(state: Shared<'_>, id: String, delete_file: bool) -> Result<(), String> {
    state
        .api
        .remove_download(parse_id(&id)?, delete_file)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn pause_all(state: Shared) {
    state.api.pause_all();
}

#[tauri::command]
async fn resume_all(state: Shared<'_>) -> Result<(), String> {
    state.api.resume_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_completed(state: Shared) {
    state.api.clear_completed();
}

#[tauri::command]
fn refresh_url(state: Shared, id: String, url: String) -> Result<(), String> {
    state
        .api
        .update_url(parse_id(&id)?, url)
        .map_err(|e| e.to_string())
}

fn reveal(path: &std::path::Path, select: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        if select {
            cmd.arg(format!("/select,{}", path.display()));
        } else {
            cmd.arg(path);
        }
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        if select {
            cmd.arg("-R");
        }
        cmd.arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if select { path.parent().unwrap_or(path) } else { path };
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// Open the finished file with its default application.
#[tauri::command]
fn open_file(state: Shared, id: String) -> Result<(), String> {
    let job = state.api.get_job(parse_id(&id)?).ok_or("Job not found")?;
    let path = job.target_filepath();
    if !path.exists() {
        return Err("File not found on disk".into());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        reveal(&path, false)
    }
}

/// Show the file in the OS file manager (or the folder if the file does not exist yet).
#[tauri::command]
fn open_folder(state: Shared, id: Option<String>) -> Result<(), String> {
    if let Some(id) = id {
        let job = state.api.get_job(parse_id(&id)?).ok_or("Job not found")?;
        let file = job.target_filepath();
        if file.exists() {
            return reveal(&file, true);
        }
        std::fs::create_dir_all(&job.destination_path).map_err(|e| e.to_string())?;
        return reveal(&job.destination_path, false);
    }
    let dir = PathBuf::from(state.settings.lock().download_dir.clone());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    reveal(&dir, false)
}

#[tauri::command]
async fn pick_folder() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .pick_folder()
            .map(|p| p.display().to_string())
    })
    .await
    .ok()
    .flatten()
}

/// Current clipboard text (used by the clipboard link catcher).
#[tauri::command]
fn get_clipboard_text() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

// ----------------------------------------------------------------- settings

#[derive(Serialize)]
struct SaveResult {
    settings: Settings,
    restart_required: bool,
}

#[tauri::command]
fn get_settings(state: Shared) -> Settings {
    state.settings.lock().clone()
}

#[tauri::command]
fn save_settings(state: Shared, mut new_settings: Settings) -> Result<SaveResult, String> {
    new_settings.sanitize();
    let mut current = state.settings.lock();
    // The secret token is never editable from the UI.
    new_settings.api_token = current.api_token.clone();
    if new_settings.download_dir.trim().is_empty() {
        new_settings.download_dir = current.download_dir.clone();
    }

    let restart_required =
        new_settings.proxy != current.proxy || new_settings.api_port != current.api_port;

    state.api.set_speed_limit(new_settings.speed_limit_bytes());
    state.api.set_max_concurrent(new_settings.max_concurrent);
    new_settings
        .save(&state.settings_path)
        .map_err(|e| e.to_string())?;
    *current = new_settings.clone();

    Ok(SaveResult {
        settings: new_settings,
        restart_required,
    })
}

/// Connection info for the browser extension (shown in Settings).
#[derive(Serialize)]
struct ApiInfo {
    port: u16,
    token: String,
}

#[tauri::command]
fn get_api_info(state: Shared) -> ApiInfo {
    let s = state.settings.lock();
    ApiInfo {
        port: s.api_port,
        token: s.api_token.clone(),
    }
}

// ------------------------------------------------------------------- window

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

/// Close button: hide to tray when enabled, otherwise quit.
#[tauri::command]
fn win_close(app: AppHandle, state: Shared, window: tauri::Window) {
    if state.settings.lock().close_to_tray {
        let _ = window.hide();
    } else {
        quit_app(&app);
    }
}

fn quit_app(app: &AppHandle) {
    if let Some(state) = app.try_state::<Arc<AppState>>() {
        state.api.pause_all();
        state.api.persist();
    }
    app.exit(0);
}

fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

// -------------------------------------------------------------------- setup

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "LODM", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause_all", "Pause all", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume_all", "Resume all", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &sep, &pause, &resume, &sep, &quit])?;

    let mut builder = TrayIconBuilder::with_id("lodm-tray")
        .tooltip("LODM — LaviniaOS Download Manager")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "pause_all" => {
                if let Some(s) = app.try_state::<Arc<AppState>>() {
                    s.api.pause_all();
                }
            }
            "resume_all" => {
                if let Some(s) = app.try_state::<Arc<AppState>>() {
                    let api = s.api.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = api.resume_all().await;
                    });
                }
            }
            "quit" => quit_app(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

#[derive(Clone, Serialize)]
struct Notice {
    kind: &'static str,
    title: String,
    body: String,
}

fn file_name_of(api: &DownloadManagerApi, id: JobId) -> String {
    api.get_job(id).map(|j| j.filename).unwrap_or_default()
}

/// Turn engine events into UI toasts + native notifications.
fn spawn_notifier(app: AppHandle, state: Arc<AppState>) {
    let mut rx = state.api.dispatcher().subscribe();
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let notice = match event {
                DownloadEvent::JobCompleted { job_id, .. } => Notice {
                    kind: "completed",
                    title: "Completed".into(),
                    body: file_name_of(&state.api, job_id),
                },
                DownloadEvent::JobFailed { job_id, error } => Notice {
                    kind: "failed",
                    title: file_name_of(&state.api, job_id),
                    body: error,
                },
                _ => continue,
            };
            let _ = app.emit("lodm://notice", notice.clone());
            if state.settings.lock().notifications {
                let _ = app
                    .notification()
                    .builder()
                    .title(format!("LODM — {}", notice.title))
                    .body(&notice.body)
                    .show();
            }
        }
    });
}

/// Push the job list to the UI (only when something changed) and autosave it.
fn spawn_snapshot_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut last = String::new();
        let mut ticks = 0u32;
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let jobs = state.api.list_downloads();
            if let Ok(json) = serde_json::to_string(&jobs) {
                if json != last {
                    let _ = app.emit("lodm://downloads", &jobs);
                    last = json;
                }
            }
            ticks += 1;
            if ticks % 8 == 0 && jobs.iter().any(|j| j.state.is_active()) {
                state.api.persist();
            }
            // Tray tooltip with live status
            if ticks % 4 == 0 {
                let active = jobs.iter().filter(|j| j.state.is_active()).count();
                if let Some(tray) = app.tray_by_id("lodm-tray") {
                    let tip = if active > 0 {
                        format!("LODM — {} active", active)
                    } else {
                        "LODM — LaviniaOS Download Manager".to_string()
                    };
                    let _ = tray.set_tooltip(Some(tip));
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let settings_path = data_dir.join("settings.json");
            let default_dir = app
                .path()
                .download_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("LODM");
            let settings = Settings::load(&settings_path, default_dir);

            let api = Arc::new(
                DownloadManagerApi::with_config(ApiConfig {
                    store_path: Some(data_dir.join("jobs.json")),
                    proxy: Some(settings.proxy.clone()),
                    max_concurrent: Some(settings.max_concurrent),
                })
                .unwrap_or_else(|e| {
                    eprintln!("[LODM] invalid proxy ({e}); starting without proxy");
                    DownloadManagerApi::with_config(ApiConfig {
                        store_path: Some(data_dir.join("jobs.json")),
                        proxy: None,
                        max_concurrent: Some(settings.max_concurrent),
                    })
                    .expect("engine init")
                }),
            );
            api.set_speed_limit(settings.speed_limit_bytes());

            // Local REST bridge for the browser extension (token protected).
            let server = LocalApiServer::new(api.clone(), settings.api_port)
                .with_token(settings.api_token.clone())
                .with_default_dir(&settings.download_dir);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server.run().await {
                    eprintln!("[LODM] local API server stopped: {e}");
                }
            });

            let state = Arc::new(AppState {
                api,
                settings: Mutex::new(settings),
                settings_path,
            });
            app.manage(state.clone());

            build_tray(app.handle())?;
            spawn_notifier(app.handle().clone(), state.clone());
            spawn_snapshot_loop(app.handle().clone(), state);
            Ok(())
        })
        .on_window_event(|window, event| {
            // Alt+F4 / OS close also respects "close to tray".
            if let WindowEvent::CloseRequested { api, .. } = event {
                let to_tray = window
                    .app_handle()
                    .try_state::<Arc<AppState>>()
                    .map(|s| s.settings.lock().close_to_tray)
                    .unwrap_or(false);
                if to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else if let Some(s) = window.app_handle().try_state::<Arc<AppState>>() {
                    s.api.pause_all();
                    s.api.persist();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_downloads,
            add_download,
            pause_download,
            resume_download,
            remove_download,
            pause_all,
            resume_all,
            clear_completed,
            refresh_url,
            open_file,
            open_folder,
            pick_folder,
            get_clipboard_text,
            get_settings,
            save_settings,
            get_api_info,
            win_minimize,
            win_maximize,
            win_close
        ])
        .build(tauri::generate_context!())
        .expect("error while building LODM tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(s) = app.try_state::<Arc<AppState>>() {
                    s.api.persist();
                }
            }
        });
}

#[allow(dead_code)]
fn _state_is_active(s: JobState) -> bool {
    s.is_active()
}
