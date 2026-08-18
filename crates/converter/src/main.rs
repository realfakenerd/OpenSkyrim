use color_eyre::{
    Result,
    eyre::{WrapErr, bail},
};
use converter::{AssetPipeline, PipelineConfig, ProgressEvent};
use std::{ffi::OsString, fs, path::PathBuf};
use tokio::sync::mpsc;

#[derive(Debug)]
struct Cli {
    data: PathBuf,
    output: PathBuf,
    report_json: Option<PathBuf>,
    cpu_jobs: Option<usize>,
    io_jobs: Option<usize>,
    fail_fast: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    suppress_caught_nif_parser_panics();
    let cli = parse_cli(std::env::args_os().skip(1).collect())?;
    let mut config = PipelineConfig::new(cli.data, cli.output);
    config.fail_fast = cli.fail_fast;
    if let Some(cpu_jobs) = cli.cpu_jobs {
        config.cpu_jobs = cpu_jobs;
    }
    if let Some(io_jobs) = cli.io_jobs {
        config.io_jobs = io_jobs;
    }
    let (tx, mut rx) = mpsc::channel::<ProgressEvent>(128);
    let printer = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            println!(
                "{:?} {:.0}% {}",
                event.stage,
                event.fraction() * 100.0,
                event.message
            );
        }
    });
    let report = AssetPipeline::run_async(config, tx).await?;
    if let Some(path) = cli.report_json {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, serde_json::to_vec_pretty(&report)?)
            .wrap_err_with(|| format!("failed to write {}", path.display()))?;
    }
    println!(
        "Converted {}, reused {}, skipped {} in {} ms (complete: {})",
        report.converted, report.cache_hits, report.skipped, report.elapsed_ms, report.complete
    );
    printer.await?;
    if !report.complete {
        bail!(
            "conversion produced {} warning(s) and {} skipped input(s); see conversion-manifest.json",
            report.warnings.len(),
            report.skipped
        );
    }
    Ok(())
}

fn suppress_caught_nif_parser_panics() {
    let report_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let is_nif_parser = info
            .location()
            .is_some_and(|location| location.file().contains("project-wormhole-nif-"));
        if !is_nif_parser {
            report_panic(info);
        }
    }));
}

fn parse_cli(args: Vec<OsString>) -> Result<Cli> {
    let mut positional = Vec::new();
    let mut report_json = None;
    let mut cpu_jobs = None;
    let mut io_jobs = None;
    let mut fail_fast = false;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--report-json") => {
                report_json = Some(PathBuf::from(next_value(&mut args, "--report-json")?))
            }
            Some("--cpu-jobs") => {
                cpu_jobs = Some(parse_jobs(
                    next_value(&mut args, "--cpu-jobs")?,
                    "--cpu-jobs",
                )?)
            }
            Some("--io-jobs") => {
                io_jobs = Some(parse_jobs(
                    next_value(&mut args, "--io-jobs")?,
                    "--io-jobs",
                )?)
            }
            Some("--fail-fast") => fail_fast = true,
            Some("--help" | "-h") => bail!(usage()),
            Some(flag) if flag.starts_with('-') => bail!("unknown option {flag}\n{}", usage()),
            _ => positional.push(PathBuf::from(argument)),
        }
    }
    if positional.is_empty() || positional.len() > 2 {
        bail!(usage());
    }
    Ok(Cli {
        data: positional.remove(0),
        output: positional
            .pop()
            .unwrap_or_else(|| PathBuf::from("modern_assets")),
        report_json,
        cpu_jobs,
        io_jobs,
        fail_fast,
    })
}

fn next_value(args: &mut impl Iterator<Item = OsString>, option: &str) -> Result<OsString> {
    args.next()
        .ok_or_else(|| color_eyre::eyre::eyre!("{option} requires a value"))
}

fn parse_jobs(value: OsString, option: &str) -> Result<usize> {
    value
        .to_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("{option} value is not valid UTF-8"))?
        .parse()
        .wrap_err_with(|| format!("{option} requires a positive integer"))
}

fn usage() -> &'static str {
    "usage: converter <Skyrim Data> [output directory] [--cpu-jobs N] [--io-jobs N] [--fail-fast] [--report-json FILE]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pipeline_options() {
        let cli = parse_cli(
            [
                "Data",
                "output",
                "--cpu-jobs",
                "8",
                "--io-jobs",
                "2",
                "--fail-fast",
                "--report-json",
                "report.json",
            ]
            .into_iter()
            .map(OsString::from)
            .collect(),
        )
        .unwrap();
        assert_eq!(cli.cpu_jobs, Some(8));
        assert_eq!(cli.io_jobs, Some(2));
        assert!(cli.fail_fast);
        assert_eq!(cli.report_json, Some(PathBuf::from("report.json")));
    }
}
