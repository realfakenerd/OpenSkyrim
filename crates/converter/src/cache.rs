use color_eyre::{Result, eyre::WrapErr};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufReader, Read, Write},
    path::Path,
};

pub const CONVERTER_SCHEMA_VERSION: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    pub source_hash: String,
    pub output: String,
    pub output_size: u64,
    pub output_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestedFile {
    pub path: String,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IngestionCacheEntry {
    pub source_hash: String,
    pub files: Vec<IngestedFile>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversionManifest {
    pub schema_version: u32,
    pub complete: bool,
    #[serde(default)]
    pub configuration_hash: String,
    #[serde(default)]
    pub inputs_by_kind: BTreeMap<String, u64>,
    #[serde(default)]
    pub failures: BTreeMap<String, String>,
    #[serde(default)]
    pub archives: BTreeMap<String, IngestionCacheEntry>,
    pub entries: BTreeMap<String, CacheEntry>,
}

impl ConversionManifest {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self {
                schema_version: CONVERTER_SCHEMA_VERSION,
                ..Self::default()
            });
        }
        let bytes =
            fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;
        let manifest: Self =
            serde_json::from_slice(&bytes).wrap_err("invalid conversion manifest")?;
        if manifest.schema_version != CONVERTER_SCHEMA_VERSION {
            return Ok(Self {
                schema_version: CONVERTER_SCHEMA_VERSION,
                ..Self::default()
            });
        }
        Ok(manifest)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        let temporary = path.with_extension(format!("json.{}.partial", std::process::id()));
        let mut file = fs::File::create(&temporary)
            .wrap_err_with(|| format!("failed to create {}", temporary.display()))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
            .wrap_err_with(|| format!("failed to publish {}", path.display()))
    }
}

pub fn configuration_hash(config: &crate::config::PipelineConfig) -> Result<String> {
    let relevant = serde_json::json!({
        "schema": CONVERTER_SCHEMA_VERSION,
        "texture_etc1s_quality": config.texture_etc1s_quality,
        "texture_uastc_level": config.texture_uastc_level,
        "script_abi_version": config.script_abi_version,
    });
    Ok(hash_bytes(&serde_json::to_vec(&relevant)?))
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hash_file(path: &Path) -> Result<String> {
    let file = fs::File::open(path)
        .wrap_err_with(|| format!("failed to open {} for hashing", path.display()))?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .wrap_err_with(|| format!("failed to hash {}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_stable() {
        assert_eq!(
            hash_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
