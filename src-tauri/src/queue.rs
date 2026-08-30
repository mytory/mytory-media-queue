use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::migrate;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputPreset {
    Mp4Compatible,
    BestVideo,
    OriginalAudio,
    Mp3_320,
}

impl OutputPreset {
    fn database_value(&self) -> &'static str {
        match self {
            Self::Mp4Compatible => "mp4_compatible",
            Self::BestVideo => "best_video",
            Self::OriginalAudio => "original_audio",
            Self::Mp3_320 => "mp3_320",
        }
    }
    fn from_database(value: &str) -> Self {
        match value {
            "best_video" => Self::BestVideo,
            "original_audio" => Self::OriginalAudio,
            "mp3_320" => Self::Mp3_320,
            _ => Self::Mp4Compatible,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct QueueJob {
    pub id: String,
    pub source_url: String,
    pub destination: String,
    pub output_preset: OutputPreset,
    pub status: DownloadStatus,
    pub progress_percent: Option<f64>,
    pub speed_bytes_per_second: Option<u64>,
    pub eta_seconds: Option<u64>,
}

pub struct DownloadQueue {
    connection: Mutex<Connection>,
}

impl DownloadQueue {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut connection = Connection::open(path)?;
        migrate(&mut connection)?;
        connection.execute("UPDATE download_jobs SET status = 'queued', updated_at = CURRENT_TIMESTAMP WHERE status = 'running'", [])?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn in_memory() -> Result<Self> {
        let mut connection = Connection::open_in_memory()?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn enqueue(&self, urls: &[String], destination: &Path) -> Result<Vec<QueueJob>> {
        self.enqueue_with_preset(urls, destination, OutputPreset::Mp4Compatible)
    }

    pub fn enqueue_with_preset(
        &self,
        urls: &[String],
        destination: &Path,
        output_preset: OutputPreset,
    ) -> Result<Vec<QueueJob>> {
        let destination = destination.to_string_lossy().into_owned();
        let connection = self.connection.lock().expect("queue lock poisoned");
        let transaction = connection.unchecked_transaction()?;
        let mut jobs = Vec::with_capacity(urls.len());
        for url in urls {
            if !valid_url(url) {
                continue;
            }
            let id = format!("job-{}", Uuid::new_v4());
            transaction.execute(
                "INSERT INTO download_jobs (id, source_url, destination, output_preset, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![id, url, destination, output_preset.database_value()],
            )?;
            jobs.push(QueueJob {
                id,
                source_url: url.clone(),
                destination: destination.clone(),
                output_preset: output_preset.clone(),
                status: DownloadStatus::Queued,
                progress_percent: None,
                speed_bytes_per_second: None,
                eta_seconds: None,
            });
        }
        transaction.commit()?;
        Ok(jobs)
    }

    pub fn concurrency(&self) -> Result<u8> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        let value = connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'download_concurrency'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(value
            .and_then(|value| value.parse().ok())
            .filter(|value| (1..=5).contains(value))
            .unwrap_or(3))
    }

    pub fn set_concurrency(&self, concurrency: u8) -> Result<bool> {
        if !(1..=5).contains(&concurrency) {
            return Ok(false);
        }
        self.connection.lock().expect("queue lock poisoned").execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('download_concurrency', ?1, CURRENT_TIMESTAMP) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            [concurrency.to_string()],
        )?;
        Ok(true)
    }

    pub fn start_available(&self) -> Result<Vec<QueueJob>> {
        let concurrency = self.concurrency()? as usize;
        let connection = self.connection.lock().expect("queue lock poisoned");
        let running: i64 = connection.query_row(
            "SELECT COUNT(*) FROM download_jobs WHERE status = 'running'",
            [],
            |row| row.get(0),
        )?;
        let slots = (concurrency as i64 - running).max(0);
        if slots == 0 {
            return Ok(Vec::new());
        }
        let mut statement = connection.prepare("SELECT id, source_url, destination, output_preset FROM download_jobs WHERE status = 'queued' ORDER BY created_at, rowid LIMIT ?1")?;
        let queued = statement
            .query_map([slots], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;
        let transaction = connection.unchecked_transaction()?;
        let mut jobs = Vec::with_capacity(queued.len());
        for (id, source_url, destination, output_preset) in queued {
            transaction.execute("UPDATE download_jobs SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'queued'", [&id])?;
            jobs.push(QueueJob {
                id,
                source_url,
                destination,
                output_preset: OutputPreset::from_database(&output_preset),
                status: DownloadStatus::Running,
                progress_percent: None,
                speed_bytes_per_second: None,
                eta_seconds: None,
            });
        }
        transaction.commit()?;
        Ok(jobs)
    }

    pub fn cancel(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute("UPDATE download_jobs SET status = 'cancelled', cancelled_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status IN ('queued', 'running')", [id])?;
        Ok(())
    }

    pub fn retry(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute("UPDATE download_jobs SET status = 'queued', attempt_count = attempt_count + 1, failure_kind = NULL, diagnostic_log = NULL, cancelled_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status IN ('failed', 'cancelled')", [id])?;
        Ok(())
    }

    pub fn prune_completed_history(&self) -> Result<usize> {
        self.connection.lock().expect("queue lock poisoned").execute("DELETE FROM download_jobs WHERE status = 'completed' AND completed_at < datetime('now', '-90 days')", [])
    }

    pub fn mark_running(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'queued'", [id])?;
        Ok(())
    }

    pub fn jobs(&self) -> Result<Vec<QueueJob>> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        let mut statement = connection.prepare("SELECT id, source_url, destination, output_preset, status, progress_percent, speed_bytes_per_second, eta_seconds FROM download_jobs ORDER BY created_at, rowid")?;
        let jobs = statement
            .query_map([], |row| {
                Ok(QueueJob {
                    id: row.get(0)?,
                    source_url: row.get(1)?,
                    destination: row.get(2)?,
                    output_preset: OutputPreset::from_database(&row.get::<_, String>(3)?),
                    status: match row.get::<_, String>(4)?.as_str() {
                        "running" => DownloadStatus::Running,
                        "completed" => DownloadStatus::Completed,
                        "failed" => DownloadStatus::Failed,
                        "cancelled" => DownloadStatus::Cancelled,
                        _ => DownloadStatus::Queued,
                    },
                    progress_percent: row.get(5)?,
                    speed_bytes_per_second: row.get::<_, Option<i64>>(6)?.map(|value| value as u64),
                    eta_seconds: row.get::<_, Option<i64>>(7)?.map(|value| value as u64),
                })
            })?
            .collect();
        jobs
    }
}

fn valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("simulator://")
}
