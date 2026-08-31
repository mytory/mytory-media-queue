use std::{
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const CURRENT_DIRECTORY: &str = "current";
const PREVIOUS_DIRECTORY: &str = "previous";
const DOWNLOADER_FILE: &str = "yt-dlp.whl";
const EXTRACTOR_FILE: &str = "yt-dlp-ejs.whl";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Clone, Debug)]
pub struct ManagedToolSet {
    version: String,
    downloader: PathBuf,
    downloader_sha256: String,
    extractor: PathBuf,
    extractor_sha256: String,
}

impl ManagedToolSet {
    pub fn new(
        version: impl Into<String>,
        downloader: impl Into<PathBuf>,
        downloader_sha256: impl Into<String>,
        extractor: impl Into<PathBuf>,
        extractor_sha256: impl Into<String>,
    ) -> Self {
        Self {
            version: version.into(),
            downloader: downloader.into(),
            downloader_sha256: downloader_sha256.into(),
            extractor: extractor.into(),
            extractor_sha256: extractor_sha256.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledToolSet {
    pub version: String,
    pub downloader: PathBuf,
    pub extractor: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedUpdateOutcome {
    Applied { version: String },
    AlreadyCurrent { version: String },
}

#[derive(Debug)]
pub enum ToolManagerError {
    Io(io::Error),
    InvalidManifest(serde_json::Error),
    ChecksumMismatch { file: PathBuf },
    IncompleteSet { directory: PathBuf },
    InvalidUpdateManifest { reason: String },
    UntrustedArtifactUrl { url: String },
}

impl fmt::Display for ToolManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "tool manager I/O error: {error}"),
            Self::InvalidManifest(error) => write!(f, "invalid managed tool manifest: {error}"),
            Self::ChecksumMismatch { file } => {
                write!(f, "checksum verification failed for {}", file.display())
            }
            Self::IncompleteSet { directory } => {
                write!(f, "managed tool set is incomplete: {}", directory.display())
            }
            Self::InvalidUpdateManifest { reason } => {
                write!(f, "invalid managed update manifest: {reason}")
            }
            Self::UntrustedArtifactUrl { url } => {
                write!(
                    f,
                    "managed update artifact is not from an approved origin: {url}"
                )
            }
        }
    }
}

impl Error for ToolManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidManifest(error) => Some(error),
            Self::ChecksumMismatch { .. }
            | Self::IncompleteSet { .. }
            | Self::InvalidUpdateManifest { .. }
            | Self::UntrustedArtifactUrl { .. } => None,
        }
    }
}

impl From<io::Error> for ToolManagerError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
pub struct ToolManager {
    root: PathBuf,
}

impl ToolManager {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ToolManagerError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn current(&self) -> Result<Option<InstalledToolSet>, ToolManagerError> {
        let directory = self.root.join(CURRENT_DIRECTORY);
        if !directory.exists() {
            return Ok(None);
        }
        read_installed_set(&directory).map(Some)
    }

    pub fn initialize_if_missing(
        &self,
        bundled_set: ManagedToolSet,
    ) -> Result<(), ToolManagerError> {
        if self.current()?.is_none() {
            self.install(bundled_set)?;
        }
        Ok(())
    }

    pub fn update_from_manifest_url(
        &self,
        manifest_url: impl AsRef<str>,
    ) -> Result<ManagedUpdateOutcome, ToolManagerError> {
        let manifest_url = manifest_url.as_ref();
        let manifest = download_json(manifest_url)?;
        validate_update_manifest(&manifest)?;
        if self
            .current()?
            .is_some_and(|current| current.version == manifest.version)
        {
            return Ok(ManagedUpdateOutcome::AlreadyCurrent {
                version: manifest.version,
            });
        }

        let download_directory = self.root.join(format!(".download-{}", Uuid::new_v4()));
        fs::create_dir(&download_directory)?;
        let downloader = download_directory.join(DOWNLOADER_FILE);
        let extractor = download_directory.join(EXTRACTOR_FILE);
        let result = download_file(&manifest.downloader.url, &downloader)
            .and_then(|()| download_file(&manifest.extractor.url, &extractor))
            .and_then(|()| {
                self.install(ManagedToolSet::new(
                    manifest.version.clone(),
                    &downloader,
                    &manifest.downloader.sha256,
                    &extractor,
                    &manifest.extractor.sha256,
                ))
            });
        let _ = fs::remove_dir_all(&download_directory);
        result.map(|()| ManagedUpdateOutcome::Applied {
            version: manifest.version,
        })
    }

    pub fn install(&self, set: ManagedToolSet) -> Result<(), ToolManagerError> {
        verify_checksum(&set.downloader, &set.downloader_sha256)?;
        verify_checksum(&set.extractor, &set.extractor_sha256)?;

        let staging = self.root.join(format!(".staging-{}", Uuid::new_v4()));
        fs::create_dir(&staging)?;
        let result = self
            .stage(&staging, &set)
            .and_then(|()| self.promote(&staging));
        if staging.exists() {
            let _ = fs::remove_dir_all(&staging);
        }
        result
    }

    fn stage(&self, staging: &Path, set: &ManagedToolSet) -> Result<(), ToolManagerError> {
        fs::copy(&set.downloader, staging.join(DOWNLOADER_FILE))?;
        fs::copy(&set.extractor, staging.join(EXTRACTOR_FILE))?;
        let manifest = StoredToolSet {
            version: set.version.clone(),
        };
        fs::write(
            staging.join(MANIFEST_FILE),
            serde_json::to_vec(&manifest).map_err(ToolManagerError::InvalidManifest)?,
        )?;
        Ok(())
    }

    fn promote(&self, staging: &Path) -> Result<(), ToolManagerError> {
        let current = self.root.join(CURRENT_DIRECTORY);
        let previous = self.root.join(PREVIOUS_DIRECTORY);
        if previous.exists() {
            fs::remove_dir_all(&previous)?;
        }
        if current.exists() {
            fs::rename(&current, &previous)?;
        }
        if let Err(error) = fs::rename(staging, &current) {
            if previous.exists() {
                let _ = fs::rename(&previous, &current);
            }
            return Err(error.into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct UpdateArtifact {
    url: String,
    sha256: String,
}

#[derive(Deserialize)]
struct UpdateManifest {
    version: String,
    downloader: UpdateArtifact,
    extractor: UpdateArtifact,
}

#[derive(Deserialize, Serialize)]
struct StoredToolSet {
    version: String,
}

fn update_http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build()
}

fn download_json(url: &str) -> Result<UpdateManifest, ToolManagerError> {
    let response = update_http_agent().get(url).call().map_err(http_error)?;
    serde_json::from_reader(response.into_reader()).map_err(ToolManagerError::InvalidManifest)
}

fn download_file(url: &str, destination: &Path) -> Result<(), ToolManagerError> {
    let response = update_http_agent().get(url).call().map_err(http_error)?;
    let mut source = response.into_reader();
    let mut destination = File::create(destination)?;
    io::copy(&mut source, &mut destination)?;
    Ok(())
}

fn http_error(error: ureq::Error) -> ToolManagerError {
    ToolManagerError::Io(io::Error::other(error.to_string()))
}

fn validate_update_manifest(manifest: &UpdateManifest) -> Result<(), ToolManagerError> {
    if manifest.version.trim().is_empty()
        || !is_sha256(&manifest.downloader.sha256)
        || !is_sha256(&manifest.extractor.sha256)
    {
        return Err(ToolManagerError::InvalidUpdateManifest {
            reason: "version and artifact SHA-256 values are required".into(),
        });
    }
    for artifact in [&manifest.downloader, &manifest.extractor] {
        if !is_approved_artifact_url(&artifact.url) {
            return Err(ToolManagerError::UntrustedArtifactUrl {
                url: artifact.url.clone(),
            });
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_approved_artifact_url(url: &str) -> bool {
    url.starts_with("https://files.pythonhosted.org/")
        || url.starts_with("https://github.com/yt-dlp/")
        || url.starts_with("http://127.0.0.1:")
}

fn read_installed_set(directory: &Path) -> Result<InstalledToolSet, ToolManagerError> {
    let downloader = directory.join(DOWNLOADER_FILE);
    let extractor = directory.join(EXTRACTOR_FILE);
    if !downloader.is_file() || !extractor.is_file() {
        return Err(ToolManagerError::IncompleteSet {
            directory: directory.to_path_buf(),
        });
    }
    let manifest = fs::read(directory.join(MANIFEST_FILE))?;
    let manifest = serde_json::from_slice::<StoredToolSet>(&manifest)
        .map_err(ToolManagerError::InvalidManifest)?;
    Ok(InstalledToolSet {
        version: manifest.version,
        downloader,
        extractor,
    })
}

fn verify_checksum(file: &Path, expected: &str) -> Result<(), ToolManagerError> {
    let mut reader = File::open(file)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(ToolManagerError::ChecksumMismatch {
            file: file.to_path_buf(),
        })
    }
}
