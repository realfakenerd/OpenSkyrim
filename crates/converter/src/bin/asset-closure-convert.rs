use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use converter::mesh::MeshConverter;
use serde::Serialize;
use std::{
    collections::VecDeque,
    env, fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, Serialize)]
struct ConversionResult {
    model_path: String,
    source_path: Option<String>,
    output_path: String,
    status: &'static str,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConversionReport {
    format_version: u32,
    source_root: PathBuf,
    assets_root: PathBuf,
    requested: usize,
    converted: usize,
    reused: usize,
    missing_sources: usize,
    failures: usize,
    passed: bool,
    results: Vec<ConversionResult>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = env::args_os().skip(1);
    let source_root = required(&mut args, "source-mesh-root")?;
    let assets_root = required(&mut args, "assets-root")?;
    let report_path = required(&mut args, "report.json")?;
    if args.next().is_some() {
        bail!("usage: asset-closure-convert <source-mesh-root> <assets-root> <report.json>");
    }
    let source_root = source_root
        .canonicalize()
        .wrap_err_with(|| format!("source root does not exist: {}", source_root.display()))?;
    fs::create_dir_all(&assets_root)?;
    let assets_root = assets_root.canonicalize()?;
    let database = assets_root.join("skyrim_world.db");
    let connection = turso::Builder::new_local(&database.to_string_lossy())
        .build()
        .await?
        .connect()?;
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
    let requested = models.len();
    let queue = Arc::new(Mutex::new(VecDeque::from(models)));
    let results = Arc::new(Mutex::new(Vec::with_capacity(requested)));
    let completed = Arc::new(AtomicUsize::new(0));
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(8);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let completed = Arc::clone(&completed);
            let source_root = &source_root;
            let assets_root = &assets_root;
            scope.spawn(move || {
                loop {
                    let Some(model_path) = queue.lock().unwrap().pop_front() else {
                        break;
                    };
                    let relative = relative_model_path(&model_path);
                    let source = source_root.join(&relative);
                    let output = assets_root
                        .join("meshes")
                        .join(&relative)
                        .with_extension("glb");
                    let relative_output = output
                        .strip_prefix(assets_root)
                        .unwrap_or(&output)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let result = if !source.is_file() {
                        ConversionResult {
                            model_path,
                            source_path: None,
                            output_path: relative_output,
                            status: "missing_source",
                            error: Some("source NIF is missing".to_owned()),
                        }
                    } else if output.is_file()
                        && MeshConverter::glb_bounds(&output).is_ok()
                        && MeshConverter::glb_texture_uris(&output).is_ok()
                    {
                        ConversionResult {
                            model_path,
                            source_path: Some(source.to_string_lossy().into_owned()),
                            output_path: relative_output,
                            status: "reused",
                            error: None,
                        }
                    } else {
                        match MeshConverter::convert_nif_to_glb(&source, &output) {
                            Ok(()) => ConversionResult {
                                model_path,
                                source_path: Some(source.to_string_lossy().into_owned()),
                                output_path: relative_output,
                                status: "converted",
                                error: None,
                            },
                            Err(error) => ConversionResult {
                                model_path,
                                source_path: Some(source.to_string_lossy().into_owned()),
                                output_path: relative_output,
                                status: "failed",
                                error: Some(format!("{error:#}")),
                            },
                        }
                    };
                    results.lock().unwrap().push(result);
                    let count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == requested {
                        eprintln!("Processed {count}/{requested} reachable NIF files");
                    }
                }
            });
        }
    });
    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by_key(|result| result.model_path.to_ascii_lowercase());
    let converted = count_status(&results, "converted");
    let reused = count_status(&results, "reused");
    let missing_sources = count_status(&results, "missing_source");
    let failures = count_status(&results, "failed");
    let passed = converted + reused == requested && missing_sources == 0 && failures == 0;
    let report = ConversionReport {
        format_version: 1,
        source_root,
        assets_root,
        requested,
        converted,
        reused,
        missing_sources,
        failures,
        passed,
        results,
    };
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&report_path, serde_json::to_vec_pretty(&report)?)?;
    println!("closure conversion written to {}", report_path.display());
    if !passed {
        bail!("reachable NIF conversion is incomplete");
    }
    Ok(())
}

fn required(args: &mut impl Iterator<Item = std::ffi::OsString>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| color_eyre::eyre::eyre!("missing {name}"))
}

fn relative_model_path(model_path: &str) -> PathBuf {
    let normalized = model_path.replace('\\', "/");
    PathBuf::from(
        normalized
            .strip_prefix("meshes/")
            .or_else(|| normalized.strip_prefix("Meshes/"))
            .unwrap_or(&normalized),
    )
}

fn count_status(results: &[ConversionResult], status: &str) -> usize {
    results
        .iter()
        .filter(|result| result.status == status)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_optional_meshes_prefix() {
        assert_eq!(
            relative_model_path("Meshes\\Architecture\\Wall.NIF"),
            std::path::Path::new("Architecture/Wall.NIF")
        );
    }
}
