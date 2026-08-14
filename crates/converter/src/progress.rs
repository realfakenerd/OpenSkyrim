use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStage {
    Discovering,
    Extracting,
    Database,
    Textures,
    Meshes,
    Scripts,
    Validating,
    Publishing,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub stage: ProgressStage,
    pub completed: u64,
    pub total: u64,
    pub current_file: Option<PathBuf>,
    pub message: String,
}

impl ProgressEvent {
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f32 / self.total as f32).clamp(0.0, 1.0)
        }
    }
}
