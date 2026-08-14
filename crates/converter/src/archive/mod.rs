mod ba2;
mod bsa;

use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
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
        let bytes = unsafe { memmap2::Mmap::map(&file) }
            .wrap_err_with(|| format!("failed to map archive {}", archive_path.display()))?;
        let entries = match bytes.get(..4) {
            Some(b"BSA\0") => bsa::read_entries(&bytes)?,
            Some(b"BTDX") => ba2::read_entries(&bytes)?,
            _ => bail!("unsupported archive magic in {}", archive_path.display()),
        };

        let mut extracted = Vec::with_capacity(entries.len());
        for (name, data) in entries {
            let relative = safe_relative_path(&name)?;
            let destination = output_root.join(&relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            atomic_write(&destination, &data)
                .wrap_err_with(|| format!("failed to extract {}", destination.display()))?;
            extracted.push(ExtractedFile {
                path: relative,
                bytes_written: data.len() as u64,
            });
        }
        Ok(extracted)
    }
}

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
