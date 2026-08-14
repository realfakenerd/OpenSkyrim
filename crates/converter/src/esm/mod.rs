use crate::esm::{
    binary::{parse_plugin_file, parse_plugin_metadata},
    exporter::{create_tables, export_to_db},
    records::RawRecord,
};
use color_eyre::Result;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
pub mod binary;
pub mod cell_cache;
pub mod exporter;
pub mod extractors;
pub mod mmap_reader;
pub mod records;
pub mod types;

pub struct EsmParser;

impl EsmParser {
    /// Parses .esm files and exports world data to skyrim_world.db
    pub fn convert_plugins(plugin_paths: &[PathBuf], db_path: &Path) -> Result<()> {
        let conn = Connection::open(db_path)?;
        create_tables(&conn)?;
        for (priority, path) in plugin_paths.iter().enumerate() {
            let checksum = Sha256::digest(std::fs::read(path)?);
            conn.execute(
                "INSERT OR REPLACE INTO plugins (id, name, priority, checksum) VALUES (?1, ?2, ?3, ?4)",
                params![priority as i64, path.file_name().unwrap_or_default().to_string_lossy(), priority as i64, checksum.as_slice()],
            )?;
        }
        let master = Self::merge_plugins(plugin_paths)?;
        export_to_db(&conn, &master)?;

        Ok(())
    }

    pub fn merge_plugins(plugin_paths: &[PathBuf]) -> Result<HashMap<u32, RawRecord>> {
        let names: Vec<String> = plugin_paths
            .iter()
            .map(|path| {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_ascii_lowercase()
            })
            .collect();
        let mut normal_indices = HashMap::new();
        let mut light_indices = HashMap::new();
        let mut next_normal = 0u32;
        let mut next_light = 0u32;
        for (path, name) in plugin_paths.iter().zip(&names) {
            let metadata = parse_plugin_metadata(path)?;
            if path
                .extension()
                .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("esl"))
                || metadata.flags & 0x0000_0200 != 0
            {
                light_indices.insert(name.clone(), next_light);
                next_light += 1;
            } else {
                normal_indices.insert(name.clone(), next_normal);
                next_normal += 1;
            }
        }
        let mut merged = HashMap::new();
        for (priority, path) in plugin_paths.iter().enumerate() {
            let metadata = parse_plugin_metadata(path)?;
            for mut record in parse_plugin_file(path)? {
                record.load_order = priority as u32;
                remap_record_form_ids(
                    &mut record,
                    &names[priority],
                    &metadata.masters,
                    &normal_indices,
                    &light_indices,
                )?;
                if record.is_deleted() {
                    merged.remove(&record.form_id);
                } else {
                    merged.insert(record.form_id, record);
                }
            }
        }
        Ok(merged)
    }
}

pub fn read_plugins_txt(path: &Path, data_dir: &Path) -> Result<Vec<PathBuf>> {
    let contents = std::fs::read_to_string(path)?;
    let mut plugins = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let enabled = line.starts_with('*');
        let name = line.strip_prefix('*').unwrap_or(line).trim();
        if !matches!(
            Path::new(name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .as_deref(),
            Some("esm" | "esp" | "esl")
        ) {
            continue;
        }
        if !enabled && !name.to_ascii_lowercase().ends_with(".esm") {
            continue;
        }
        let exact = data_dir.join(name);
        if exact.is_file() {
            plugins.push(exact);
            continue;
        }
        if let Some(found) = std::fs::read_dir(data_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|candidate| {
                candidate
                    .file_name()
                    .is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name))
            })
        {
            plugins.push(found);
        } else {
            color_eyre::eyre::bail!("active plugin not found: {name}");
        }
    }
    Ok(plugins)
}

fn remap_record_form_ids(
    record: &mut RawRecord,
    plugin_name: &str,
    masters: &[String],
    normal_indices: &HashMap<String, u32>,
    light_indices: &HashMap<String, u32>,
) -> Result<()> {
    let remap = |form_id: u32| -> Result<u32> {
        if form_id == 0 {
            return Ok(0);
        }
        let local_index = (form_id >> 24) as usize;
        let owner = if local_index < masters.len() {
            masters[local_index].to_ascii_lowercase()
        } else {
            plugin_name.to_owned()
        };
        if let Some(index) = light_indices.get(&owner) {
            return Ok(0xFE00_0000 | (index << 12) | (form_id & 0xFFF));
        }
        let index = normal_indices.get(&owner).ok_or_else(|| {
            color_eyre::eyre::eyre!("master {owner} is not present in load order")
        })?;
        Ok((index << 24) | (form_id & 0x00FF_FFFF))
    };
    record.form_id = remap(record.form_id)?;
    record.cell_form_id = record.cell_form_id.map(&remap).transpose()?;
    record.worldspace_form_id = record.worldspace_form_id.map(&remap).transpose()?;
    const FORM_ID_TAGS: [[u8; 4]; 12] = [
        *b"NAME", *b"XOWN", *b"XGLB", *b"XEZN", *b"XLCN", *b"XLRL", *b"WNAM", *b"CNAM", *b"TNAM",
        *b"SNAM", *b"INAM", *b"RNAM",
    ];
    for (tag, data) in &mut record.subrecords {
        if tag.as_slice() == b"VMAD" {
            records::record_type::vmad::remap_primary_form_ids(data, &remap)?;
            continue;
        }
        if data.len() >= 4
            && FORM_ID_TAGS
                .iter()
                .any(|candidate| tag.as_slice() == candidate)
        {
            let value = u32::from_le_bytes(data[..4].try_into().unwrap());
            data[..4].copy_from_slice(&remap(value)?.to_le_bytes());
        }
    }
    Ok(())
}
