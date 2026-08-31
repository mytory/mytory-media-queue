use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use mytory_media_queue_lib::{
    DownloadRun, DownloaderCommand, DownloaderEvent, DownloaderRequest, DownloaderRunner,
};

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
            diagnostic_log: None,
        }
    );
}

#[test]
fn runs_yt_dlp_from_the_bundled_python_environment_with_the_bundled_runtime() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let command = DownloaderCommand::bundled_python(
        simulator,
        "/tools/current/yt-dlp.whl",
        "/tools/current/yt-dlp-ejs.whl",
        "/resources/deno",
    );
    let request = DownloaderRequest::new("simulator://bundled-python", "/downloads");

    assert!(
        DownloaderRunner::with_command(command)
            .run(&request)
            .unwrap()
            .succeeded
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
fn accepts_yt_dlp_decimal_speed_and_eta_fields() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://decimal-progress", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(matches!(
        run.events.as_slice(),
        [
            DownloaderEvent::Started { .. },
            DownloaderEvent::Progress {
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
fn cancels_a_running_downloader_process() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://slow-success", "/downloads");
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancellation_signal = cancelled.clone();
    let started_at = Instant::now();

    let run = DownloaderRunner::new(simulator)
        .run_with_cancellation(&request, cancelled, |event| {
            if matches!(event, DownloaderEvent::Progress { .. }) {
                cancellation_signal.store(true, std::sync::atomic::Ordering::Release);
            }
        })
        .unwrap();

    assert!(!run.succeeded);
    assert!(started_at.elapsed() < Duration::from_secs(1));
}

#[test]
fn writes_thumbnails_and_subtitles_only_when_requested() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));

    let thumbnail = DownloaderRequest::new("simulator://success", "/downloads");
    assert!(
        DownloaderRunner::new(simulator.clone())
            .run(&thumbnail)
            .unwrap()
            .succeeded
    );

    let with_subs = DownloaderRequest::new("simulator://subs", "/downloads").with_subtitles(true);
    assert!(
        DownloaderRunner::new(simulator.clone())
            .run(&with_subs)
            .unwrap()
            .succeeded
    );
}

#[test]
fn prefers_h264_and_aac_for_the_mp4_compatible_preset() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://mp4-compatible", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(run.succeeded);
}

#[test]
fn passes_ffmpeg_location_to_the_downloader() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://ffmpeg-location", "/downloads")
        .with_ffmpeg_location(Some(PathBuf::from("/resources/binaries")));

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(run.succeeded);
}

#[test]
fn passes_cookie_file_only_to_the_downloader_process() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://cookies", "/downloads")
        .with_cookies(Some(PathBuf::from("/private/cookies.txt")));

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(run.succeeded);
    assert!(!format!("{run:?}").contains("/private/cookies.txt"));
}

#[test]
fn classifies_raw_network_errors_as_transient_with_a_refined_diagnostic() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://raw-network-error", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(!run.succeeded);
    assert!(matches!(
        run.events.last(),
        Some(DownloaderEvent::Failed {
            kind: mytory_media_queue_lib::DownloadFailureKind::TransientNetwork,
            message,
        }) if message.contains("Name or service not known")
    ));
    assert!(run
        .diagnostic_log
        .as_deref()
        .is_some_and(|log| log.contains("Name or service not known")));
}

#[test]
fn classifies_raw_permission_errors_without_retaining_raw_secrets() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://raw-permission-error", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(!run.succeeded);
    assert!(matches!(
        run.events.last(),
        Some(DownloaderEvent::Failed {
            kind: mytory_media_queue_lib::DownloadFailureKind::Permission,
            ..
        })
    ));
    assert!(run
        .diagnostic_log
        .as_deref()
        .is_some_and(|log| log.contains("Permission denied")));
}

#[test]
fn classifies_unrecognized_errors_as_unknown() {
    let simulator = PathBuf::from(env!("CARGO_BIN_EXE_downloader-simulator"));
    let request = DownloaderRequest::new("simulator://raw-unknown-error", "/downloads");

    let run = DownloaderRunner::new(simulator).run(&request).unwrap();

    assert!(!run.succeeded);
    assert!(matches!(
        run.events.last(),
        Some(DownloaderEvent::Failed {
            kind: mytory_media_queue_lib::DownloadFailureKind::Unknown,
            message,
        }) if message.contains("Video unavailable")
    ));
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
                kind: mytory_media_queue_lib::DownloadFailureKind::Interrupted,
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
            mytory_media_queue_lib::DownloadFailureKind::TransientNetwork,
            "Temporary network interruption.",
        ),
        (
            "simulator://permission-failure",
            mytory_media_queue_lib::DownloadFailureKind::Permission,
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
