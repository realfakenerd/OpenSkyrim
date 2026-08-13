use crate::esm::{
    binary::parse_plugin_file,
    exporter::{create_tables, export_to_db},
    records::RawRecord,
};
use color_eyre::Result;
use rusqlite::Connection;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
pub mod binary;
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

        let mut master: HashMap<u32, RawRecord> = HashMap::new();

        for path in plugin_paths {
            let records = parse_plugin_file(path)?;
            for rec in records {
                if rec.is_deleted() {
                    master.remove(&rec.form_id);
                } else {
                    master.insert(rec.form_id, rec);
                }
            }
        }

        export_to_db(&conn, &master)?;

        Ok(())
    }
}
