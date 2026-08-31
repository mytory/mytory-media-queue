use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

const BUNDLED_TOOL_VERSION: &str = "2026.8.19+ejs.0.8.0";
const BUNDLED_DOWNLOADER_SHA256: &str =
    "1d57897e94c6665a0a6f9bc54b34e584284e32c034ffab3a7df25d8f7b24eedf";
const BUNDLED_EXTRACTOR_SHA256: &str =
    "79300e5fca7f937a1eeede11f0456862c1b41107ce1d726871e0207424f4bdb4";
// This immutable release asset is the Managed Update trust root. It deliberately
// does not use GitHub's `latest` endpoint.
const MANAGED_UPDATE_MANIFEST_URL: &str =
    "https://github.com/mytory/mytory-media-queue/releases/download/managed-tools-v1/manifest.json";
use tauri_plugin_opener::OpenerExt;

pub mod downloader;
pub mod queue;
pub mod service;
pub mod storage;
pub mod tools;

pub use downloader::{
    DownloadFailureKind, DownloadRun, DownloaderCommand, DownloaderError, DownloaderEvent,
    DownloaderRequest, DownloaderRunner,
};
pub use queue::{DownloadQueue, DownloadStatus, OutputPreset, QueueJob};
pub use service::DownloadService;
pub use storage::migrate;
pub use tools::{
    InstalledToolSet, ManagedToolSet, ManagedUpdateOutcome, ToolManager, ToolManagerError,
};

struct AppState {
    queue: Arc<DownloadQueue>,
    service: DownloadService,
    tool_manager: ToolManager,
    update_status: Arc<Mutex<ManagedUpdateStatus>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct ManagedUpdateStatus {
    current_version: String,
    state: String,
    message: String,
}

fn managed_update_status(
    queue: &DownloadQueue,
    tool_manager: &ToolManager,
) -> Result<ManagedUpdateStatus, ToolManagerError> {
    let current = tool_manager
        .current()?
        .expect("the bundled tool set was initialized");
    if queue
        .has_running_work()
        .map_err(|error| ToolManagerError::Io(std::io::Error::other(error.to_string())))?
    {
        return Ok(ManagedUpdateStatus {
            current_version: current.version,
            state: "deferred".into(),
            message: "진행 중인 다운로드가 끝난 뒤 업데이트를 적용합니다.".into(),
        });
    }
    queue
        .record_managed_update_check()
        .map_err(|error| ToolManagerError::Io(std::io::Error::other(error.to_string())))?;
    match tool_manager.update_from_manifest_url(MANAGED_UPDATE_MANIFEST_URL)? {
        ManagedUpdateOutcome::Applied { version } => Ok(ManagedUpdateStatus {
            current_version: version,
            state: "updated".into(),
            message: "Downloader와 Bundled Extractor를 함께 업데이트했습니다.".into(),
        }),
        ManagedUpdateOutcome::AlreadyCurrent { version } => Ok(ManagedUpdateStatus {
            current_version: version,
            state: "current".into(),
            message: "이미 최신 Downloader와 Bundled Extractor 세트입니다.".into(),
        }),
    }
}

#[derive(Deserialize)]
struct EnqueueDownloadsRequest {
    urls: Vec<String>,
    destination: String,
    output_preset: Option<OutputPreset>,
    write_subs: Option<bool>,
    cookies: Option<String>,
}

#[tauri::command]
fn enqueue_downloads(
    request: EnqueueDownloadsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<QueueJob>, String> {
    let jobs = state
        .queue
        .enqueue_with_options(
            &request.urls,
            Path::new(&request.destination),
            request.output_preset.unwrap_or(OutputPreset::Mp4Compatible),
            request.write_subs.unwrap_or(false),
        )
        .map_err(|error| error.to_string())?;
    if let Some(cookies) = request.cookies {
        let cookies = PathBuf::from(cookies);
        for job in &jobs {
            state
                .service
                .remember_cookie_source(&job.id, cookies.clone());
        }
    }
    state
        .service
        .start_available()
        .map_err(|error| error.to_string())?;
    Ok(jobs)
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<QueueJob>, String> {
    state.queue.jobs().map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.remove(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.cancel(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn retry_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.service.retry(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .service
        .clear_history()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_download_folder(
    id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let destination = state
        .service
        .destination_of(&id)
        .ok_or_else(|| format!("작업을 찾을 수 없습니다: {id}"))?;
    app.opener()
        .open_path(destination.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn default_download_destination(app: AppHandle) -> Result<String, String> {
    app.path()
        .download_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_managed_update_status(state: State<'_, AppState>) -> Result<ManagedUpdateStatus, String> {
    Ok(state
        .update_status
        .lock()
        .expect("managed update status lock poisoned")
        .clone())
}

#[tauri::command]
fn check_managed_update(state: State<'_, AppState>) -> Result<ManagedUpdateStatus, String> {
    let status = managed_update_status(&state.queue, &state.tool_manager)
        .map_err(|error| error.to_string())?;
    *state
        .update_status
        .lock()
        .expect("managed update status lock poisoned") = status.clone();
    Ok(status)
}

#[tauri::command]
fn get_download_concurrency(state: State<'_, AppState>) -> Result<u8, String> {
    state.queue.concurrency().map_err(|error| error.to_string())
}

#[tauri::command]
fn set_download_concurrency(concurrency: u8, state: State<'_, AppState>) -> Result<bool, String> {
    let changed = state
        .queue
        .set_concurrency(concurrency)
        .map_err(|error| error.to_string())?;
    if changed {
        state
            .service
            .start_available()
            .map_err(|error| error.to_string())?;
    }
    Ok(changed)
}

fn sidecar_candidates(name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
            candidates.push(dir.join(format!("{name}.exe")));
        }
    }
    candidates.push(PathBuf::from(name));
    candidates
}

fn resolve_sidecar(app: &tauri::App, name: &str) -> Option<PathBuf> {
    for candidate in sidecar_candidates(name) {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // Bundled Linux resources fall back to the resource directory.
    for candidate in [format!("binaries/{name}"), format!("binaries/{name}.exe")] {
        if let Ok(path) = app
            .path()
            .resolve(candidate, tauri::path::BaseDirectory::Resource)
        {
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_bundled_resource(app: &tauri::App, relative_path: &str) -> Option<PathBuf> {
    app.path()
        .resolve(relative_path, tauri::path::BaseDirectory::Resource)
        .ok()
        .filter(|path| path.is_file())
}

fn resolve_bundled_python(app: &tauri::App) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("MYTORY_PYTHON_PATH").map(PathBuf::from) {
        return path.is_file().then_some(path);
    }
    #[cfg(target_os = "macos")]
    let relative_path = if std::env::consts::ARCH == "aarch64" {
        "binaries/python/aarch64-apple-darwin/bin/python3"
    } else {
        "binaries/python/x86_64-apple-darwin/bin/python3"
    };
    #[cfg(target_os = "windows")]
    let relative_path = "binaries/python/x86_64-pc-windows-msvc/python.exe";
    #[cfg(target_os = "linux")]
    let relative_path = "binaries/python/x86_64-unknown-linux-gnu/bin/python3";
    resolve_bundled_resource(app, relative_path)
}

fn required_bundled_resource(app: &tauri::App, relative_path: &str) -> std::io::Result<PathBuf> {
    resolve_bundled_resource(app, relative_path).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("missing bundled resource: {relative_path}"),
        )
    })
}

fn resolve_ffmpeg_dir(app: &tauri::App) -> Option<PathBuf> {
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        if dir.join("ffmpeg").is_file() || dir.join("ffmpeg.exe").is_file() {
            return Some(dir);
        }
    }
    let dir = app
        .path()
        .resolve("binaries", tauri::path::BaseDirectory::Resource)
        .ok()?;
    let has_ffmpeg = dir.join("ffmpeg").is_file() || dir.join("ffmpeg.exe").is_file();
    if dir.is_dir() && has_ffmpeg {
        Some(dir)
    } else {
        None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let queue = Arc::new(DownloadQueue::open(data_dir.join("downloads.sqlite"))?);
            let tool_manager = ToolManager::open(data_dir.join("tools"))?;
            let bundled_downloader = required_bundled_resource(app, "binaries/wheels/yt-dlp.whl")?;
            let bundled_extractor =
                required_bundled_resource(app, "binaries/wheels/yt-dlp-ejs.whl")?;
            tool_manager.initialize_if_missing(ManagedToolSet::new(
                BUNDLED_TOOL_VERSION,
                bundled_downloader,
                BUNDLED_DOWNLOADER_SHA256,
                bundled_extractor,
                BUNDLED_EXTRACTOR_SHA256,
            ))?;
            let current = tool_manager
                .current()?
                .expect("the bundled tool set was just initialized");
            let current_version = current.version.clone();
            let python = resolve_bundled_python(app).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "missing Bundled Python")
            })?;
            let deno = resolve_sidecar(app, "deno").ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "missing Bundled Runtime")
            })?;
            let command = DownloaderCommand::bundled_python(
                python,
                current.downloader,
                current.extractor,
                deno,
            );
            let service = DownloadService::with_downloader_command(queue.clone(), command)
                .with_ffmpeg_location(resolve_ffmpeg_dir(app));
            service.start_available()?;
            let initial_status = ManagedUpdateStatus {
                current_version,
                state: "idle".into(),
                message: "업데이트를 아직 확인하지 않았습니다.".into(),
            };
            if queue.managed_update_is_due()? {
                let queue_for_check = queue.clone();
                let manager_for_check = tool_manager.clone();
                let update_status_for_check = Arc::new(Mutex::new(initial_status.clone()));
                let status_for_thread = update_status_for_check.clone();
                std::thread::spawn(move || {
                    if let Ok(status) = managed_update_status(&queue_for_check, &manager_for_check)
                    {
                        *status_for_thread
                            .lock()
                            .expect("managed update status lock poisoned") = status;
                    }
                });
                app.manage(AppState {
                    queue,
                    service,
                    tool_manager,
                    update_status: update_status_for_check,
                });
            } else {
                app.manage(AppState {
                    queue,
                    service,
                    tool_manager,
                    update_status: Arc::new(Mutex::new(initial_status)),
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enqueue_downloads,
            list_downloads,
            remove_download,
            cancel_download,
            retry_download,
            clear_history,
            open_download_folder,
            default_download_destination,
            get_managed_update_status,
            check_managed_update,
            get_download_concurrency,
            set_download_concurrency
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mytory Media Queue");
}
