//! Final, asset-aware validation performed after all offline conversions.

use crate::mesh::MeshConverter;
use color_eyre::{Result, eyre::WrapErr};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use turso::{Builder, Connection, params};
use walkdir::WalkDir;

const MAX_REPORTED_ISSUES: usize = 100;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrationReport {
    pub schema_version: u32,
    pub statics_total: u64,
    pub statics_with_models: u64,
    pub bounds_updated: u64,
    pub references_total: u64,
    pub exterior_cells: u64,
    pub terrain_cells: u64,
    pub cache_cells: u64,
    pub texture_sets_with_diffuse: u64,
    pub waters_with_flow_normal: u64,
    pub missing_model_count: u64,
    pub invalid_model_count: u64,
    pub missing_texture_count: u64,
    pub issues: Vec<String>,
    pub passed: bool,
}

pub async fn finalize_world_database(staging: &Path) -> Result<Option<IntegrationReport>> {
    let database_path = staging.join("skyrim_world.db");
    if !database_path.is_file() {
        return Ok(None);
    }
    let mut connection = Builder::new_local(&database_path.to_string_lossy())
        .build()
        .await?
        .connect()?;
    let mut report = IntegrationReport {
        schema_version: count(&connection, "SELECT version FROM schema_info LIMIT 1").await? as u32,
        statics_total: count(&connection, "SELECT count(*) FROM statics").await?,
        statics_with_models: count(
            &connection,
            "SELECT count(*) FROM statics WHERE model_path IS NOT NULL AND model_path <> ''",
        )
        .await?,
        references_total: count(&connection, "SELECT count(*) FROM \"references\"").await?,
        exterior_cells: count(
            &connection,
            "SELECT count(*) FROM cells WHERE worldspace_id IS NOT NULL AND grid_x IS NOT NULL AND grid_y IS NOT NULL",
        )
        .await?,
        terrain_cells: count(&connection, "SELECT count(*) FROM land").await?,
        texture_sets_with_diffuse: count(
            &connection,
            "SELECT count(*) FROM texture_sets WHERE diffuse_path IS NOT NULL AND diffuse_path <> ''",
        )
        .await?,
        waters_with_flow_normal: count(
            &connection,
            "SELECT count(*) FROM waters WHERE flow_normal_path IS NOT NULL AND flow_normal_path <> ''",
        )
        .await?,
        ..Default::default()
    };
    let files = converted_file_index(staging)?;
    let static_models = {
        let mut rows = connection
            .query(
                "SELECT id,model_path FROM statics WHERE model_path IS NOT NULL AND model_path <> '' ORDER BY id",
                (),
            )
            .await?;
        let mut static_models = Vec::new();
        while let Some(row) = rows.next().await? {
            static_models.push((row.get::<u32>(0)?, row.get::<String>(1)?));
        }
        static_models
    };
    let transaction = connection.transaction().await?;
    for (form_id, model_path) in static_models {
        let key = converted_key(&model_path, "meshes", "glb");
        let Some(path) = files.get(&key) else {
            report.missing_model_count += 1;
            issue(
                &mut report,
                format!("missing model {model_path} for {form_id:08X}"),
            );
            continue;
        };
        match MeshConverter::glb_bounds(path) {
            Ok(bounds) => {
                transaction
                    .execute(
                        "UPDATE statics SET bounds_min_x=?1,bounds_min_y=?2,bounds_min_z=?3,bounds_max_x=?4,bounds_max_y=?5,bounds_max_z=?6,bounds_valid=1 WHERE id=?7",
                        params![
                            bounds.min[0] as f64,
                            bounds.min[1] as f64,
                            bounds.min[2] as f64,
                            bounds.max[0] as f64,
                            bounds.max[1] as f64,
                            bounds.max[2] as f64,
                            form_id
                        ],
                    )
                    .await?;
                report.bounds_updated += 1;
            }
            Err(error) => {
                report.invalid_model_count += 1;
                issue(
                    &mut report,
                    format!("invalid bounds for {model_path}: {error:#}"),
                );
            }
        }
    }
    transaction.commit().await?;

    let diffuse_paths = {
        let mut rows = connection
            .query(
                "SELECT id,diffuse_path FROM texture_sets WHERE diffuse_path IS NOT NULL AND diffuse_path <> '' ORDER BY id",
                (),
            )
            .await?;
        let mut diffuse_paths = Vec::new();
        while let Some(row) = rows.next().await? {
            diffuse_paths.push((row.get::<u32>(0)?, row.get::<String>(1)?));
        }
        diffuse_paths
    };
    for (form_id, texture_path) in diffuse_paths {
        let key = converted_key(&texture_path, "textures", "ktx2");
        if !files.contains_key(&key) {
            report.missing_texture_count += 1;
            issue(
                &mut report,
                format!("missing diffuse texture {texture_path} for TXST {form_id:08X}"),
            );
        }
    }
    let flow_paths = {
        let mut rows = connection
            .query(
                "SELECT id,flow_normal_path FROM waters WHERE flow_normal_path IS NOT NULL AND flow_normal_path <> '' ORDER BY id",
                (),
            )
            .await?;
        let mut flow_paths = Vec::new();
        while let Some(row) = rows.next().await? {
            flow_paths.push((row.get::<u32>(0)?, row.get::<String>(1)?));
        }
        flow_paths
    };
    for (form_id, texture_path) in flow_paths {
        let key = converted_key(&texture_path, "textures", "ktx2");
        if !files.contains_key(&key) {
            report.missing_texture_count += 1;
            issue(
                &mut report,
                format!("missing flow-normal texture {texture_path} for WATR {form_id:08X}"),
            );
        }
    }
    let cache_path = staging.join("cell_cache.rkyv");
    if cache_path.is_file() {
        let mmap = crate::esm::cell_cache::validate_cell_cache(&cache_path)?;
        let cache = rkyv::access::<shared::ArchivedCellCache, rkyv::rancor::Error>(&mmap)
            .wrap_err("invalid integration cell cache")?;
        report.cache_cells = cache.cells.len() as u64;
        if report.cache_cells != report.terrain_cells {
            let database_cells = report.terrain_cells;
            let cache_cells = report.cache_cells;
            issue(
                &mut report,
                format!(
                    "terrain/cache cell count mismatch: database={}, cache={}",
                    database_cells, cache_cells
                ),
            );
        }
    } else {
        issue(&mut report, "missing cell_cache.rkyv".to_owned());
    }
    report.passed = report.schema_version == shared::WORLD_DATABASE_SCHEMA_VERSION
        && report.bounds_updated == report.statics_with_models
        && report.missing_model_count == 0
        && report.invalid_model_count == 0
        && report.missing_texture_count == 0
        && report.cache_cells == report.terrain_cells;
    let output = staging.join("integration-report.json");
    fs::write(&output, serde_json::to_vec_pretty(&report)?)
        .wrap_err_with(|| format!("failed to write {}", output.display()))?;
    Ok(Some(report))
}

fn converted_file_index(staging: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut files = HashMap::new();
    for entry in WalkDir::new(staging).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(staging)?;
            files.insert(normalize(relative), entry.into_path());
        }
    }
    Ok(files)
}

fn converted_key(source: &str, kind: &str, extension: &str) -> String {
    let normalized = source.replace('\\', "/");
    let without_kind = normalized
        .strip_prefix(&format!("{kind}/"))
        .or_else(|| normalized.strip_prefix(&format!("{}/", capitalize(kind))))
        .unwrap_or(&normalized);
    let mut path = PathBuf::from(kind).join(without_kind);
    path.set_extension(extension);
    normalize(&path)
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

async fn count(connection: &Connection, sql: &str) -> Result<u64> {
    let mut rows = connection.query(sql, ()).await?;
    let row = rows
        .next()
        .await?
        .ok_or_else(|| color_eyre::eyre::eyre!("query returned no rows: {sql}"))?;
    Ok(row.get(0)?)
}

fn issue(report: &mut IntegrationReport, message: String) {
    if report.issues.len() < MAX_REPORTED_ISSUES {
        report.issues.push(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_creation_paths_to_converted_assets() {
        assert_eq!(
            converted_key("Meshes\\Architecture\\Wall.NIF", "meshes", "glb"),
            "meshes/architecture/wall.glb"
        );
        assert_eq!(
            converted_key("land/grass.dds", "textures", "ktx2"),
            "textures/land/grass.ktx2"
        );
    }

    #[tokio::test]
    async fn enriches_database_with_real_glb_bounds() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("skyrim_world.db");
        let connection = Builder::new_local(&database.to_string_lossy())
            .build()
            .await
            .unwrap()
            .connect()
            .unwrap();
        crate::esm::exporter::create_tables(&connection)
            .await
            .unwrap();
        connection
            .execute(
                "INSERT INTO statics(id,model_path,flags) VALUES(1,'architecture/wall.nif',0)",
                (),
            )
            .await
            .unwrap();
        drop(connection);
        let mesh_path = directory.path().join("meshes/architecture/wall.glb");
        fs::create_dir_all(mesh_path.parent().unwrap()).unwrap();
        let mut json = br#"{
            "asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],
            "nodes":[{"mesh":0}],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
            "accessors":[{"min":[-2,-3,-4],"max":[5,6,7]}]
        }"#
        .to_vec();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let total = 20 + json.len();
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend_from_slice(&json);
        fs::write(mesh_path, glb).unwrap();
        let cache = shared::CellCache {
            version: shared::CELL_CACHE_VERSION,
            cells: vec![],
        };
        fs::write(
            directory.path().join("cell_cache.rkyv"),
            rkyv::to_bytes::<rkyv::rancor::Error>(&cache).unwrap(),
        )
        .unwrap();

        let report = finalize_world_database(directory.path())
            .await
            .unwrap()
            .unwrap();
        assert!(report.passed);
        assert_eq!(report.bounds_updated, 1);
        let connection = Builder::new_local(&database.to_string_lossy())
            .build()
            .await
            .unwrap()
            .connect()
            .unwrap();
        let row = connection
            .query(
                "SELECT bounds_min_x,bounds_max_z,bounds_valid FROM statics WHERE id=1",
                (),
            )
            .await
            .unwrap()
            .next()
            .await
            .unwrap()
            .unwrap();
        let bounds: (f32, f32, i32) = (
            row.get::<f64>(0).unwrap() as f32,
            row.get::<f64>(1).unwrap() as f32,
            row.get(2).unwrap(),
        );
        assert_eq!(bounds, (-2.0, 7.0, 1));
    }
}
