use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use mytory_yt_dlp_lib::{DownloadRun, DownloaderEvent, DownloaderRequest, DownloaderRunner};

#[test]
fn reports_progress_and_completion_from_the_downloader_simulator() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let destination = PathBuf::from("/downloads");
    let request = DownloaderRequest::new("simulator://success", &destination);

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert_eq!(
        run,
        DownloadRun {
            events: vec![
                DownloaderEvent::Started {
                    url: "simulator://success".into(),
                },
                DownloaderEvent::Progress {
                    downloaded_bytes: 524_288,
                    total_bytes: Some(1_048_576),
                    percent: Some(50.0),
                    speed_bytes_per_second: Some(1_048_576),
                    eta_seconds: Some(12),
                },
                DownloaderEvent::Succeeded { destination },
            ],
            succeeded: true,
        }
    );
}

#[test]
fn accepts_raw_yt_dlp_progress_fields_when_the_total_size_is_only_an_estimate() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://estimated-progress", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(matches!(
        run.events.as_slice(),
        [
            DownloaderEvent::Started { .. },
            DownloaderEvent::Progress {
                percent: Some(50.0),
                speed_bytes_per_second: Some(1_048_576),
                eta_seconds: Some(12),
                ..
            },
            DownloaderEvent::Succeeded { .. }
        ]
    ));
}

#[test]
fn emits_progress_before_the_downloader_process_exits() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://slow-success", "/downloads");
    let started_at = Instant::now();
    let mut progress_at = None;

    let run = DownloaderRunner::new(simulator)
        .run_with_events(&request, |event| {
            if matches!(event, DownloaderEvent::Progress { .. }) {
                progress_at = Some(started_at.elapsed());
            }
        })
        .unwrap();

    assert!(run.succeeded);
    assert!(progress_at.is_some_and(|elapsed| elapsed < Duration::from_secs(1)));
}

#[test]
fn reports_a_deterministic_interruption_from_the_downloader_simulator() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://interrupted", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(!run.succeeded);
    assert!(matches!(
        run.events.as_slice(),
        [
            DownloaderEvent::Started { .. },
            DownloaderEvent::Progress { .. },
            DownloaderEvent::Failed {
                kind: mytory_yt_dlp_lib::DownloadFailureKind::Interrupted,
                message,
            }
        ] if message == "Download interrupted."
    ));
}

#[test]
fn classifies_safe_simulator_failures_without_retaining_raw_stderr() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));

    for (scenario, kind, message) in [
        (
            "simulator://transient-network-failure",
            mytory_yt_dlp_lib::DownloadFailureKind::TransientNetwork,
            "Temporary network interruption.",
        ),
        (
            "simulator://permission-failure",
            mytory_yt_dlp_lib::DownloadFailureKind::Permission,
            "Destination is not writable.",
        ),
    ] {
        let request = DownloaderRequest::new(scenario, "/downloads");
        let run = DownloaderRunner::new(simulator.clone())
            .run(&request)
            .unwrap();

        assert!(!run.succeeded);
        assert!(matches!(
            run.events.last(),
            Some(DownloaderEvent::Failed {
                kind: actual_kind,
                message: actual_message,
            }) if *actual_kind == kind && actual_message == message
        ));
        assert!(!format!("{run:?}").contains("MYTORY_TEST_COOKIE=not-a-real-cookie"));
    }
}
