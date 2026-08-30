use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

pub mod downloader;
pub mod queue;
pub mod service;
pub mod storage;

pub use downloader::{
    DownloadFailureKind, DownloadRun, DownloaderError, DownloaderEvent, DownloaderRequest,
    DownloaderRunner,
};
pub use queue::{DownloadQueue, DownloadStatus, OutputPreset, QueueJob};
pub use service::DownloadService;
pub use storage::migrate;

struct AppState {
    queue: Arc<DownloadQueue>,
    service: DownloadService,
}

#[derive(Deserialize)]
struct EnqueueDownloadsRequest {
    urls: Vec<String>,
    destination: String,
    output_preset: Option<OutputPreset>,
    write_subs: Option<bool>,
    cookies: Option<String>,
}

#[tauri::command]
fn enqueue_downloads(
    request: EnqueueDownloadsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<QueueJob>, String> {
    let jobs = state
        .queue
        .enqueue_with_options(
            &request.urls,
            Path::new(&request.destination),
            request.output_preset.unwrap_or(OutputPreset::Mp4Compatible),
            request.write_subs.unwrap_or(false),
        )
        .map_err(|error| error.to_string())?;
    if let Some(cookies) = request.cookies {
        let cookies = PathBuf::from(cookies);
        for job in &jobs {
            state
                .service
                .remember_cookie_source(&job.id, cookies.clone());
        }
    }
    state
        .service
        .start_available()
        .map_err(|error| error.to_string())?;
    Ok(jobs)
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<QueueJob>, String> {
    state.queue.jobs().map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.remove(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.cancel(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn retry_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.retry(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .service
        .clear_history()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_download_folder(
    id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let destination = state
        .service
        .destination_of(&id)
        .ok_or_else(|| format!("작업을 찾을 수 없습니다: {id}"))?;
    app.opener()
        .open_path(destination.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn default_download_destination(app: AppHandle) -> Result<String, String> {
    app.path()
        .download_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_download_concurrency(state: State<'_, AppState>) -> Result<u8, String> {
    state.queue.concurrency().map_err(|error| error.to_string())
}

#[tauri::command]
fn set_download_concurrency(concurrency: u8, state: State<'_, AppState>) -> Result<bool, String> {
    let changed = state
        .queue
        .set_concurrency(concurrency)
        .map_err(|error| error.to_string())?;
    if changed {
        state
            .service
            .start_available()
            .map_err(|error| error.to_string())?;
    }
    Ok(changed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let queue = Arc::new(DownloadQueue::open(data_dir.join("downloads.sqlite"))?);
            let executable = std::env::var_os("MYTORY_DOWNLOADER_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("yt-dlp"));
            let service = DownloadService::new(queue.clone(), executable);
            service.start_available()?;
            app.manage(AppState { queue, service });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enqueue_downloads,
            list_downloads,
            remove_download,
            cancel_download,
            retry_download,
            clear_history,
            open_download_folder,
            default_download_destination,
            get_download_concurrency,
            set_download_concurrency
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mytory YT-DLP");
}
