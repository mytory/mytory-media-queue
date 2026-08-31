use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::downloader::DownloadFailureKind;
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
    pub write_subs: bool,
    pub status: DownloadStatus,
    pub progress_percent: Option<f64>,
    pub speed_bytes_per_second: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub attempt_count: u32,
    pub failure_kind: Option<DownloadFailureKind>,
    pub diagnostic_log: Option<String>,
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
        self.enqueue_with_options(urls, destination, OutputPreset::Mp4Compatible, false)
    }

    pub fn enqueue_with_preset(
        &self,
        urls: &[String],
        destination: &Path,
        output_preset: OutputPreset,
    ) -> Result<Vec<QueueJob>> {
        self.enqueue_with_options(urls, destination, output_preset, false)
    }

    pub fn enqueue_with_options(
        &self,
        urls: &[String],
        destination: &Path,
        output_preset: OutputPreset,
        write_subs: bool,
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
                "INSERT INTO download_jobs (id, source_url, destination, output_preset, write_subs, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![id, url, destination, output_preset.database_value(), write_subs],
            )?;
            jobs.push(QueueJob {
                id,
                source_url: url.clone(),
                destination: destination.clone(),
                output_preset: output_preset.clone(),
                write_subs,
                status: DownloadStatus::Queued,
                progress_percent: None,
                speed_bytes_per_second: None,
                eta_seconds: None,
                attempt_count: 0,
                failure_kind: None,
                diagnostic_log: None,
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

    pub fn has_running_work(&self) -> Result<bool> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM download_jobs WHERE status = 'running')",
            [],
            |row| row.get(0),
        )
    }

    pub fn managed_update_is_due(&self) -> Result<bool> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        connection.query_row(
            "SELECT NOT EXISTS(SELECT 1 FROM app_settings WHERE key = 'managed_update_last_checked_at' AND updated_at >= datetime('now', '-1 day'))",
            [],
            |row| row.get(0),
        )
    }

    pub fn record_managed_update_check(&self) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('managed_update_last_checked_at', 'checked', CURRENT_TIMESTAMP) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            [],
        )?;
        Ok(())
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
        let mut statement = connection.prepare("SELECT id, source_url, destination, output_preset, write_subs, attempt_count FROM download_jobs WHERE status = 'queued' ORDER BY created_at, rowid LIMIT ?1")?;
        let queued = statement
            .query_map([slots], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, bool>(4)?,
                    row.get::<_, u32>(5)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;
        let transaction = connection.unchecked_transaction()?;
        let mut jobs = Vec::with_capacity(queued.len());
        for (id, source_url, destination, output_preset, write_subs, attempt_count) in queued {
            transaction.execute("UPDATE download_jobs SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'queued'", [&id])?;
            jobs.push(QueueJob {
                id,
                source_url,
                destination,
                output_preset: OutputPreset::from_database(&output_preset),
                write_subs,
                status: DownloadStatus::Running,
                progress_percent: None,
                speed_bytes_per_second: None,
                eta_seconds: None,
                attempt_count,
                failure_kind: None,
                diagnostic_log: None,
            });
        }
        transaction.commit()?;
        Ok(jobs)
    }

    pub fn update_progress(
        &self,
        id: &str,
        percent: Option<f32>,
        speed_bytes_per_second: Option<u64>,
        eta_seconds: Option<u64>,
    ) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET progress_percent = ?2, speed_bytes_per_second = ?3, eta_seconds = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'running'",
            params![id, percent.map(f64::from), speed_bytes_per_second.map(|value| value as i64), eta_seconds.map(|value| value as i64)],
        )?;
        Ok(())
    }

    pub fn mark_completed(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute("UPDATE download_jobs SET status = 'completed', completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'running'", [id])?;
        Ok(())
    }

    pub fn mark_failed(
        &self,
        id: &str,
        failure_kind: Option<DownloadFailureKind>,
        diagnostic_log: Option<String>,
    ) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET status = 'failed', failure_kind = ?2, diagnostic_log = ?3, completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'running'",
            params![id, failure_kind.map(|kind| kind.database_value()), diagnostic_log],
        )?;
        Ok(())
    }

    pub fn cancel(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute("UPDATE download_jobs SET status = 'cancelled', cancelled_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status IN ('queued', 'running')", [id])?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "DELETE FROM download_jobs WHERE id = ?1 AND status IN ('queued', 'running', 'failed', 'cancelled')",
            [id],
        )?;
        Ok(())
    }

    pub fn retry(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute("UPDATE download_jobs SET status = 'queued', attempt_count = attempt_count + 1, failure_kind = NULL, diagnostic_log = NULL, cancelled_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status IN ('failed', 'cancelled')", [id])?;
        Ok(())
    }

    pub fn mark_retry_from_running(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET status = 'queued', attempt_count = attempt_count + 1, failure_kind = NULL, diagnostic_log = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'running'",
            [id],
        )?;
        Ok(())
    }

    pub fn prune_completed_history(&self) -> Result<usize> {
        self.connection.lock().expect("queue lock poisoned").execute("DELETE FROM download_jobs WHERE status = 'completed' AND completed_at < datetime('now', '-90 days')", [])
    }

    pub fn clear_history(&self) -> Result<usize> {
        self.connection
            .lock()
            .expect("queue lock poisoned")
            .execute(
                "DELETE FROM download_jobs WHERE status IN ('completed', 'failed', 'cancelled')",
                [],
            )
    }

    pub fn mark_running(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?1 AND status = 'queued'", [id])?;
        Ok(())
    }

    pub fn jobs(&self) -> Result<Vec<QueueJob>> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        let mut statement = connection.prepare("SELECT id, source_url, destination, output_preset, write_subs, status, progress_percent, speed_bytes_per_second, eta_seconds, attempt_count, failure_kind, diagnostic_log FROM download_jobs ORDER BY created_at, rowid")?;
        let jobs = statement
            .query_map([], |row| {
                Ok(QueueJob {
                    id: row.get(0)?,
                    source_url: row.get(1)?,
                    destination: row.get(2)?,
                    output_preset: OutputPreset::from_database(&row.get::<_, String>(3)?),
                    write_subs: row.get(4)?,
                    status: match row.get::<_, String>(5)?.as_str() {
                        "running" => DownloadStatus::Running,
                        "completed" => DownloadStatus::Completed,
                        "failed" => DownloadStatus::Failed,
                        "cancelled" => DownloadStatus::Cancelled,
                        _ => DownloadStatus::Queued,
                    },
                    progress_percent: row.get(6)?,
                    speed_bytes_per_second: row.get::<_, Option<i64>>(7)?.map(|value| value as u64),
                    eta_seconds: row.get::<_, Option<i64>>(8)?.map(|value| value as u64),
                    attempt_count: row.get(9)?,
                    failure_kind: row
                        .get::<_, Option<String>>(10)?
                        .map(|value| DownloadFailureKind::from_database(&value)),
                    diagnostic_log: row.get(11)?,
                })
            })?
            .collect();
        jobs
    }
}

fn valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("simulator://")
}
