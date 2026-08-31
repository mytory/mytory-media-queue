use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, SyncSender},
        Arc,
    },
    thread,
};

const PROGRESS_PREFIX: &str = "MYTORY_PROGRESS:";
const FAILURE_PREFIX: &str = "MYTORY_FAILURE:";
const PROGRESS_TEMPLATE: &str = "download:MYTORY_PROGRESS:%(progress.downloaded_bytes)s:%(progress.total_bytes)s:%(progress.total_bytes_estimate)s:%(progress.speed)s:%(progress.eta)s";
const OUTPUT_CHANNEL_CAPACITY: usize = 16;
const CANCELLATION_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);
const MAX_STDERR_TAIL_LINES: usize = 30;
const MAX_DIAGNOSTIC_CHARS: usize = 4000;
// MP4 호환 우선: H.264 영상과 AAC 오디오를 먼저 고르고, 없으면 단계적으로 폴백한다.
const MP4_COMPATIBLE_FORMAT: &str =
    "bv*[vcodec^=avc1]+ba[acodec^=mp4a]/bv*[vcodec^=avc1]+ba/bv*+ba[acodec^=mp4a]/bv*+ba/b";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloaderRequest {
    pub url: String,
    pub destination: PathBuf,
    pub output_preset: crate::OutputPreset,
    pub write_thumbnail: bool,
    pub write_subs: bool,
    pub cookies: Option<PathBuf>,
    pub ffmpeg_location: Option<PathBuf>,
}

impl DownloaderRequest {
    pub fn new(url: impl Into<String>, destination: impl AsRef<Path>) -> Self {
        Self {
            url: url.into(),
            destination: destination.as_ref().to_path_buf(),
            output_preset: crate::OutputPreset::Mp4Compatible,
            write_thumbnail: true,
            write_subs: false,
            cookies: None,
            ffmpeg_location: None,
        }
    }

    pub fn with_preset(mut self, output_preset: crate::OutputPreset) -> Self {
        self.output_preset = output_preset;
        self
    }

    pub fn with_subtitles(mut self, write_subs: bool) -> Self {
        self.write_subs = write_subs;
        self
    }

    pub fn with_cookies(mut self, cookies: Option<PathBuf>) -> Self {
        self.cookies = cookies;
        self
    }

    pub fn with_ffmpeg_location(mut self, location: Option<PathBuf>) -> Self {
        self.ffmpeg_location = location;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DownloaderEvent {
    Started {
        url: String,
    },
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        percent: Option<f32>,
        speed_bytes_per_second: Option<u64>,
        eta_seconds: Option<u64>,
    },
    Succeeded {
        destination: PathBuf,
    },
    Failed {
        kind: DownloadFailureKind,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadFailureKind {
    TransientNetwork,
    Permission,
    Interrupted,
    Unknown,
}

impl DownloadFailureKind {
    pub fn database_value(&self) -> &'static str {
        match self {
            Self::TransientNetwork => "transient_network",
            Self::Permission => "permission",
            Self::Interrupted => "interrupted",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_database(value: &str) -> Self {
        match value {
            "transient_network" => Self::TransientNetwork,
            "permission" => Self::Permission,
            "interrupted" => Self::Interrupted,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DownloadRun {
    pub events: Vec<DownloaderEvent>,
    pub succeeded: bool,
    pub diagnostic_log: Option<String>,
}

#[derive(Debug)]
pub enum DownloaderError {
    Spawn {
        executable: PathBuf,
        source: io::Error,
    },
    ReadOutput {
        executable: PathBuf,
        source: io::Error,
    },
    Wait {
        executable: PathBuf,
        source: io::Error,
    },
    InvalidProgress {
        line: String,
    },
}

impl fmt::Display for DownloaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { executable, source } => {
                write!(
                    f,
                    "could not start downloader at {}: {source}",
                    executable.display()
                )
            }
            Self::ReadOutput { executable, source } => {
                write!(
                    f,
                    "could not read downloader output at {}: {source}",
                    executable.display()
                )
            }
            Self::Wait { executable, source } => {
                write!(
                    f,
                    "could not wait for downloader at {}: {source}",
                    executable.display()
                )
            }
            Self::InvalidProgress { line } => write!(f, "invalid downloader progress: {line}"),
        }
    }
}

impl Error for DownloaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. }
            | Self::ReadOutput { source, .. }
            | Self::Wait { source, .. } => Some(source),
            Self::InvalidProgress { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownloaderCommand {
    program: PathBuf,
    prefix_arguments: Vec<String>,
    environment: Vec<(String, String)>,
}

impl DownloaderCommand {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            prefix_arguments: Vec::new(),
            environment: Vec::new(),
        }
    }

    pub fn bundled_python(
        python: impl Into<PathBuf>,
        downloader: impl AsRef<Path>,
        extractor: impl AsRef<Path>,
        deno: impl AsRef<Path>,
    ) -> Self {
        let module_path = std::env::join_paths([downloader.as_ref(), extractor.as_ref()])
            .expect("wheel paths do not contain path-list separators")
            .to_string_lossy()
            .into_owned();
        Self {
            program: python.into(),
            prefix_arguments: vec![
                "-m".into(),
                "yt_dlp".into(),
                "--js-runtimes".into(),
                format!("deno:{}", deno.as_ref().display()),
            ],
            environment: vec![("PYTHONPATH".into(), module_path)],
        }
    }
}

#[derive(Clone, Debug)]
pub struct DownloaderRunner {
    command: DownloaderCommand,
}

impl DownloaderRunner {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self::with_command(DownloaderCommand::new(executable))
    }

    pub fn with_command(command: DownloaderCommand) -> Self {
        Self { command }
    }

    pub fn run(&self, request: &DownloaderRequest) -> Result<DownloadRun, DownloaderError> {
        self.run_with_events(request, |_| {})
    }

    pub fn run_with_events<F>(
        &self,
        request: &DownloaderRequest,
        on_event: F,
    ) -> Result<DownloadRun, DownloaderError>
    where
        F: FnMut(&DownloaderEvent),
    {
        self.run_with_cancellation(request, Arc::new(AtomicBool::new(false)), on_event)
    }

    pub fn run_with_cancellation<F>(
        &self,
        request: &DownloaderRequest,
        cancelled: Arc<AtomicBool>,
        mut on_event: F,
    ) -> Result<DownloadRun, DownloaderError>
    where
        F: FnMut(&DownloaderEvent),
    {
        let output_template = request.destination.join("%(title)s [%(id)s].%(ext)s");
        let mut command = Command::new(&self.command.program);
        command
            .args(&self.command.prefix_arguments)
            .envs(
                self.command
                    .environment
                    .iter()
                    .map(|(key, value)| (key, value)),
            )
            .args(["--newline", "--progress-template", PROGRESS_TEMPLATE, "-o"])
            .arg(output_template);
        if request.write_thumbnail {
            command.arg("--write-thumbnail");
        }
        if request.write_subs {
            command.args([
                "--write-subs",
                "--sub-langs",
                "ko,en",
                "--convert-subs",
                "vtt",
            ]);
        }
        if let Some(cookies) = &request.cookies {
            command.arg("--cookies").arg(cookies);
        }
        if let Some(location) = &request.ffmpeg_location {
            command.arg("--ffmpeg-location").arg(location);
        }
        match request.output_preset {
            crate::OutputPreset::Mp4Compatible => {
                command.args(["-f", MP4_COMPATIBLE_FORMAT, "--merge-output-format", "mp4"]);
            }
            crate::OutputPreset::BestVideo => {
                command.args(["-f", "bv*+ba/b"]);
            }
            crate::OutputPreset::OriginalAudio => {
                command.args(["-f", "ba/b"]);
            }
            crate::OutputPreset::Mp3_320 => {
                command.args([
                    "-f",
                    "ba/b",
                    "-x",
                    "--audio-format",
                    "mp3",
                    "--audio-quality",
                    "320K",
                ]);
            }
        }
        let mut child = command
            .arg("--")
            .arg(&request.url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| DownloaderError::Spawn {
                executable: self.command.program.clone(),
                source,
            })?;
        let stdout = child.stdout.take().expect("stdout is piped");
        let stderr = child.stderr.take().expect("stderr is piped");
        let (sender, receiver) = mpsc::sync_channel(OUTPUT_CHANNEL_CAPACITY);
        let stdout_reader = spawn_reader(stdout, sender.clone(), StreamKind::StandardOutput);
        let stderr_reader = spawn_reader(stderr, sender.clone(), StreamKind::StandardError);
        drop(sender);

        let mut events = Vec::new();
        let started = DownloaderEvent::Started {
            url: request.url.clone(),
        };
        on_event(&started);
        events.push(started);
        let mut safe_failure = None;
        let mut stderr_tail: Vec<String> = Vec::new();

        loop {
            let stream_line = match receiver.recv_timeout(CANCELLATION_POLL_INTERVAL) {
                Ok(stream_line) => stream_line,
                Err(RecvTimeoutError::Timeout) if cancelled.load(Ordering::Acquire) => {
                    let _ = child.kill();
                    break;
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            };
            match stream_line {
                StreamLine::StandardOutput(line) => {
                    if let Some(progress) = line.strip_prefix(PROGRESS_PREFIX) {
                        let event = match parse_progress(progress, &line) {
                            Ok(event) => event,
                            Err(error) => return stop_child(child, error),
                        };
                        on_event(&event);
                        events.push(event);
                    }
                }
                StreamLine::StandardError(line) => {
                    push_bounded(
                        &mut stderr_tail,
                        line.trim().to_owned(),
                        MAX_STDERR_TAIL_LINES,
                    );
                    if let Some(failure) = parse_failure_line(&line) {
                        safe_failure = Some(failure);
                    }
                }
                StreamLine::ReadError(source) => {
                    return stop_child(
                        child,
                        DownloaderError::ReadOutput {
                            executable: self.command.program.clone(),
                            source,
                        },
                    );
                }
            }
        }

        let status = child.wait().map_err(|source| DownloaderError::Wait {
            executable: self.command.program.clone(),
            source,
        })?;
        let _ = stdout_reader.join();
        let _ = stderr_reader.join();

        let terminal_event = if status.success() {
            DownloaderEvent::Succeeded {
                destination: request.destination.clone(),
            }
        } else {
            match &safe_failure {
                Some(failure) => failure.clone(),
                None => classify_failure(&stderr_tail),
            }
        };
        let diagnostic_log = match &terminal_event {
            DownloaderEvent::Succeeded { .. } => None,
            DownloaderEvent::Failed { message, .. } => Some(if safe_failure.is_some() {
                message.clone()
            } else {
                refine_diagnostic(&stderr_tail)
            }),
            DownloaderEvent::Started { .. } | DownloaderEvent::Progress { .. } => None,
        };
        on_event(&terminal_event);
        events.push(terminal_event);

        Ok(DownloadRun {
            events,
            succeeded: status.success(),
            diagnostic_log,
        })
    }
}

fn push_bounded(buffer: &mut Vec<String>, line: String, capacity: usize) {
    if buffer.len() == capacity {
        buffer.remove(0);
    }
    buffer.push(line);
}

fn classify_failure(stderr_tail: &[String]) -> DownloaderEvent {
    let text = stderr_tail.join("\n").to_lowercase();
    let kind = if text.contains("permission denied")
        || text.contains("not writable")
        || text.contains("permissionerror")
    {
        DownloadFailureKind::Permission
    } else if text.contains("name or service not known")
        || text.contains("network is unreachable")
        || text.contains("connection reset")
        || text.contains("connection timed out")
        || text.contains("temporary failure in name resolution")
        || text.contains("unable to download")
    {
        DownloadFailureKind::TransientNetwork
    } else {
        DownloadFailureKind::Unknown
    };
    let message = stderr_tail
        .iter()
        .rev()
        .find(|line| !line.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "Downloader exited without a safe diagnostic.".into());
    DownloaderEvent::Failed { kind, message }
}

fn refine_diagnostic(stderr_tail: &[String]) -> String {
    if stderr_tail.is_empty() {
        return "Downloader exited without a safe diagnostic.".into();
    }
    let mut refined: String = stderr_tail
        .join("\n")
        .chars()
        .take(MAX_DIAGNOSTIC_CHARS)
        .collect();
    while refined.ends_with('\n') {
        refined.pop();
    }
    refined
}

fn stop_child<T>(
    mut child: std::process::Child,
    error: DownloaderError,
) -> Result<T, DownloaderError> {
    let _ = child.kill();
    let _ = child.wait();
    Err(error)
}

#[derive(Clone, Copy)]
enum StreamKind {
    StandardOutput,
    StandardError,
}

enum StreamLine {
    StandardOutput(String),
    StandardError(String),
    ReadError(io::Error),
}

fn spawn_reader<R>(
    reader: R,
    sender: SyncSender<StreamLine>,
    kind: StreamKind,
) -> thread::JoinHandle<()>
where
    R: io::Read + Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let stream_line = match line {
                Ok(line) => match kind {
                    StreamKind::StandardOutput => StreamLine::StandardOutput(line),
                    StreamKind::StandardError => StreamLine::StandardError(line),
                },
                Err(error) => StreamLine::ReadError(error),
            };

            if sender.send(stream_line).is_err() {
                break;
            }
        }
    })
}

fn parse_progress(progress: &str, line: &str) -> Result<DownloaderEvent, DownloaderError> {
    let mut values = progress.split(':');
    let downloaded_bytes = parse_required_u64(values.next(), line)?;
    let total_bytes =
        parse_optional_u64(values.next(), line)?.or(parse_optional_u64(values.next(), line)?);
    let speed_bytes_per_second = parse_optional_u64(values.next(), line)?;
    let eta_seconds = parse_optional_u64(values.next(), line)?;

    if values.next().is_some() {
        return Err(DownloaderError::InvalidProgress {
            line: line.to_owned(),
        });
    }

    let percent = total_bytes
        .filter(|total_bytes| *total_bytes > 0)
        .map(|total_bytes| downloaded_bytes as f32 * 100.0 / total_bytes as f32);

    Ok(DownloaderEvent::Progress {
        downloaded_bytes,
        total_bytes,
        percent,
        speed_bytes_per_second,
        eta_seconds,
    })
}

fn parse_required_u64(value: Option<&str>, line: &str) -> Result<u64, DownloaderError> {
    parse_optional_u64(value, line)?.ok_or_else(|| DownloaderError::InvalidProgress {
        line: line.to_owned(),
    })
}

fn parse_optional_u64(value: Option<&str>, line: &str) -> Result<Option<u64>, DownloaderError> {
    match value {
        Some("NA" | "None" | "") => Ok(None),
        Some(value) => value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0 && *value <= u64::MAX as f64)
            .map(|value| value as u64)
            .map(Some)
            .ok_or_else(|| DownloaderError::InvalidProgress {
                line: line.to_owned(),
            }),
        None => Err(DownloaderError::InvalidProgress {
            line: line.to_owned(),
        }),
    }
}

fn parse_failure_line(line: &str) -> Option<DownloaderEvent> {
    let failure = line.strip_prefix(FAILURE_PREFIX)?;
    let (kind, message) = failure.split_once(':').unwrap_or(("unknown", failure));
    let kind = match kind {
        "transient_network" => DownloadFailureKind::TransientNetwork,
        "permission" => DownloadFailureKind::Permission,
        "interrupted" => DownloadFailureKind::Interrupted,
        _ => DownloadFailureKind::Unknown,
    };

    Some(DownloaderEvent::Failed {
        kind,
        message: message.trim().to_owned(),
    })
}
