use std::collections::HashMap;

use chrono::{DateTime, Utc};

pub type FileID = String;
pub type ModelID = String;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct File {
    #[serde(default)]
    pub id: FileID,
    pub name: String,
    pub size: String,
    pub quantization: String,
    #[serde(default)]
    pub downloaded: bool,
    #[serde(default)]
    pub downloaded_path: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub featured: bool,
    pub context_size: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Author {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub enum CompatibilityGuess {
    #[default]
    PossiblySupported,
    NotSupported,
}

impl CompatibilityGuess {
    pub fn as_str(&self) -> &str {
        match self {
            CompatibilityGuess::PossiblySupported => "Possibly Supported",
            CompatibilityGuess::NotSupported => "Not Supported",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DownloadedFile {
    pub file: File,
    pub model: Model,
    pub downloaded_at: DateTime<Utc>,
    pub compatibility_guess: CompatibilityGuess,
    pub information: String,
}

#[derive(Clone, Debug, Default)]
pub enum PendingDownloadsStatus {
    #[default]
    Initializing,
    Downloading,
    Paused,
    Error,
}

#[derive(Clone, Debug, Default)]
pub struct PendingDownload {
    pub file: File,
    pub model: Model,
    pub progress: f64,
    pub status: PendingDownloadsStatus,
}

// We're using the HuggingFace identifier as the model ID for now
// We should consider using a different identifier in the future if more
// models sources are added.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub id: ModelID,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub files: Vec<File>,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
    pub metrics: HashMap<String, f32>,
}
