use std::path::Path;

use serde::Deserialize;
use tauri::{Manager, State};

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
}

#[tauri::command]
fn enqueue_downloads(
    request: EnqueueDownloadsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<QueueJob>, String> {
    state
        .0
        .enqueue(&request.urls, Path::new(&request.destination))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<QueueJob>, String> {
    state.0.jobs().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            app.manage(AppState(DownloadQueue::open(
                data_dir.join("downloads.sqlite"),
            )?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![enqueue_downloads, list_downloads])
        .run(tauri::generate_context!())
        .expect("error while running Mytory YT-DLP");
}
