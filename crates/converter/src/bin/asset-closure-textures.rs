use color_eyre::{Result, eyre::WrapErr};
use converter::texture::TextureConverter;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    env, fs,
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, Deserialize)]
struct ClosureReport {
    assets: Vec<ClosureAsset>,
}

#[derive(Debug, Deserialize)]
struct ClosureAsset {
    glb_path: Option<String>,
    missing_textures: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TextureResult {
    path: String,
    source: Option<String>,
    status: &'static str,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct TextureReport {
    format_version: u32,
    assets_root: PathBuf,
    source_roots: Vec<PathBuf>,
    requested: usize,
    converted: usize,
    reused: usize,
    missing_sources: usize,
    failures: usize,
    passed: bool,
    results: Vec<TextureResult>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = env::args_os().skip(1);
    let closure_path = required(&mut args, "closure-report.json")?;
    let report_path = required(&mut args, "texture-report.json")?;
    let assets_root = required(&mut args, "assets-root")?.canonicalize()?;
    let source_roots = args
        .map(PathBuf::from)
        .map(|path| path.canonicalize())
        .collect::<std::io::Result<Vec<_>>>()?;
    color_eyre::eyre::ensure!(
        !source_roots.is_empty(),
        "at least one DDS source root is required"
    );
    let closure: ClosureReport = serde_json::from_slice(&fs::read(&closure_path)?)?;
    let requests = texture_requests(&closure, &assets_root)?;
    let requested = requests.len();
    let queue = Arc::new(Mutex::new(VecDeque::from(
        requests.into_values().collect::<Vec<_>>(),
    )));
    let results = Arc::new(Mutex::new(Vec::with_capacity(requested)));
    let completed = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let completed = Arc::clone(&completed);
            let source_roots = &source_roots;
            let assets_root = &assets_root;
            scope.spawn(move || {
                loop {
                    let Some(relative) = queue.lock().unwrap().pop_front() else {
                        break;
                    };
                    let output = assets_root.join(&relative);
                    let mut dds_relative = relative.clone();
                    dds_relative.set_extension("dds");
                    let source_relative = dds_relative
                        .strip_prefix("textures")
                        .unwrap_or(&dds_relative);
                    let source = source_roots
                        .iter()
                        .map(|root| root.join(source_relative))
                        .find(|path| path.is_file());
                    let result = if output.is_file() {
                        TextureResult {
                            path: normalize(&relative),
                            source: source.map(|path| path.to_string_lossy().into_owned()),
                            status: "reused",
                            error: None,
                        }
                    } else if let Some(source) = source {
                        match TextureConverter::convert_dds_to_ktx2(
                            &source,
                            &output,
                            TextureConverter::is_normal_map(&source),
                        ) {
                            Ok(()) => TextureResult {
                                path: normalize(&relative),
                                source: Some(source.to_string_lossy().into_owned()),
                                status: "converted",
                                error: None,
                            },
                            Err(error) => TextureResult {
                                path: normalize(&relative),
                                source: Some(source.to_string_lossy().into_owned()),
                                status: "failed",
                                error: Some(format!("{error:#}")),
                            },
                        }
                    } else {
                        TextureResult {
                            path: normalize(&relative),
                            source: None,
                            status: "missing_source",
                            error: Some("DDS source is missing".to_owned()),
                        }
                    };
                    results.lock().unwrap().push(result);
                    let count = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    if count.is_multiple_of(100) || count == requested {
                        eprintln!("Processed {count}/{requested} reachable textures");
                    }
                }
            });
        }
    });
    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by_key(|result| result.path.to_ascii_lowercase());
    let converted = count_status(&results, "converted");
    let reused = count_status(&results, "reused");
    let missing_sources = count_status(&results, "missing_source");
    let failures = count_status(&results, "failed");
    let passed = converted + reused == requested && missing_sources == 0 && failures == 0;
    let report = TextureReport {
        format_version: 1,
        assets_root,
        source_roots,
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
    println!("texture closure written to {}", report_path.display());
    color_eyre::eyre::ensure!(passed, "reachable texture conversion is incomplete");
    Ok(())
}

fn texture_requests(
    closure: &ClosureReport,
    assets_root: &Path,
) -> Result<BTreeMap<String, PathBuf>> {
    let mut requests = BTreeMap::new();
    for asset in &closure.assets {
        let Some(glb_path) = &asset.glb_path else {
            continue;
        };
        let glb = assets_root.join(glb_path);
        for uri in &asset.missing_textures {
            let decoded = uri.replace("%20", " ");
            let candidate = lexical_normalize(&glb.parent().unwrap_or(assets_root).join(decoded));
            let relative = candidate.strip_prefix(assets_root).wrap_err_with(|| {
                format!("texture URI escapes assets root: {uri} in {glb_path}")
            })?;
            requests
                .entry(normalize(relative))
                .or_insert_with(|| relative.to_owned());
        }
    }
    Ok(requests)
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

fn normalize(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn required(args: &mut impl Iterator<Item = std::ffi::OsString>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| color_eyre::eyre::eyre!("missing {name}"))
}

fn count_status(results: &[TextureResult], status: &str) -> usize {
    results
        .iter()
        .filter(|result| result.status == status)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_texture_uri_relative_to_glb() {
        let root = Path::new("C:/assets");
        let closure = ClosureReport {
            assets: vec![ClosureAsset {
                glb_path: Some("meshes/architecture/a.glb".to_owned()),
                missing_textures: vec!["../../textures/a.ktx2".to_owned()],
            }],
        };
        let requests = texture_requests(&closure, root).unwrap();
        assert!(requests.contains_key("textures/a.ktx2"));
    }
}
