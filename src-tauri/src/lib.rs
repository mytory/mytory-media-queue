pub mod downloader;
pub mod storage;

pub use downloader::{
    DownloadFailureKind, DownloadRun, DownloaderError, DownloaderEvent, DownloaderRequest,
    DownloaderRunner,
};
pub use storage::migrate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Mytory YT-DLP");
}
