use bevy::prelude::Resource;
use color_eyre::{Result, eyre::WrapErr};
use memmap2::Mmap;
use rkyv::rancor::Error;
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Debug, Clone)]
pub struct TerrainSnapshot {
    pub cell_id: u32,
    pub width: u16,
    pub height: u16,
    pub heights: Vec<f32>,
    pub normals: Vec<i8>,
    pub vertex_colors: Vec<u8>,
    pub layers: Vec<TerrainLayerSnapshot>,
    pub water_height: Option<f32>,
    pub water_type_form_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TerrainLayerSnapshot {
    pub texture_form_id: u32,
    pub quadrant: u8,
    pub layer: u8,
    pub weights: Vec<(u16, f32)>,
}

#[derive(Resource)]
pub struct CellCache {
    mmap: Mmap,
    index: HashMap<u32, usize>,
}

impl CellCache {
    pub fn open(path: &Path) -> Result<Self> {
        let file =
            File::open(path).wrap_err_with(|| format!("failed to open {}", path.display()))?;
        let mmap = unsafe { Mmap::map(&file) }.wrap_err("failed to map cell cache")?;
        let archived = rkyv::access::<shared::ArchivedCellCache, Error>(&mmap)
            .wrap_err("invalid cell cache")?;
        color_eyre::eyre::ensure!(
            archived.version == shared::CELL_CACHE_VERSION,
            "cell cache version {} is unsupported; reconvert assets for version {}",
            archived.version,
            shared::CELL_CACHE_VERSION
        );
        let index = archived
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| (cell.cell_id.into(), index))
            .collect();
        Ok(Self { mmap, index })
    }

    pub fn terrain(&self, cell_id: u32) -> Option<TerrainSnapshot> {
        let archived = rkyv::access::<shared::ArchivedCellCache, Error>(&self.mmap).ok()?;
        let cell = archived.cells.get(*self.index.get(&cell_id)?)?;
        Some(TerrainSnapshot {
            cell_id: cell.cell_id.into(),
            width: cell.width.into(),
            height: cell.height.into(),
            heights: cell.heights.iter().copied().map(Into::into).collect(),
            normals: cell.normals.iter().copied().collect(),
            vertex_colors: cell.vertex_colors.iter().copied().collect(),
            layers: cell
                .layers
                .iter()
                .map(|layer| TerrainLayerSnapshot {
                    texture_form_id: layer.texture_form_id.into(),
                    quadrant: layer.quadrant,
                    layer: layer.layer,
                    weights: layer
                        .weights
                        .iter()
                        .map(|weight| (weight.vertex.into(), weight.opacity.into()))
                        .collect(),
                })
                .collect(),
            water_height: cell.water_height.as_ref().copied().map(Into::into),
            water_type_form_id: cell.water_type_form_id.as_ref().copied().map(Into::into),
        })
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_reads_versioned_terrain() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cell_cache.rkyv");
        let source = shared::CellCache {
            version: shared::CELL_CACHE_VERSION,
            cells: vec![shared::CachedLand {
                cell_id: 42,
                width: 2,
                height: 2,
                heights: vec![1.0, 2.0, 3.0, 4.0],
                normals: vec![0; 12],
                vertex_colors: vec![255; 12],
                layers: vec![],
                water_height: Some(8.0),
                water_type_form_id: Some(7),
            }],
        };
        let bytes = rkyv::to_bytes::<Error>(&source).unwrap();
        std::fs::write(&path, bytes).unwrap();

        let cache = CellCache::open(&path).unwrap();
        let terrain = cache.terrain(42).unwrap();
        assert_eq!(terrain.heights, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(terrain.water_height, Some(8.0));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn rejects_previous_cache_version() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("old.rkyv");
        let bytes = rkyv::to_bytes::<Error>(&shared::CellCache {
            version: shared::CELL_CACHE_VERSION - 1,
            cells: vec![],
        })
        .unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(CellCache::open(&path).is_err());
    }
}
