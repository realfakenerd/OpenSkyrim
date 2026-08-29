use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub plugins_file: Option<PathBuf>,
    pub cpu_jobs: usize,
    pub io_jobs: usize,
    pub enable_ba2: bool,
    pub fail_fast: bool,
    pub invalidate_cache: bool,
    pub verify_cache: bool,
    pub texture_etc1s_quality: u8,
    pub texture_uastc_level: u8,
    pub script_abi_version: u32,
}

impl PipelineConfig {
    pub fn new(data_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            output_dir: output_dir.into(),
            plugins_file: None,
            cpu_jobs: std::thread::available_parallelism().map_or(1, usize::from),
            io_jobs: 2,
            enable_ba2: true,
            fail_fast: false,
            invalidate_cache: false,
            verify_cache: true,
            texture_etc1s_quality: 192,
            texture_uastc_level: 2,
            script_abi_version: 1,
        }
    }

    pub(crate) fn validate(&self) -> color_eyre::Result<()> {
        color_eyre::eyre::ensure!(
            self.data_dir.is_dir(),
            "Skyrim Data directory does not exist: {}",
            self.data_dir.display()
        );
        color_eyre::eyre::ensure!(self.cpu_jobs > 0, "cpu_jobs must be greater than zero");
        color_eyre::eyre::ensure!(self.io_jobs > 0, "io_jobs must be greater than zero");
        color_eyre::eyre::ensure!(
            (1..=255).contains(&self.texture_etc1s_quality),
            "texture_etc1s_quality must be between 1 and 255"
        );
        color_eyre::eyre::ensure!(
            self.texture_uastc_level <= 4,
            "texture_uastc_level must be between 0 and 4"
        );
        color_eyre::eyre::ensure!(
            self.data_dir != self.output_dir,
            "output directory must not be the Skyrim Data directory"
        );
        Ok(())
    }
}
