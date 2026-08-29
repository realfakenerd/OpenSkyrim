mod ba2;
mod bsa;

use crate::cache::{IngestedFile, IngestionCacheEntry, hash_bytes, hash_file};
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
    pub sha256: String,
}

#[derive(Debug)]
pub struct ExtractionOutcome {
    pub files: Vec<ExtractedFile>,
    pub cache_entry: IngestionCacheEntry,
    pub cache_hit: bool,
}

pub struct ArchiveExtractor;

impl ArchiveExtractor {
    pub fn extract_cached(
        archive_path: &Path,
        output_root: &Path,
        previous_cache_root: &Path,
        cache_root: &Path,
        previous: Option<&IngestionCacheEntry>,
        verify_integrity: bool,
    ) -> Result<ExtractionOutcome> {
        let source_hash = hash_file(archive_path)?;
        if let Some(entry) = previous.filter(|entry| entry.source_hash == source_hash)
            && let Some(files) = restore_cached_files(
                entry,
                output_root,
                previous_cache_root,
                cache_root,
                verify_integrity,
            )?
        {
            return Ok(ExtractionOutcome {
                files,
                cache_entry: entry.clone(),
                cache_hit: true,
            });
        }

        let files = Self::extract(archive_path, output_root)?;
        let cache_entry = IngestionCacheEntry {
            source_hash,
            files: files
                .iter()
                .map(|file| IngestedFile {
                    path: file.path.to_string_lossy().replace('\\', "/"),
                    size: file.bytes_written,
                    hash: file.sha256.clone(),
                })
                .collect(),
        };
        persist_cache_blobs(&files, output_root, cache_root)?;
        Ok(ExtractionOutcome {
            files,
            cache_entry,
            cache_hit: false,
        })
    }

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
                        let sha256 = hash_bytes(&data);

                        atomic_write(&destination, &data).wrap_err_with(|| {
                            format!("failed to extract {}", destination.display())
                        })?;

                        Ok(ExtractedFile {
                            path: relative,
                            bytes_written,
                            sha256,
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
                        let sha256 = hash_bytes(&data);

                        atomic_write(&destination, &data).wrap_err_with(|| {
                            format!("failed to extract {}", destination.display())
                        })?;

                        Ok(ExtractedFile {
                            path: relative,
                            bytes_written,
                            sha256,
                        })
                    })
                    .collect()
            }
            _ => bail!("unsupported archive magic in {}", archive_path.display()),
        }
    }
}

fn restore_cached_files(
    entry: &IngestionCacheEntry,
    output_root: &Path,
    previous_cache_root: &Path,
    cache_root: &Path,
    verify_integrity: bool,
) -> Result<Option<Vec<ExtractedFile>>> {
    for file in &entry.files {
        let blob = blob_path(previous_cache_root, &file.hash)?;
        if !blob.is_file()
            || fs::metadata(&blob).map_or(true, |metadata| metadata.len() != file.size)
            || (verify_integrity && hash_file(&blob).map_or(true, |hash| hash != file.hash))
        {
            return Ok(None);
        }
    }

    let mut restored = Vec::with_capacity(entry.files.len());
    for file in &entry.files {
        let relative = safe_relative_path(&file.path)?;
        let old_blob = blob_path(previous_cache_root, &file.hash)?;
        let new_blob = blob_path(cache_root, &file.hash)?;
        copy_if_missing(&old_blob, &new_blob)?;
        let destination = output_root.join(&relative);
        copy_file(&new_blob, &destination)?;
        restored.push(ExtractedFile {
            path: relative,
            bytes_written: file.size,
            sha256: file.hash.clone(),
        });
    }
    Ok(Some(restored))
}

fn persist_cache_blobs(
    files: &[ExtractedFile],
    output_root: &Path,
    cache_root: &Path,
) -> Result<()> {
    for file in files {
        copy_if_missing(
            &output_root.join(&file.path),
            &blob_path(cache_root, &file.sha256)?,
        )?;
    }
    Ok(())
}

fn blob_path(cache_root: &Path, hash: &str) -> Result<PathBuf> {
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid SHA-256 cache key");
    }
    Ok(cache_root.join("sha256").join(&hash[..2]).join(hash))
}

fn copy_if_missing(source: &Path, destination: &Path) -> Result<()> {
    if destination.is_file() {
        return Ok(());
    }
    copy_file(source, destination)
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination).wrap_err_with(|| {
        format!(
            "failed to restore cached asset {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
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
        assert!(blob_path(Path::new("cache"), "../../outside").is_err());
    }

    #[test]
    fn reuses_verified_archive_blobs_and_recovers_from_corruption() {
        let directory = tempfile::tempdir().unwrap();
        let archive = directory.path().join("assets.ba2");
        fs::write(&archive, general_ba2_fixture()).unwrap();

        let first_output = directory.path().join("first/vfs");
        let first_cache = directory.path().join("first/.ingestion-cache");
        let first = ArchiveExtractor::extract_cached(
            &archive,
            &first_output,
            Path::new("unused"),
            &first_cache,
            None,
            true,
        )
        .unwrap();
        assert!(!first.cache_hit);
        assert_eq!(
            fs::read(first_output.join("textures/test.dds")).unwrap(),
            b"DDS "
        );

        let second_output = directory.path().join("second/vfs");
        let second_cache = directory.path().join("second/.ingestion-cache");
        let second = ArchiveExtractor::extract_cached(
            &archive,
            &second_output,
            &first_cache,
            &second_cache,
            Some(&first.cache_entry),
            true,
        )
        .unwrap();
        assert!(second.cache_hit);
        assert_eq!(
            fs::read(second_output.join("textures/test.dds")).unwrap(),
            b"DDS "
        );

        let blob = blob_path(&second_cache, &second.files[0].sha256).unwrap();
        fs::write(&blob, b"BAD!").unwrap();
        let third = ArchiveExtractor::extract_cached(
            &archive,
            &directory.path().join("third/vfs"),
            &second_cache,
            &directory.path().join("third/.ingestion-cache"),
            Some(&second.cache_entry),
            true,
        )
        .unwrap();
        assert!(!third.cache_hit);
    }

    fn general_ba2_fixture() -> Vec<u8> {
        let name = b"textures/test.dds";
        let payload = b"DDS ";
        let names_offset = 24 + 36;
        let payload_offset = names_offset + 2 + name.len();
        let mut bytes = vec![0u8; payload_offset];
        bytes[..4].copy_from_slice(b"BTDX");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..12].copy_from_slice(b"GNRL");
        bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
        bytes[16..24].copy_from_slice(&(names_offset as u64).to_le_bytes());
        bytes[40..48].copy_from_slice(&(payload_offset as u64).to_le_bytes());
        bytes[52..56].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[names_offset..names_offset + 2].copy_from_slice(&(name.len() as u16).to_le_bytes());
        bytes[names_offset + 2..payload_offset].copy_from_slice(name);
        bytes.extend_from_slice(payload);
        bytes
    }
}
