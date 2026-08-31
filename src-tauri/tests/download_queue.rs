use std::{
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use mytory_media_queue_lib::{
    DownloadQueue, DownloadStatus, DownloaderRequest, DownloaderRunner, OutputPreset,
};

#[test]
fn enqueues_multiple_urls_with_the_mp4_compatible_default_and_preserves_fifo_order() {
    let queue = DownloadQueue::in_memory().unwrap();
    let urls = vec![
        "https://example.test/one".into(),
        "https://example.test/two".into(),
    ];
    let jobs = queue.enqueue(&urls, Path::new("/downloads")).unwrap();
    assert_eq!(jobs.len(), 2);
    assert!(jobs
        .iter()
        .all(|job| job.output_preset == OutputPreset::Mp4Compatible
            && job.status == DownloadStatus::Queued));
    assert_eq!(
        queue
            .jobs()
            .unwrap()
            .into_iter()
            .map(|job| job.source_url)
            .collect::<Vec<_>>(),
        urls
    );
}

#[test]
fn enqueuing_many_urls_assigns_each_job_a_unique_identifier() {
    let queue = DownloadQueue::in_memory().unwrap();
    let urls = (0..128)
        .map(|index| format!("https://example.test/{index}"))
        .collect::<Vec<_>>();

    let jobs = queue.enqueue(&urls, Path::new("/downloads")).unwrap();
    let unique_ids = jobs
        .iter()
        .map(|job| &job.id)
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(jobs.len(), unique_ids.len());
}

#[test]
fn persists_each_output_preset() {
    let queue = DownloadQueue::in_memory().unwrap();
    for preset in [
        OutputPreset::Mp4Compatible,
        OutputPreset::BestVideo,
        OutputPreset::OriginalAudio,
        OutputPreset::Mp3_320,
    ] {
        let job = queue
            .enqueue_with_preset(
                &["https://example.test/video".into()],
                Path::new("/downloads"),
                preset.clone(),
            )
            .unwrap()
            .remove(0);
        assert_eq!(
            queue
                .jobs()
                .unwrap()
                .iter()
                .find(|saved| saved.id == job.id)
                .unwrap()
                .output_preset,
            preset
        );
    }
}

#[test]
fn persists_the_subtitle_flag_and_failure_details() {
    let queue = DownloadQueue::in_memory().unwrap();
    let job = queue
        .enqueue_with_options(
            &["https://example.test/video".into()],
            Path::new("/downloads"),
            OutputPreset::Mp4Compatible,
            true,
        )
        .unwrap()
        .remove(0);
    queue.mark_running(&job.id).unwrap();
    queue
        .mark_failed(
            &job.id,
            Some(mytory_media_queue_lib::DownloadFailureKind::Permission),
            Some("Destination is not writable.".into()),
        )
        .unwrap();

    let saved = queue.jobs().unwrap().remove(0);
    assert!(saved.write_subs);
    assert_eq!(
        saved.failure_kind,
        Some(mytory_media_queue_lib::DownloadFailureKind::Permission)
    );
    assert_eq!(
        saved.diagnostic_log.as_deref(),
        Some("Destination is not writable.")
    );
}

#[test]
fn retry_clears_failure_details_and_increments_the_attempt_count() {
    let queue = DownloadQueue::in_memory().unwrap();
    let job = queue
        .enqueue(
            &["https://example.test/video".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    queue.mark_running(&job.id).unwrap();
    queue
        .mark_failed(
            &job.id,
            Some(mytory_media_queue_lib::DownloadFailureKind::TransientNetwork),
            Some("Temporary network interruption.".into()),
        )
        .unwrap();
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Failed);

    queue.retry(&job.id).unwrap();

    let saved = queue.jobs().unwrap().remove(0);
    assert_eq!(saved.status, DownloadStatus::Queued);
    assert_eq!(saved.attempt_count, 1);
    assert_eq!(saved.failure_kind, None);
    assert_eq!(saved.diagnostic_log, None);
}

#[test]
fn clears_only_terminal_history_and_keeps_active_work() {
    let queue = DownloadQueue::in_memory().unwrap();
    let completed = queue
        .enqueue(
            &["https://example.test/done".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    queue.mark_running(&completed.id).unwrap();
    queue.mark_completed(&completed.id).unwrap();
    queue
        .enqueue(
            &["https://example.test/pending".into()],
            Path::new("/downloads"),
        )
        .unwrap();

    let removed = queue.clear_history().unwrap();

    assert_eq!(removed, 1);
    let remaining = queue.jobs().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].source_url, "https://example.test/pending");
}

#[test]
fn records_the_daily_managed_update_check_without_touching_download_jobs() {
    let queue = DownloadQueue::in_memory().unwrap();
    assert!(queue.managed_update_is_due().unwrap());
    queue.record_managed_update_check().unwrap();
    assert!(!queue.managed_update_is_due().unwrap());
    assert!(queue.jobs().unwrap().is_empty());
}

#[test]
fn accepts_more_urls_while_an_existing_job_is_running() {
    let queue = DownloadQueue::in_memory().unwrap();
    let first = queue
        .enqueue(
            &["simulator://slow-success".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    queue.mark_running(&first.id).unwrap();
    let runner = DownloaderRunner::new(PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator")));
    let request = DownloaderRequest::new("simulator://slow-success", "/downloads");
    thread::scope(|scope| {
        let running_download = scope.spawn(|| runner.run(&request).unwrap());
        thread::sleep(Duration::from_millis(100));
        queue
            .enqueue(
                &["https://example.test/new".into()],
                Path::new("/downloads"),
            )
            .unwrap();
        assert!(running_download.join().unwrap().succeeded);
    });
    let jobs = queue.jobs().unwrap();
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].status, DownloadStatus::Running);
    assert_eq!(jobs[1].status, DownloadStatus::Queued);
}

#[test]
fn resets_interrupted_running_jobs_to_queued_when_reopened() {
    let database = std::env::temp_dir().join(format!("mytory-queue-{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&database);
    let queue = DownloadQueue::open(&database).unwrap();
    let job = queue
        .enqueue(
            &["https://example.test/one".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    queue.mark_running(&job.id).unwrap();
    drop(queue);

    let reopened = DownloadQueue::open(&database).unwrap();
    assert_eq!(reopened.jobs().unwrap()[0].status, DownloadStatus::Queued);
    drop(reopened);
    std::fs::remove_file(database).unwrap();
}
