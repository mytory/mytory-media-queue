use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    DownloadFailureKind, DownloadQueue, DownloaderEvent, DownloaderRequest, DownloaderRunner,
};

const MAX_AUTO_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct DownloadService {
    queue: Arc<DownloadQueue>,
    executable: PathBuf,
    ffmpeg_dir: Option<PathBuf>,
    cancellations: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    cookie_sources: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl DownloadService {
    pub fn new(queue: Arc<DownloadQueue>, executable: impl Into<PathBuf>) -> Self {
        Self {
            queue,
            executable: executable.into(),
            ffmpeg_dir: None,
            cancellations: Arc::new(Mutex::new(HashMap::new())),
            cookie_sources: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_ffmpeg_location(mut self, location: Option<PathBuf>) -> Self {
        self.ffmpeg_dir = location;
        self
    }

    pub fn remember_cookie_source(&self, job_id: &str, cookies: PathBuf) {
        self.cookie_sources
            .lock()
            .expect("cookie source lock poisoned")
            .insert(job_id.to_owned(), cookies);
    }

    pub fn start_available(&self) -> rusqlite::Result<()> {
        for job in self.queue.start_available()? {
            let cancellation = Arc::new(AtomicBool::new(false));
            self.cancellations
                .lock()
                .expect("cancellation lock poisoned")
                .insert(job.id.clone(), cancellation.clone());
            let cookies = self
                .cookie_sources
                .lock()
                .expect("cookie source lock poisoned")
                .remove(&job.id);
            let service = self.clone();
            thread::spawn(move || {
                let queue = service.queue.clone();
                let job_id = job.id.clone();
                let request = DownloaderRequest::new(&job.source_url, &job.destination)
                    .with_preset(job.output_preset.clone())
                    .with_subtitles(job.write_subs)
                    .with_cookies(cookies)
                    .with_ffmpeg_location(service.ffmpeg_dir.clone());
                let result = DownloaderRunner::new(&service.executable).run_with_cancellation(
                    &request,
                    cancellation,
                    |event| {
                        if let DownloaderEvent::Progress {
                            percent,
                            speed_bytes_per_second,
                            eta_seconds,
                            ..
                        } = event
                        {
                            let _ = queue.update_progress(
                                &job_id,
                                *percent,
                                *speed_bytes_per_second,
                                *eta_seconds,
                            );
                        }
                    },
                );
                service
                    .cancellations
                    .lock()
                    .expect("cancellation lock poisoned")
                    .remove(&job.id);
                let transition = match result {
                    Ok(run) if run.succeeded => service.queue.mark_completed(&job.id),
                    Ok(run) => {
                        let (kind, diagnostic) = terminal_failure(&run);
                        if kind == DownloadFailureKind::TransientNetwork
                            && job.attempt_count < MAX_AUTO_RETRIES
                        {
                            service.queue.mark_retry_from_running(&job.id)
                        } else {
                            service.queue.mark_failed(&job.id, Some(kind), diagnostic)
                        }
                    }
                    Err(error) => service.queue.mark_failed(
                        &job.id,
                        Some(DownloadFailureKind::Unknown),
                        Some(error.to_string()),
                    ),
                };
                if let Err(error) = transition {
                    eprintln!(
                        "could not persist terminal download state for {}: {error}",
                        job.id
                    );
                    return;
                }
                if let Err(error) = service.start_available() {
                    eprintln!("could not start queued downloads: {error}");
                }
            });
        }
        Ok(())
    }

    pub fn retry(&self, id: &str) -> rusqlite::Result<()> {
        self.queue.retry(id)?;
        self.start_available()
    }

    pub fn cancel(&self, id: &str) -> rusqlite::Result<()> {
        if let Some(cancellation) = self
            .cancellations
            .lock()
            .expect("cancellation lock poisoned")
            .get(id)
        {
            cancellation.store(true, Ordering::Release);
        }
        self.queue.cancel(id)?;
        self.start_available()
    }

    pub fn remove(&self, id: &str) -> rusqlite::Result<()> {
        if let Some(cancellation) = self
            .cancellations
            .lock()
            .expect("cancellation lock poisoned")
            .get(id)
        {
            cancellation.store(true, Ordering::Release);
        }
        self.queue.remove(id)?;
        self.start_available()
    }

    pub fn clear_history(&self) -> rusqlite::Result<usize> {
        self.queue.clear_history()
    }

    pub fn destination_of(&self, id: &str) -> Option<PathBuf> {
        self.queue
            .jobs()
            .ok()?
            .into_iter()
            .find(|job| job.id == id)
            .map(|job| PathBuf::from(job.destination))
    }
}

fn terminal_failure(run: &crate::DownloadRun) -> (DownloadFailureKind, Option<String>) {
    let kind = match run.events.last() {
        Some(DownloaderEvent::Failed { kind, .. }) => kind.clone(),
        _ => DownloadFailureKind::Unknown,
    };
    let diagnostic = run
        .diagnostic_log
        .clone()
        .or_else(|| match run.events.last() {
            Some(DownloaderEvent::Failed { message, .. }) => Some(message.clone()),
            _ => None,
        });
    (kind, diagnostic)
}
