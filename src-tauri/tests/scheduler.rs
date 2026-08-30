use std::path::Path;

use mytory_yt_dlp_lib::{DownloadQueue, DownloadStatus, DownloaderRequest, DownloaderRunner};

#[test]
fn starts_jobs_in_fifo_order_up_to_the_configured_concurrency() {
    let queue = DownloadQueue::in_memory().unwrap();
    queue.set_concurrency(2).unwrap();
    queue
        .enqueue(
            &[
                "simulator://success".into(),
                "simulator://slow-success".into(),
                "https://example.test/third".into(),
            ],
            Path::new("/downloads"),
        )
        .unwrap();

    let started = queue.start_available().unwrap();

    assert_eq!(started.len(), 2);
    assert_eq!(started[0].source_url, "simulator://success");
    assert_eq!(started[1].source_url, "simulator://slow-success");
    assert!(queue.start_available().unwrap().is_empty());

    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let run = DownloaderRunner::new(simulator)
        .run(&DownloaderRequest::new(
            &started[0].source_url,
            &started[0].destination,
        ))
        .unwrap();
    assert!(run.succeeded);
}

#[test]
fn enforces_the_one_through_five_concurrency_range_and_supports_cancel_retry() {
    let queue = DownloadQueue::in_memory().unwrap();
    assert!(!queue.set_concurrency(0).unwrap());
    assert!(!queue.set_concurrency(6).unwrap());
    assert_eq!(queue.concurrency().unwrap(), 3);
    assert!(queue.set_concurrency(1).unwrap());

    let job = queue
        .enqueue(&["simulator://success".into()], Path::new("/downloads"))
        .unwrap()
        .remove(0);
    queue.cancel(&job.id).unwrap();
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Cancelled);
    queue.retry(&job.id).unwrap();
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Queued);
}
