use crate::{
    archive::ArchiveExtractor,
    cache::{CacheEntry, ConversionManifest, configuration_hash, hash_file},
    config::PipelineConfig,
    esm::{EsmParser, cell_cache::write_cell_cache, exporter::validate_database, read_plugins_txt},
    integration::{IntegrationReport, finalize_world_database},
    mesh::MeshConverter,
    progress::{ProgressEvent, ProgressStage},
    script::ScriptConverter,
    texture::TextureConverter,
};
use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::mpsc::{Sender, unbounded_channel},
    task::spawn_blocking,
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineReport {
    pub complete: bool,
    pub converted: u64,
    pub cache_hits: u64,
    pub skipped: u64,
    pub warnings: Vec<String>,
    pub artifacts: Vec<PathBuf>,
    pub inputs_by_kind: BTreeMap<String, u64>,
    pub elapsed_ms: u128,
    pub integration: Option<IntegrationReport>,
}

pub struct AssetPipeline;

impl AssetPipeline {
    pub async fn run_async(
        config: PipelineConfig,
        progress_tx: Sender<ProgressEvent>,
    ) -> Result<PipelineReport> {
        config.validate()?;
        let started = Instant::now();
        send(
            &progress_tx,
            ProgressStage::Discovering,
            0,
            0,
            None,
            "Discovering Skyrim assets",
        )
        .await;
        let previous_manifest =
            ConversionManifest::load(&config.output_dir.join("conversion-manifest.json"))?;
        let expected_configuration = configuration_hash(&config)?;
        let previous_manifest = if previous_manifest.configuration_hash == expected_configuration {
            previous_manifest
        } else {
            ConversionManifest::default()
        };
        let staging = staging_path(&config.output_dir);
        fs::create_dir_all(staging.join("vfs"))?;
        let run_result = Self::run_into(&config, &staging, &previous_manifest, &progress_tx).await;
        let mut report = match run_result {
            Ok(report) => report,
            Err(error) => {
                let _ = fs::remove_dir_all(&staging);
                return Err(error);
            }
        };
        send(
            &progress_tx,
            ProgressStage::Publishing,
            1,
            1,
            None,
            "Publishing converted assets",
        )
        .await;
        publish_directory(&staging, &config.output_dir)?;
        report.elapsed_ms = started.elapsed().as_millis();
        if report.complete {
            send(
                &progress_tx,
                ProgressStage::Complete,
                1,
                1,
                None,
                "Asset conversion complete",
            )
            .await;
        }
        Ok(report)
    }

    async fn run_into(
        config: &PipelineConfig,
        staging: &Path,
        previous: &ConversionManifest,
        progress_tx: &Sender<ProgressEvent>,
    ) -> Result<PipelineReport> {
        let mut report = PipelineReport::default();
        let mut manifest = ConversionManifest {
            schema_version: crate::cache::CONVERTER_SCHEMA_VERSION,
            complete: false,
            configuration_hash: configuration_hash(config)?,
            inputs_by_kind: Default::default(),
            failures: Default::default(),
            entries: Default::default(),
        };
        let files = discover(&config.data_dir)?;
        let archives: Vec<_> = files
            .iter()
            .filter(|path| extension(path, &["bsa", "ba2"]))
            .cloned()
            .collect();
        let enabled_archives: Vec<_> = archives
            .into_iter()
            .filter(|archive| !extension(archive, &["ba2"]) || config.enable_ba2)
            .collect();
        if !enabled_archives.is_empty() {
            send(
                progress_tx,
                ProgressStage::Extracting,
                0,
                enabled_archives.len() as u64,
                None,
                "Extracting Skyrim archives",
            )
            .await;
        }

        let vfs_dir = staging.join("vfs");
        fs::create_dir_all(&vfs_dir)?;

        for (index, archive) in enabled_archives.iter().enumerate() {
            let archive_for_worker = archive.clone();
            let vfs_for_worker = vfs_dir.clone();

            let result = spawn_blocking(move || {
                ArchiveExtractor::extract(&archive_for_worker, &vfs_for_worker)
            })
            .await
            .wrap_err("archive worker panicked")?;

            send(
                progress_tx,
                ProgressStage::Extracting,
                (index + 1) as u64,
                enabled_archives.len() as u64,
                Some(archive.clone()),
                "Extracted archive",
            )
            .await;

            match result {
                Ok(entries) => {
                    report.converted += entries.len() as u64;
                }
                Err(error) if !config.fail_fast => {
                    report.skipped += 1;
                    let message = format!("{}: {error:#}", archive.display());
                    manifest.failures.insert(
                        archive.to_string_lossy().replace('\\', "/"),
                        message.clone(),
                    );
                    report.warnings.push(message);
                }
                Err(error) => return Err(error),
            }
        }

        overlay_loose_assets(&config.data_dir, &staging.join("vfs"), &files)?;

        let plugins = plugin_paths(config, &files)?;
        if !plugins.is_empty() {
            send(
                progress_tx,
                ProgressStage::Database,
                0,
                plugins.len() as u64,
                None,
                "Building skyrim_world.db",
            )
            .await;
            let db_path = staging.join("skyrim_world.db");
            EsmParser::convert_plugins(&plugins, &db_path)?;
            validate_database(&Connection::open(&db_path)?)?;
            let merged = EsmParser::merge_plugins(&plugins)?;
            write_cell_cache(&merged, &staging.join("cell_cache.rkyv"))?;
            report.artifacts.extend([
                PathBuf::from("skyrim_world.db"),
                PathBuf::from("cell_cache.rkyv"),
            ]);
        }

        let vfs_files = discover(&staging.join("vfs"))?;
        {
            let mut batch = ConversionBatch {
                config,
                staging,
                previous,
                manifest: &mut manifest,
                report: &mut report,
                progress_tx,
            };
            batch
                .convert_kind(&vfs_files, "dds", ProgressStage::Textures)
                .await?;
            batch
                .convert_kind(&vfs_files, "nif", ProgressStage::Meshes)
                .await?;
            batch
                .convert_kind(&vfs_files, "pex", ProgressStage::Scripts)
                .await?;
        }
        if let Some(integration) = finalize_world_database(staging)? {
            if !integration.passed {
                report.warnings.push(format!(
                    "asset integration failed: {} missing models, {} invalid models, {} missing textures",
                    integration.missing_model_count,
                    integration.invalid_model_count,
                    integration.missing_texture_count
                ));
            }
            report.integration = Some(integration);
            report
                .artifacts
                .push(PathBuf::from("integration-report.json"));
        }
        let runtime_path = staging.join("scripts/papyrus_runtime.luau");
        if let Some(parent) = runtime_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            &runtime_path,
            include_str!("../../shared/src/papyrus_runtime.luau"),
        )?;
        report
            .artifacts
            .push(PathBuf::from("scripts/papyrus_runtime.luau"));
        send(
            progress_tx,
            ProgressStage::Validating,
            0,
            report.artifacts.len() as u64,
            None,
            "Validating generated artifacts",
        )
        .await;
        validate_artifacts(staging, &report.artifacts)?;
        send(
            progress_tx,
            ProgressStage::Validating,
            report.artifacts.len() as u64,
            report.artifacts.len() as u64,
            None,
            "Generated artifacts are valid",
        )
        .await;
        manifest.complete = report.skipped == 0 && report.warnings.is_empty();
        report.complete = manifest.complete;
        report.inputs_by_kind = manifest.inputs_by_kind.clone();
        manifest.save(&staging.join("conversion-manifest.json"))?;
        report
            .artifacts
            .push(PathBuf::from("conversion-manifest.json"));
        Ok(report)
    }
}

struct ConversionBatch<'a> {
    config: &'a PipelineConfig,
    staging: &'a Path,
    previous: &'a ConversionManifest,
    manifest: &'a mut ConversionManifest,
    report: &'a mut PipelineReport,
    progress_tx: &'a Sender<ProgressEvent>,
}

impl ConversionBatch<'_> {
    async fn convert_kind(
        &mut self,
        files: &[PathBuf],
        source_ext: &str,
        stage: ProgressStage,
    ) -> Result<()> {
        let selected: Vec<_> = files
            .iter()
            .filter(|path| extension(path, &[source_ext]))
            .cloned()
            .collect();

        self.manifest
            .inputs_by_kind
            .insert(source_ext.to_owned(), selected.len() as u64);

        if selected.is_empty() {
            return Ok(());
        }

        let total_files = selected.len() as u64;
        let progress_tx = self.progress_tx.clone();
        let (outcome_tx, mut outcome_rx) = unbounded_channel();

        let staging_vfs = self.staging.join("vfs");
        let staging_root = self.staging.to_path_buf();
        let output_dir = self.config.output_dir.clone();
        let source_kind = source_ext.to_owned();
        let etc1s_quality = self.config.texture_etc1s_quality;
        let uastc_level = self.config.texture_uastc_level;
        let previous_entries = self.previous.entries.clone();

        let rayon_handle = spawn_blocking(move || {
            use rayon::prelude::*;

            selected
                .into_par_iter()
                .enumerate()
                .for_each(|(index, source)| {
                    let relative = match source.strip_prefix(&staging_vfs) {
                        Ok(rel) => rel,
                        Err(err) => {
                            let _ = outcome_tx.send((
                                index,
                                String::new(),
                                String::new(),
                                PathBuf::new(),
                                source.clone(),
                                Err(color_eyre::Report::from(err)),
                                PathBuf::new(),
                            ));
                            return;
                        }
                    };

                    let (folder, target_ext) = match source_kind.as_str() {
                        "dds" => ("textures", "ktx2"),
                        "nif" => ("meshes", "glb"),
                        "pex" => ("scripts", "luau"),
                        _ => unreachable!(),
                    };

                    let mut target_rel =
                        PathBuf::from(folder).join(strip_leading_kind(relative, folder));
                    target_rel.set_extension(target_ext);
                    let target = staging_root.join(&target_rel);
                    let key = relative
                        .to_string_lossy()
                        .replace('\\', "/")
                        .to_ascii_lowercase();

                    let mut hash = match hash_file(&source) {
                        Ok(h) => h,
                        Err(err) => {
                            let _ = outcome_tx.send((
                                index,
                                key,
                                String::new(),
                                target_rel,
                                relative.to_path_buf(),
                                Err(err),
                                target,
                            ));
                            return;
                        }
                    };

                    if source_kind == "nif" {
                        for dependency in MeshConverter::dependency_paths(&source) {
                            match hash_file(&dependency) {
                                Ok(dep_hash) => {
                                    hash.push(':');
                                    hash.push_str(&dep_hash);
                                }
                                Err(err) => {
                                    let _ = outcome_tx.send((
                                        index,
                                        key,
                                        hash,
                                        target_rel,
                                        relative.to_path_buf(),
                                        Err(err),
                                        target,
                                    ));
                                    return;
                                }
                            }
                        }
                    }

                    // Check cache
                    if let Some(entry) =
                        previous_entries.get(&key).filter(|e| e.source_hash == hash)
                    {
                        let old = output_dir.join(&entry.output);
                        if old.is_file()
                            && fs::metadata(&old).is_ok_and(|m| m.len() == entry.output_size)
                            && hash_file(&old).is_ok_and(|h| h == entry.output_hash)
                        {
                            if let Some(parent) = target.parent() {
                                let _ = fs::create_dir_all(parent);
                            }
                            if fs::copy(&old, &target).is_ok() {
                                let _ = outcome_tx.send((
                                    index,
                                    key,
                                    hash,
                                    target_rel,
                                    relative.to_path_buf(),
                                    Ok(true), // is_cache_hit = true
                                    target,
                                ));
                                return;
                            };
                        }
                    }

                    let result = match source_kind.as_str() {
                        "dds" => TextureConverter::convert_dds_to_ktx2_with_options(
                            &source,
                            &target,
                            TextureConverter::is_normal_map(&source),
                            etc1s_quality,
                            uastc_level,
                        ),
                        "nif" => MeshConverter::convert_nif_to_glb(&source, &target),
                        "pex" => ScriptConverter::convert_pex_to_luau(&source, &target),
                        _ => unreachable!(),
                    };

                    let _ = outcome_tx.send((
                        index,
                        key,
                        hash,
                        target_rel,
                        relative.to_path_buf(),
                        result.map(|_| false), // is_cache_hit = false
                        target,
                    ));
                });
        });

        let mut completed = 0u64;
        while let Some((_, key, hash, target_rel, relative, conversion, target)) =
            outcome_rx.recv().await
        {
            completed += 1;
            send(
                &progress_tx,
                stage,
                completed,
                total_files,
                Some(relative.clone()),
                "Converted asset",
            )
            .await;

            match conversion {
                Ok(is_cache_hit) => {
                    if is_cache_hit {
                        if let Some(entry) = self.previous.entries.get(&key) {
                            self.manifest.entries.insert(key, entry.clone());
                        }
                        self.report.cache_hits += 1;
                    } else {
                        let size = fs::metadata(&target)?.len();
                        let output_hash = hash_file(&target)?;
                        self.manifest.entries.insert(
                            key,
                            CacheEntry {
                                source_hash: hash,
                                output: target_rel
                                    .to_string_lossy()
                                    .into_owned()
                                    .replace('\\', "/"),
                                output_size: size,
                                output_hash,
                            },
                        );
                        self.report.converted += 1;
                    }
                    self.report.artifacts.push(target_rel);
                }
                Err(error) => return Err(error),
            }
        }

        rayon_handle.await.wrap_err("rayon batch worker panicked")?;
        Ok(())
    }
}

fn discover(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<_> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();
    files.sort_by_key(|path| path.to_string_lossy().to_ascii_lowercase());
    Ok(files)
}

fn validate_artifacts(staging: &Path, artifacts: &[PathBuf]) -> Result<()> {
    let lua = mlua::Lua::new();
    for relative in artifacts {
        let path = staging.join(relative);
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("ktx2") => {
                let bytes = fs::read(&path)?;
                ktx2::Reader::new(&bytes).map_err(|error| {
                    color_eyre::eyre::eyre!("invalid KTX2 {}: {error:?}", path.display())
                })?;
            }
            Some("glb") => {
                let bytes = fs::read(&path)?;
                if bytes.len() < 12 || &bytes[..4] != b"glTF" {
                    bail!("invalid GLB artifact {}", path.display());
                }
            }
            Some("luau") => {
                let source = fs::read_to_string(&path)?;
                lua.load(&source)
                    .set_name(path.to_string_lossy())
                    .into_function()
                    .map_err(|error| {
                        color_eyre::eyre::eyre!("invalid Luau artifact {}: {error}", path.display())
                    })?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn overlay_loose_assets(data: &Path, vfs: &Path, files: &[PathBuf]) -> Result<()> {
    for source in files
        .iter()
        .filter(|path| extension(path, &["dds", "nif", "pex"]))
    {
        let relative = source.strip_prefix(data)?;
        let destination = vfs.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn plugin_paths(config: &PipelineConfig, files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    if let Some(path) = &config.plugins_file {
        return read_plugins_txt(path, &config.data_dir);
    }
    let mut plugins: Vec<_> = files
        .iter()
        .filter(|path| extension(path, &["esm", "esp", "esl"]))
        .cloned()
        .collect();
    plugins.sort_by_key(|path| {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        let rank = match name.as_str() {
            "skyrim.esm" => 0,
            "update.esm" => 1,
            "dawnguard.esm" => 2,
            "hearthfires.esm" => 3,
            "dragonborn.esm" => 4,
            _ => 10,
        };
        (rank, name)
    });
    Ok(plugins)
}

fn extension(path: &Path, expected: &[&str]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            expected
                .iter()
                .any(|expected| value.eq_ignore_ascii_case(expected))
        })
}

fn strip_leading_kind<'a>(path: &'a Path, kind: &str) -> &'a Path {
    path.strip_prefix(kind).unwrap_or(path)
}

fn staging_path(output: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    output.with_extension(format!("staging-{}-{stamp}", std::process::id()))
}

fn publish_directory(staging: &Path, output: &Path) -> Result<()> {
    let backup = output.with_extension(format!("backup-{}", std::process::id()));
    if backup.exists() {
        bail!("refusing to overwrite stale backup {}", backup.display());
    }
    if output.exists() {
        fs::rename(output, &backup).wrap_err("failed to preserve previous asset output")?;
    }
    if let Err(error) = fs::rename(staging, output) {
        if backup.exists() {
            let _ = fs::rename(&backup, output);
        }
        return Err(error).wrap_err("failed to publish converted assets");
    }
    if backup.exists() {
        fs::remove_dir_all(backup)?;
    }
    Ok(())
}

async fn send(
    tx: &Sender<ProgressEvent>,
    stage: ProgressStage,
    completed: u64,
    total: u64,
    current_file: Option<PathBuf>,
    message: &str,
) {
    let _ = tx
        .send(ProgressEvent {
            stage,
            completed,
            total,
            current_file,
            message: message.to_owned(),
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn normalizes_target_layout() {
        assert_eq!(
            strip_leading_kind(Path::new("textures/a/b.dds"), "textures"),
            Path::new("a/b.dds")
        );
    }

    #[tokio::test]
    async fn converts_and_reuses_assets_end_to_end() {
        let temp = tempfile::tempdir().unwrap();
        let data = temp.path().join("Data");
        let output = temp.path().join("modern");
        fs::create_dir_all(data.join("scripts")).unwrap();
        fs::write(data.join("scripts/one.pex"), minimal_pex("One")).unwrap();
        fs::write(data.join("scripts/two.pex"), minimal_pex("Two")).unwrap();
        let mut config = PipelineConfig::new(&data, &output);
        config.cpu_jobs = 2;
        let (tx, mut rx) = mpsc::channel(64);
        let drain = tokio::spawn(async move { while rx.recv().await.is_some() {} });
        let first = AssetPipeline::run_async(config.clone(), tx).await.unwrap();
        drain.await.unwrap();
        assert_eq!(first.converted, 2);
        assert!(output.join("scripts/one.luau").is_file());
        assert!(output.join("scripts/papyrus_runtime.luau").is_file());

        let (tx, mut rx) = mpsc::channel(64);
        let drain = tokio::spawn(async move { while rx.recv().await.is_some() {} });
        let second = AssetPipeline::run_async(config, tx).await.unwrap();
        drain.await.unwrap();
        assert_eq!(second.cache_hits, 2);
        assert_eq!(second.skipped, 0);
        assert!(
            ConversionManifest::load(&output.join("conversion-manifest.json"))
                .unwrap()
                .complete
        );
    }

    fn minimal_pex(object_name: &str) -> Vec<u8> {
        fn be16(out: &mut Vec<u8>, value: u16) {
            out.extend_from_slice(&value.to_be_bytes());
        }
        fn be32(out: &mut Vec<u8>, value: u32) {
            out.extend_from_slice(&value.to_be_bytes());
        }
        fn string(out: &mut Vec<u8>, value: &str) {
            be16(out, value.len() as u16);
            out.extend_from_slice(value.as_bytes());
        }
        let strings = [object_name, "", "ObjectReference", "Run", "None"];
        let mut bytes = 0xFA57_C0DEu32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&[3, 2]);
        be16(&mut bytes, 1);
        bytes.extend_from_slice(&0u64.to_be_bytes());
        for value in ["test.psc", "user", "machine"] {
            string(&mut bytes, value);
        }
        be16(&mut bytes, strings.len() as u16);
        for value in strings {
            string(&mut bytes, value);
        }
        bytes.push(0);
        be16(&mut bytes, 0);
        be16(&mut bytes, 1);
        be16(&mut bytes, 0);
        be32(&mut bytes, 0);
        be16(&mut bytes, 2);
        be16(&mut bytes, 1);
        be32(&mut bytes, 0);
        be16(&mut bytes, 1);
        be16(&mut bytes, 0);
        be16(&mut bytes, 0);
        be16(&mut bytes, 1);
        be16(&mut bytes, 1);
        be16(&mut bytes, 1);
        be16(&mut bytes, 3);
        be16(&mut bytes, 4);
        be16(&mut bytes, 1);
        be32(&mut bytes, 0);
        bytes.push(0);
        be16(&mut bytes, 0);
        be16(&mut bytes, 0);
        be16(&mut bytes, 1);
        bytes.extend_from_slice(&[26, 0]);
        bytes
    }
}
