use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use converter::mesh::MeshConverter;
use serde::Serialize;
use std::{
    collections::HashMap,
    env, fs,
    path::{Component, Path, PathBuf},
};
use turso::Builder;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct ClosureAsset {
    model_path: String,
    glb_path: Option<String>,
    bounds: Option<SerializableBounds>,
    texture_uris: Vec<String>,
    missing_textures: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SerializableBounds {
    min: [f32; 3],
    max: [f32; 3],
}

impl From<shared::Bounds3> for SerializableBounds {
    fn from(bounds: shared::Bounds3) -> Self {
        Self {
            min: bounds.min,
            max: bounds.max,
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct ClosureSummary {
    unique_models: usize,
    valid_models: usize,
    missing_models: usize,
    invalid_models: usize,
    external_texture_references: usize,
    missing_texture_references: usize,
}

#[derive(Debug, Serialize)]
struct ClosureReport {
    format_version: u32,
    record_scope: [&'static str; 3],
    assets_root: PathBuf,
    database_schema: u32,
    summary: ClosureSummary,
    assets: Vec<ClosureAsset>,
    passed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = env::args_os().skip(1);
    let assets_root = args.next().map(PathBuf::from).ok_or_else(|| {
        color_eyre::eyre::eyre!("usage: asset-closure <assets-root> <report.json>")
    })?;
    let report_path = args.next().map(PathBuf::from).ok_or_else(|| {
        color_eyre::eyre::eyre!("usage: asset-closure <assets-root> <report.json>")
    })?;
    if args.next().is_some() {
        bail!("usage: asset-closure <assets-root> <report.json>");
    }
    let assets_root = assets_root
        .canonicalize()
        .wrap_err_with(|| format!("assets root does not exist: {}", assets_root.display()))?;
    let database_path = assets_root.join("skyrim_world.db");
    let db = Builder::new_local(&database_path.to_string_lossy())
        .build()
        .await
        .wrap_err_with(|| format!("failed to open {}", database_path.display()))?;
    let connection = db.connect()?;
    let database_schema = {
        let mut rows = connection
            .query("SELECT version FROM schema_info LIMIT 1", ())
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("schema_info table is empty"))?;
        row.get::<u32>(0)?
    };
    let mut models = {
        let mut rows = connection
            .query(
                "SELECT DISTINCT model_path FROM statics \
                 WHERE model_path IS NOT NULL AND model_path <> '' \
                 ORDER BY model_path COLLATE NOCASE",
                (),
            )
            .await?;
        let mut models = Vec::new();
        while let Some(row) = rows.next().await? {
            models.push(row.get::<String>(0)?);
        }
        models
    };
    models.sort_by_key(|model| model.to_ascii_lowercase());
    models.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

    let files = file_index(&assets_root)?;
    let mut summary = ClosureSummary {
        unique_models: models.len(),
        ..Default::default()
    };
    let mut assets = Vec::with_capacity(models.len());
    for model_path in models {
        let key = converted_model_key(&model_path);
        let Some(glb_path) = files.get(&key) else {
            summary.missing_models += 1;
            assets.push(ClosureAsset {
                model_path,
                glb_path: None,
                bounds: None,
                texture_uris: Vec::new(),
                missing_textures: Vec::new(),
                error: Some("converted GLB is missing".to_owned()),
            });
            continue;
        };
        let relative_glb = normalize(glb_path.strip_prefix(&assets_root).unwrap_or(glb_path));
        match inspect_asset(&assets_root, glb_path) {
            Ok((bounds, texture_uris, missing_textures)) => {
                summary.external_texture_references += texture_uris.len();
                summary.missing_texture_references += missing_textures.len();
                if missing_textures.is_empty() {
                    summary.valid_models += 1;
                } else {
                    summary.invalid_models += 1;
                }
                assets.push(ClosureAsset {
                    model_path,
                    glb_path: Some(relative_glb),
                    bounds: Some(bounds.into()),
                    texture_uris,
                    missing_textures,
                    error: None,
                });
            }
            Err(error) => {
                summary.invalid_models += 1;
                assets.push(ClosureAsset {
                    model_path,
                    glb_path: Some(relative_glb),
                    bounds: None,
                    texture_uris: Vec::new(),
                    missing_textures: Vec::new(),
                    error: Some(format!("{error:#}")),
                });
            }
        }
    }
    let passed = database_schema == shared::WORLD_DATABASE_SCHEMA_VERSION
        && summary.valid_models == summary.unique_models
        && summary.missing_models == 0
        && summary.invalid_models == 0
        && summary.missing_texture_references == 0;
    let report = ClosureReport {
        format_version: 1,
        record_scope: ["STAT", "MSTT", "FURN"],
        assets_root,
        database_schema,
        summary,
        assets,
        passed,
    };
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)
        .wrap_err_with(|| format!("failed to write {}", report_path.display()))?;
    println!("asset closure written to {}", report_path.display());
    if !report.passed {
        bail!("Phase 2 asset closure failed");
    }
    Ok(())
}

fn inspect_asset(
    assets_root: &Path,
    glb_path: &Path,
) -> Result<(shared::Bounds3, Vec<String>, Vec<String>)> {
    let bounds = MeshConverter::glb_bounds(glb_path)?;
    let texture_uris = MeshConverter::glb_texture_uris(glb_path)?;
    let mut missing = Vec::new();
    for uri in &texture_uris {
        if uri.starts_with("data:") {
            continue;
        }
        let decoded = uri.replace("%20", " ");
        let candidate = lexical_normalize(&glb_path.parent().unwrap_or(assets_root).join(decoded));
        if !is_within(&candidate, assets_root) || !candidate.is_file() {
            missing.push(uri.clone());
        }
    }
    Ok((bounds, texture_uris, missing))
}

fn file_index(root: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut files = HashMap::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(root)?;
            files.insert(normalize(relative), entry.into_path());
        }
    }
    Ok(files)
}

fn converted_model_key(source: &str) -> String {
    let normalized = source.replace('\\', "/");
    let without_meshes = normalized
        .strip_prefix("meshes/")
        .or_else(|| normalized.strip_prefix("Meshes/"))
        .unwrap_or(&normalized);
    let mut path = PathBuf::from("meshes").join(without_meshes);
    path.set_extension("glb");
    normalize(&path)
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                output.pop();
            }
            other => output.push(other.as_os_str()),
        }
    }
    output
}

fn is_within(path: &Path, root: &Path) -> bool {
    let path = normalize(path);
    let mut root = normalize(root);
    if !root.ends_with('/') {
        root.push('/');
    }
    path == root.trim_end_matches('/') || path.starts_with(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_model_paths() {
        assert_eq!(
            converted_model_key("Meshes\\Architecture\\Wall.NIF"),
            "meshes/architecture/wall.glb"
        );
    }

    #[test]
    fn rejects_texture_escape_outside_assets() {
        let root = Path::new("C:/assets");
        assert!(is_within(
            &lexical_normalize(&root.join("meshes/a/../../textures/a.ktx2")),
            root
        ));
        assert!(!is_within(
            &lexical_normalize(&root.join("meshes/../../../secret.ktx2")),
            root
        ));
    }
}
