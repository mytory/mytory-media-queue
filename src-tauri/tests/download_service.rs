use std::{path::Path, sync::Arc, thread, time::Duration};

use mytory_media_queue_lib::{DownloadQueue, DownloadService, DownloadStatus};

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

#[test]
fn removes_a_running_download_and_stops_its_process() {
    let queue = Arc::new(DownloadQueue::in_memory().unwrap());
    let job = queue
        .enqueue(
            &["simulator://slow-success".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let service = DownloadService::new(queue.clone(), simulator);
    service.start_available().unwrap();

    for _ in 0..30 {
        if queue.jobs().unwrap()[0].status == DownloadStatus::Running {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Running);

    service.remove(&job.id).unwrap();
    assert!(queue.jobs().unwrap().is_empty());

    thread::sleep(Duration::from_millis(2100));
    assert!(queue.jobs().unwrap().is_empty());
}

#[test]
fn retries_transient_network_failures_up_to_three_times_then_fails() {
    let queue = Arc::new(DownloadQueue::in_memory().unwrap());
    queue
        .enqueue(
            &["simulator://transient-network-failure".into()],
            Path::new("/downloads"),
        )
        .unwrap();
    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let service = DownloadService::new(queue.clone(), simulator);
    service.start_available().unwrap();

    for _ in 0..200 {
        let job = &queue.jobs().unwrap()[0];
        if job.status == DownloadStatus::Failed {
            assert_eq!(job.attempt_count, 3);
            assert_eq!(
                job.failure_kind,
                Some(mytory_media_queue_lib::DownloadFailureKind::TransientNetwork)
            );
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("transient failure was not retried three times and then failed");
}

#[test]
fn does_not_retry_permanent_failures() {
    let queue = Arc::new(DownloadQueue::in_memory().unwrap());
    queue
        .enqueue(
            &["simulator://permission-failure".into()],
            Path::new("/downloads"),
        )
        .unwrap();
    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let service = DownloadService::new(queue.clone(), simulator);
    service.start_available().unwrap();

    for _ in 0..100 {
        let job = &queue.jobs().unwrap()[0];
        if job.status == DownloadStatus::Failed {
            assert_eq!(job.attempt_count, 0);
            assert_eq!(
                job.failure_kind,
                Some(mytory_media_queue_lib::DownloadFailureKind::Permission)
            );
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("permission failure was unexpectedly retried");
}

#[test]
fn cancels_a_running_download_without_rescheduling_it() {
    let queue = Arc::new(DownloadQueue::in_memory().unwrap());
    let job = queue
        .enqueue(
            &["simulator://slow-success".into()],
            Path::new("/downloads"),
        )
        .unwrap()
        .remove(0);
    let simulator = std::path::PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let service = DownloadService::new(queue.clone(), simulator);
    service.start_available().unwrap();

    for _ in 0..30 {
        if queue.jobs().unwrap()[0].status == DownloadStatus::Running {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Running);

    service.cancel(&job.id).unwrap();

    for _ in 0..100 {
        if queue.jobs().unwrap()[0].status == DownloadStatus::Cancelled {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Cancelled);
    thread::sleep(Duration::from_millis(2100));
    assert_eq!(queue.jobs().unwrap()[0].status, DownloadStatus::Cancelled);
}
