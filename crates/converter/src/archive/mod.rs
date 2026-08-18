mod ba2;
mod bsa;

use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use memmap2::Mmap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveKind {
    Bsa,
    Ba2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFile {
    pub path: PathBuf,
    pub bytes_written: u64,
}

pub struct ArchiveExtractor;

impl ArchiveExtractor {
    pub fn extract(archive_path: &Path, output_root: &Path) -> Result<Vec<ExtractedFile>> {
        let file = File::open(archive_path)
            .wrap_err_with(|| format!("failed to open archive {}", archive_path.display()))?;
        // SAFETY: the file remains open and the mapping is read-only for the duration
        // of parsing. No process-local code mutates the archive while it is mapped.
        let bytes = unsafe { Mmap::map(&file) }
            .wrap_err_with(|| format!("failed to map archive {}", archive_path.display()))?;

        match bytes.get(..4) {
            Some(b"BSA\0") => {
                let entries = bsa::iter_raw_entries(&bytes)?;
                entries
                    .into_par_iter()
                    .map(|entry| {
                        let relative = safe_relative_path(&entry.name)?;
                        let destination = output_root.join(&relative);
                        if let Some(parent) = destination.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        let data = entry.decompress()?;
                        let bytes_written = data.len() as u64;

                        atomic_write(&destination, &data).wrap_err_with(|| {
                            format!("failed to extract {}", destination.display())
                        })?;

                        Ok(ExtractedFile {
                            path: relative,
                            bytes_written,
                        })
                    })
                    .collect()
            }
            Some(b"BTDX") => {
                let entries = ba2::read_entries(&bytes)?;
                entries
                    .into_par_iter()
                    .map(|(name, data)| {
                        let relative = safe_relative_path(&name)?;
                        let destination = output_root.join(&relative);
                        if let Some(parent) = destination.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        let bytes_written = data.len() as u64;

                        atomic_write(&destination, &data).wrap_err_with(|| {
                            format!("failed to extract {}", destination.display())
                        })?;

                        Ok(ExtractedFile {
                            path: relative,
                            bytes_written,
                        })
                    })
                    .collect()
            }
            _ => bail!("unsupported archive magic in {}", archive_path.display()),
        }
    }
}

/// Atomically writes data to a file, ensuring the destination is replaced atomically.
fn atomic_write(destination: &Path, data: &[u8]) -> Result<()> {
    let file_name = destination
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let temporary =
        destination.with_file_name(format!(".{file_name}.{}.partial", std::process::id()));
    let mut file = File::create(&temporary)?;
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(&temporary, destination)?;
    Ok(())
}

/// Converts an archive-relative path to a safe, relative path, ensuring it does not contain
/// absolute paths or drive letters.
pub(crate) fn safe_relative_path(name: &str) -> Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute() || normalized.contains(':') {
        bail!("archive contains absolute path: {name}");
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) if !part.is_empty() => safe.push(part),
            Component::CurDir => {}
            _ => bail!("archive contains unsafe path: {name}"),
        }
    }
    if safe.as_os_str().is_empty() {
        bail!("archive contains an empty path");
    }
    Ok(safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_archive_paths() {
        assert_eq!(
            safe_relative_path(r"meshes\actors\wolf.nif").unwrap(),
            PathBuf::from("meshes/actors/wolf.nif")
        );
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(safe_relative_path("../outside.txt").is_err());
        assert!(safe_relative_path("C:/outside.txt").is_err());
    }
}
