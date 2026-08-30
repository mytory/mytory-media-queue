use std::path::Path;

use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

pub mod downloader;
pub mod queue;
pub mod storage;

pub use downloader::{
    DownloadFailureKind, DownloadRun, DownloaderError, DownloaderEvent, DownloaderRequest,
    DownloaderRunner,
};
pub use queue::{DownloadQueue, DownloadStatus, OutputPreset, QueueJob};
pub use storage::migrate;

struct AppState(DownloadQueue);

#[derive(Deserialize)]
struct EnqueueDownloadsRequest {
    urls: Vec<String>,
    destination: String,
    output_preset: Option<OutputPreset>,
}

#[tauri::command]
fn enqueue_downloads(
    request: EnqueueDownloadsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<QueueJob>, String> {
    state
        .0
        .enqueue_with_preset(
            &request.urls,
            Path::new(&request.destination),
            request.output_preset.unwrap_or(OutputPreset::Mp4Compatible),
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<QueueJob>, String> {
    state.0.jobs().map_err(|error| error.to_string())
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
    state.0.concurrency().map_err(|error| error.to_string())
}

#[tauri::command]
fn set_download_concurrency(concurrency: u8, state: State<'_, AppState>) -> Result<bool, String> {
    state
        .0
        .set_concurrency(concurrency)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            app.manage(AppState(DownloadQueue::open(
                data_dir.join("downloads.sqlite"),
            )?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enqueue_downloads,
            list_downloads,
            default_download_destination,
            get_download_concurrency,
            set_download_concurrency
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mytory YT-DLP");
}
