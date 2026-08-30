CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE download_jobs (
  id TEXT PRIMARY KEY,
  source_url TEXT NOT NULL,
  destination TEXT NOT NULL,
  output_preset TEXT NOT NULL,
  status TEXT NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  diagnostic_log TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  completed_at TEXT
);

CREATE INDEX download_jobs_status_created_at
  ON download_jobs (status, created_at);
