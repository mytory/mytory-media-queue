use std::{path::PathBuf, sync::Arc, thread};

use crate::{DownloadQueue, DownloaderEvent, DownloaderRequest, DownloaderRunner};

#[derive(Clone)]
pub struct DownloadService {
    queue: Arc<DownloadQueue>,
    executable: PathBuf,
}

impl DownloadService {
    pub fn new(queue: Arc<DownloadQueue>, executable: impl Into<PathBuf>) -> Self {
        Self {
            queue,
            executable: executable.into(),
        }
    }

    pub fn start_available(&self) -> rusqlite::Result<()> {
        for job in self.queue.start_available()? {
            let service = self.clone();
            thread::spawn(move || {
                let queue = service.queue.clone();
                let job_id = job.id.clone();
                let result = DownloaderRunner::new(&service.executable).run_with_events(
                    &DownloaderRequest::new(&job.source_url, &job.destination)
                        .with_preset(job.output_preset.clone()),
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
                let transition = if matches!(result, Ok(ref run) if run.succeeded) {
                    service.queue.mark_completed(&job.id)
                } else {
                    service.queue.mark_failed(&job.id)
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
}
