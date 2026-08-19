use crate::{
    config::EngineConfig,
    profiling::{ProfilingState, SystemMetadata},
    streaming::StreamingMetrics,
};
use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, SystemInfo,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use serde::Serialize;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct AcceptanceMetricsPlugin;

impl Plugin for AcceptanceMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BenchmarkSamples>()
            .add_plugins((
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
            ))
            .add_systems(Last, collect_and_finish);
    }
}

#[derive(Resource, Default)]
struct BenchmarkSamples {
    frames_seen: u32,
    frame_ms: Vec<f64>,
    peak_process_memory_gib: f64,
    first_process_memory_gib: Option<f64>,
    last_process_memory_gib: Option<f64>,
    finished: bool,
}

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    format_version: u32,
    generated_unix_ms: u128,
    scenario: String,
    frames: usize,
    warmup_frames: u32,
    synthetic_instances: usize,
    elapsed_seconds: f64,
    average_fps: f64,
    frame_ms_mean: f64,
    frame_ms_p50: f64,
    frame_ms_p95: f64,
    frame_ms_p99: f64,
    frame_ms_worst: f64,
    peak_process_memory_gib: Option<f64>,
    process_memory_growth_gib: Option<f64>,
    entity_count: Option<u64>,
    system: Option<SystemSnapshot>,
    streaming: Option<StreamingMetrics>,
    thresholds: Thresholds,
    passed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SystemSnapshot {
    os: String,
    kernel: String,
    cpu: String,
    core_count: String,
    memory: String,
}

#[derive(Debug, Serialize)]
struct Thresholds {
    minimum_average_fps: f64,
    maximum_p95_frame_ms: f64,
    maximum_memory_growth_gib: f64,
    no_streaming_failures: bool,
}

#[allow(clippy::too_many_arguments)]
fn collect_and_finish(
    time: Res<Time>,
    config: Res<EngineConfig>,
    diagnostics: Res<DiagnosticsStore>,
    system: Option<Res<SystemInfo>>,
    streaming: Option<Res<StreamingMetrics>>,
    mut samples: ResMut<BenchmarkSamples>,
    mut profiler: ResMut<ProfilingState>,
    mut exit: MessageWriter<AppExit>,
) {
    if samples.finished
        || (config.benchmark_frames.is_none() && config.benchmark_duration_secs.is_none())
    {
        return;
    }
    samples.frames_seen = samples.frames_seen.saturating_add(1);
    let process_memory = diagnostic_value(
        &diagnostics,
        &SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
    );
    profiler.sample_frame(&diagnostics, process_memory);
    if let Some(memory) = process_memory {
        samples.peak_process_memory_gib = samples.peak_process_memory_gib.max(memory);
        if samples.frames_seen > config.benchmark_warmup_frames {
            samples.first_process_memory_gib.get_or_insert(memory);
            samples.last_process_memory_gib = Some(memory);
        }
    }
    if samples.frames_seen > config.benchmark_warmup_frames {
        let milliseconds = time.delta_secs_f64() * 1000.0;
        if milliseconds.is_finite() && milliseconds > 0.0 {
            samples.frame_ms.push(milliseconds);
        }
    }
    let frame_limit_reached = config.benchmark_frames.is_some_and(|limit| {
        samples.frames_seen >= limit.saturating_add(config.benchmark_warmup_frames)
    });
    let duration_reached = config
        .benchmark_duration_secs
        .is_some_and(|limit| samples.frame_ms.iter().sum::<f64>() / 1000.0 >= limit);
    if !frame_limit_reached && !duration_reached {
        return;
    }
    let mut ordered = samples.frame_ms.clone();
    ordered.sort_by(f64::total_cmp);
    let total_ms = ordered.iter().sum::<f64>();
    let mean = total_ms / ordered.len().max(1) as f64;
    let p50 = percentile(&ordered, 0.50);
    let p95 = percentile(&ordered, 0.95);
    let p99 = percentile(&ordered, 0.99);
    let worst = ordered.last().copied().unwrap_or_default();
    let average_fps = if mean > 0.0 { 1000.0 / mean } else { 0.0 };
    let no_streaming_failures = streaming
        .as_ref()
        .is_none_or(|value| value.failed_cells == 0);
    let memory_growth_gib = samples
        .first_process_memory_gib
        .zip(samples.last_process_memory_gib)
        .map(|(first, last)| (last - first).max(0.0));
    let passed = average_fps >= config.accept_min_fps
        && p95 <= config.accept_p95_ms
        && memory_growth_gib.is_none_or(|growth| growth <= config.accept_max_memory_growth_gib)
        && no_streaming_failures;
    let system_snapshot = system.map(|value| SystemSnapshot {
        os: value.os.clone(),
        kernel: value.kernel.clone(),
        cpu: value.cpu.clone(),
        core_count: value.core_count.clone(),
        memory: value.memory.clone(),
    });
    let report = BenchmarkReport {
        format_version: 1,
        generated_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis()),
        scenario: if config.profile_scenario.is_empty() {
            if config.benchmark_only {
                "synthetic".to_owned()
            } else {
                "world".to_owned()
            }
        } else {
            config.profile_scenario.clone()
        },
        frames: ordered.len(),
        warmup_frames: config.benchmark_warmup_frames,
        synthetic_instances: if config.benchmark_only {
            config.synthetic_instances
        } else {
            0
        },
        elapsed_seconds: total_ms / 1000.0,
        average_fps,
        frame_ms_mean: mean,
        frame_ms_p50: p50,
        frame_ms_p95: p95,
        frame_ms_p99: p99,
        frame_ms_worst: worst,
        peak_process_memory_gib: (samples.peak_process_memory_gib > 0.0)
            .then_some(samples.peak_process_memory_gib),
        process_memory_growth_gib: memory_growth_gib,
        entity_count: diagnostic_value(&diagnostics, &EntityCountDiagnosticsPlugin::ENTITY_COUNT)
            .map(|value| value as u64),
        system: system_snapshot.clone(),
        streaming: streaming.as_ref().map(|value| (*value).clone()),
        thresholds: Thresholds {
            minimum_average_fps: config.accept_min_fps,
            maximum_p95_frame_ms: config.accept_p95_ms,
            maximum_memory_growth_gib: config.accept_max_memory_growth_gib,
            no_streaming_failures,
        },
        passed,
    };
    if let Some(parent) = config
        .benchmark_output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        && let Err(error) = fs::create_dir_all(parent)
    {
        error!(%error, "failed to create benchmark report directory");
        exit.write(AppExit::error());
        samples.finished = true;
        return;
    }
    let frame_metrics = match serde_json::to_value(&report) {
        Ok(value) => value,
        Err(error) => {
            error!(%error, "failed to serialize benchmark report");
            exit.write(AppExit::error());
            samples.finished = true;
            return;
        }
    };
    let bundle_system = system_snapshot.map(|value| SystemMetadata {
        os: value.os,
        kernel: value.kernel,
        cpu: value.cpu,
        core_count: value.core_count,
        memory: value.memory,
    });
    if let Err(error) =
        profiler.write_bundle(&config, &frame_metrics, streaming.as_deref(), bundle_system)
    {
        error!(%error, "failed to write profiling bundle");
        exit.write(AppExit::error());
        samples.finished = true;
        return;
    }
    match serde_json::to_vec_pretty(&report)
        .map_err(std::io::Error::other)
        .and_then(|json| fs::write(&config.benchmark_output, json))
    {
        Ok(()) => info!(
            path = %config.benchmark_output.display(),
            average_fps,
            p95_ms = p95,
            passed,
            "acceptance benchmark complete"
        ),
        Err(error) => error!(%error, "failed to write benchmark report"),
    }
    samples.finished = true;
    exit.write(if passed {
        AppExit::Success
    } else {
        AppExit::error()
    });
}

fn diagnostic_value(
    store: &DiagnosticsStore,
    path: &bevy::diagnostic::DiagnosticPath,
) -> Option<f64> {
    store.get(path).and_then(|diagnostic| diagnostic.value())
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) as f64 * percentile).ceil() as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_nearest_rank_percentiles() {
        let samples: Vec<_> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&samples, 0.50), 51.0);
        assert_eq!(percentile(&samples, 0.95), 96.0);
        assert_eq!(percentile(&samples, 0.99), 100.0);
    }
}
