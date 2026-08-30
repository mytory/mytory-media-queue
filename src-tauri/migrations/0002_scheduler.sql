ALTER TABLE download_jobs ADD COLUMN progress_percent REAL;
ALTER TABLE download_jobs ADD COLUMN speed_bytes_per_second INTEGER;
ALTER TABLE download_jobs ADD COLUMN eta_seconds INTEGER;
ALTER TABLE download_jobs ADD COLUMN failure_kind TEXT;
ALTER TABLE download_jobs ADD COLUMN cancelled_at TEXT;
