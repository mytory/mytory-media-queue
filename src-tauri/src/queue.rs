use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::migrate;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputPreset {
    Mp4Compatible,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Running,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct QueueJob {
    pub id: String,
    pub source_url: String,
    pub destination: String,
    pub output_preset: OutputPreset,
    pub status: DownloadStatus,
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
                "INSERT INTO download_jobs (id, source_url, destination, output_preset, status, created_at, updated_at) VALUES (?1, ?2, ?3, 'mp4_compatible', 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                params![id, url, destination],
            )?;
            jobs.push(QueueJob {
                id,
                source_url: url.clone(),
                destination: destination.clone(),
                output_preset: OutputPreset::Mp4Compatible,
                status: DownloadStatus::Queued,
            });
        }
        transaction.commit()?;
        Ok(jobs)
    }

    pub fn mark_running(&self, id: &str) -> Result<()> {
        self.connection.lock().expect("queue lock poisoned").execute(
            "UPDATE download_jobs SET status = 'running', updated_at = CURRENT_TIMESTAMP WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn jobs(&self) -> Result<Vec<QueueJob>> {
        let connection = self.connection.lock().expect("queue lock poisoned");
        let mut statement = connection.prepare("SELECT id, source_url, destination, output_preset, status FROM download_jobs ORDER BY created_at, rowid")?;
        let jobs = statement
            .query_map([], |row| {
                Ok(QueueJob {
                    id: row.get(0)?,
                    source_url: row.get(1)?,
                    destination: row.get(2)?,
                    output_preset: OutputPreset::Mp4Compatible,
                    status: match row.get::<_, String>(4)?.as_str() {
                        "running" => DownloadStatus::Running,
                        _ => DownloadStatus::Queued,
                    },
                })
            })?
            .collect();
        jobs
    }
}

fn valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("simulator://")
}
