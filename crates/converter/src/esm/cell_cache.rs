use crate::esm::{extractors::SubrecordView, records::RawRecord};
use color_eyre::{Result, eyre::WrapErr};
use memmap2::Mmap;
use rkyv::rancor::Error;
use shared::{CELL_CACHE_VERSION, CachedLand, CellCache, LAND_SIDE, TerrainLayer, TerrainWeight};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

pub fn write_cell_cache(records: &HashMap<u32, RawRecord>, path: &Path) -> Result<usize> {
    let water_by_cell: HashMap<u32, (Option<f32>, Option<u32>)> = records
        .values()
        .filter(|record| &record.record_type == b"CELL")
        .map(|record| {
            let view = SubrecordView::new(&record.subrecords);
            let height = view
                .find(b"XCLW")
                .filter(|bytes| bytes.len() >= 4)
                .map(|bytes| {
                    f32::from_le_bytes(bytes[..4].try_into().expect("four-byte water height"))
                })
                .and_then(normalize_water_height);
            (record.form_id, (height, view.get_form_id(b"XCWT")))
        })
        .collect();
    let mut cells = Vec::new();
    for record in records
        .values()
        .filter(|record| &record.record_type == b"LAND")
    {
        let view = SubrecordView::new(&record.subrecords);
        let heightmap = view.find(b"VHGT").unwrap_or_default();
        let cell_id = record.cell_form_id.unwrap_or(record.form_id);
        let (water_height, water_type_form_id) =
            water_by_cell.get(&cell_id).copied().unwrap_or((None, None));
        cells.push(CachedLand {
            cell_id,
            width: LAND_SIDE,
            height: LAND_SIDE,
            heights: decode_vhgt(heightmap),
            normals: view
                .find(b"VNML")
                .unwrap_or_default()
                .iter()
                .map(|value| *value as i8)
                .collect(),
            vertex_colors: view.find(b"VCLR").unwrap_or_default().to_vec(),
            layers: extract_texture_layers(&record.subrecords),
            water_height,
            water_type_form_id,
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

fn normalize_water_height(height: f32) -> Option<f32> {
    // Skyrim uses FLT_MAX as the exterior-cell "no water" sentinel. Persisting
    // it as a real height creates non-finite render transforms downstream.
    (height.is_finite() && height.abs() < 1.0e7).then_some(height)
}

fn decode_vhgt(bytes: &[u8]) -> Vec<f32> {
    let count = usize::from(LAND_SIDE) * usize::from(LAND_SIDE);
    if bytes.len() < 4 + count {
        return Vec::new();
    }
    let offset = f32::from_le_bytes(bytes[..4].try_into().expect("four-byte VHGT offset")) * 8.0;
    let deltas = &bytes[4..4 + count];
    let side = usize::from(LAND_SIDE);
    let mut heights = vec![0.0; count];
    let mut row_origin = offset;
    for row in 0..side {
        row_origin += (deltas[row * side] as i8 as f32) * 8.0;
        let mut height = row_origin;
        heights[row * side] = height;
        for column in 1..side {
            height += (deltas[row * side + column] as i8 as f32) * 8.0;
            heights[row * side + column] = height;
        }
    }
    heights
}

fn extract_texture_layers(subrecords: &[(Vec<u8>, Vec<u8>)]) -> Vec<TerrainLayer> {
    let mut layers = Vec::new();
    let mut active: Option<usize> = None;
    for (tag, data) in subrecords {
        match tag.as_slice() {
            b"BTXT" | b"ATXT" if data.len() >= 6 => {
                let texture_form_id = u32::from_le_bytes(data[..4].try_into().unwrap());
                let quadrant = data[4];
                let layer = if tag.as_slice() == b"BTXT" {
                    0
                } else {
                    data[5]
                };
                layers.push(TerrainLayer {
                    texture_form_id,
                    quadrant,
                    layer,
                    weights: Vec::new(),
                });
                active = Some(layers.len() - 1);
            }
            b"VTXT" => {
                if let Some(index) = active {
                    for entry in data.chunks_exact(8) {
                        layers[index].weights.push(TerrainWeight {
                            vertex: u16::from_le_bytes(entry[..2].try_into().unwrap()),
                            opacity: f32::from_le_bytes(entry[4..8].try_into().unwrap()),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    layers
}

pub fn validate_cell_cache(path: &Path) -> Result<Mmap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let archived =
        rkyv::access::<shared::ArchivedCellCache, Error>(&mmap).wrap_err("invalid cell cache")?;
    color_eyre::eyre::ensure!(
        archived.version == CELL_CACHE_VERSION,
        "unsupported cell cache version"
    );
    Ok(mmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_vhgt_deltas_into_absolute_heights() {
        let count = usize::from(LAND_SIDE) * usize::from(LAND_SIDE);
        let mut bytes = 2.0f32.to_le_bytes().to_vec();
        bytes.extend(std::iter::repeat_n(1, count));
        let heights = decode_vhgt(&bytes);
        assert_eq!(heights.len(), count);
        assert_eq!(heights[0], 24.0);
        assert_eq!(heights[1], 32.0);
        assert_eq!(heights[usize::from(LAND_SIDE)], 32.0);
    }

    #[test]
    fn rejects_skyrim_no_water_sentinel() {
        assert_eq!(normalize_water_height(f32::MAX), None);
        assert_eq!(normalize_water_height(f32::INFINITY), None);
        assert_eq!(normalize_water_height(-11592.0), Some(-11592.0));
    }
}
