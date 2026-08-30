use std::{path::Path, sync::Arc, thread, time::Duration};

use mytory_yt_dlp_lib::{DownloadQueue, DownloadService, DownloadStatus};

#[test]
fn starts_queued_work_and_persists_completion() {
    let queue = Arc::new(DownloadQueue::in_memory().unwrap());
    queue
        .enqueue(&["simulator://success".into()], Path::new("/downloads"))
        .unwrap();
    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    DownloadService::new(queue.clone(), simulator)
        .start_available()
        .unwrap();

    for _ in 0..30 {
        if queue.jobs().unwrap()[0].status == DownloadStatus::Completed {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("download did not complete");
}
