use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use converter::texture::TextureConverter;
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = env::args_os().skip(1);
    let source_root = required_path(&mut args, "DDS source root")?;
    let output_root = required_path(&mut args, "KTX2 output root")?;
    let engine_log = required_path(&mut args, "engine log")?;
    if args.next().is_some() {
        bail!(usage());
    }

    let log = fs::read_to_string(&engine_log)
        .wrap_err_with(|| format!("failed to read {}", engine_log.display()))?;
    let requested = missing_texture_paths(&log);
    if requested.is_empty() {
        println!(
            "No missing KTX2 textures were found in {}",
            engine_log.display()
        );
        return Ok(());
    }

    let mut converted = 0usize;
    let mut reused = 0usize;
    let mut missing = Vec::new();
    let mut failed = Vec::new();
    for relative in requested.values() {
        let mut dds_relative = relative.clone();
        dds_relative.set_extension("dds");
        let source = source_root.join(&dds_relative);
        let output = output_root.join(relative);
        if output.is_file() {
            reused += 1;
            continue;
        }
        if !source.is_file() {
            missing.push(dds_relative);
            continue;
        }
        match TextureConverter::convert_dds_to_ktx2(
            &source,
            &output,
            TextureConverter::is_normal_map(&source),
        ) {
            Ok(()) => converted += 1,
            Err(error) => failed.push((dds_relative, error.to_string())),
        }
    }

    println!(
        "Requested {}, converted {}, reused {}, source missing {}, failed {}",
        requested.len(),
        converted,
        reused,
        missing.len(),
        failed.len()
    );
    for path in &missing {
        eprintln!("source missing: {}", path.display());
    }
    for (path, error) in &failed {
        eprintln!("conversion failed: {}: {error}", path.display());
    }
    if !missing.is_empty() || !failed.is_empty() {
        bail!("texture synchronization was incomplete");
    }
    Ok(())
}

fn required_path(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    name: &str,
) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| color_eyre::eyre::eyre!("missing {name}\n{}", usage()))
}

fn usage() -> &'static str {
    "usage: texture-sync <DDS source root> <KTX2 output root> <engine stderr log>"
}

fn missing_texture_paths(log: &str) -> BTreeMap<String, PathBuf> {
    let mut paths = BTreeMap::new();
    for line in log.lines().filter(|line| line.contains("Path not found:")) {
        let normalized = line.replace('\\', "/");
        let lower = normalized.to_ascii_lowercase();
        let Some(texture_start) = lower.rfind("/textures/") else {
            continue;
        };
        let relative = &normalized[texture_start + "/textures/".len()..];
        let Some(end) = relative.to_ascii_lowercase().find(".ktx2") else {
            continue;
        };
        let relative = &relative[..end + ".ktx2".len()];
        let path = Path::new(relative).to_path_buf();
        paths.entry(relative.to_ascii_lowercase()).or_insert(path);
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_and_deduplicates_windows_and_uri_paths() {
        let log = r#"
ERROR Path not found: E:\Runtime\textures\landscape\Rocks01_N.ktx2
ERROR Path not found: E:\Runtime\textures/landscape/Rocks01_N.ktx2
ERROR Path not found: E:\Runtime\meshes\rock.glb
"#;
        let paths = missing_texture_paths(log);
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths.values().next().unwrap(),
            &PathBuf::from("landscape/Rocks01_N.ktx2")
        );
    }
}
