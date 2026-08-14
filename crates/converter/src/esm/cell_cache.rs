use crate::esm::{extractors::SubrecordView, records::RawRecord};
use color_eyre::{Result, eyre::WrapErr};
use memmap2::Mmap;
use rkyv::{Archive, Deserialize, Serialize, rancor::Error};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

pub const CELL_CACHE_VERSION: u32 = 1;

#[derive(Debug, Archive, Serialize, Deserialize)]
#[rkyv(bytecheck())]
pub struct CachedLand {
    pub cell_id: u32,
    pub heightmap: Vec<u8>,
    pub normals: Vec<u8>,
    pub vertex_colors: Vec<u8>,
    pub texture_layers: Vec<u8>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
#[rkyv(bytecheck())]
pub struct CellCache {
    pub version: u32,
    pub cells: Vec<CachedLand>,
}

pub fn write_cell_cache(records: &HashMap<u32, RawRecord>, path: &Path) -> Result<usize> {
    let mut cells = Vec::new();
    for record in records
        .values()
        .filter(|record| &record.record_type == b"LAND")
    {
        let view = SubrecordView::new(&record.subrecords);
        cells.push(CachedLand {
            cell_id: record.cell_form_id.unwrap_or(record.form_id),
            heightmap: view.find(b"VHGT").unwrap_or_default().to_vec(),
            normals: view.find(b"VNML").unwrap_or_default().to_vec(),
            vertex_colors: view.find(b"VCLR").unwrap_or_default().to_vec(),
            texture_layers: view.find(b"VTEX").unwrap_or_default().to_vec(),
        });
    }
    cells.sort_unstable_by_key(|cell| cell.cell_id);
    let count = cells.len();
    let bytes = rkyv::to_bytes::<Error>(&CellCache {
        version: CELL_CACHE_VERSION,
        cells,
    })
    .wrap_err("failed to serialize cell cache")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    validate_cell_cache(path)?;
    Ok(count)
}

pub fn validate_cell_cache(path: &Path) -> Result<Mmap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let archived =
        rkyv::access::<ArchivedCellCache, Error>(&mmap).wrap_err("invalid cell cache")?;
    color_eyre::eyre::ensure!(
        archived.version == CELL_CACHE_VERSION,
        "unsupported cell cache version"
    );
    Ok(mmap)
}
