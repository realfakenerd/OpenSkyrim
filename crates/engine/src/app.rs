use crate::{
    config::EngineConfig,
    metrics::AcceptanceMetricsPlugin,
    profiling::{ProfilingPlugin, ProfilingState},
    render::{
        TerrainExtension, TerrainMaterial, VercidiumRendererPlugin, WaterExtension, WaterMaterial,
    },
    streaming::{RenderOrigin, StreamingPlugin},
    world::{
        cache::CellCache,
        components::StreamingCamera,
        database::{AssetCatalog, WorldDatabase},
    },
};
use bevy::{
    asset::AssetPlugin,
    camera::visibility::RenderLayers,
    core_pipeline::prepass::DepthPrepass,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::{PresentMode, WindowPlugin},
};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use serde::Deserialize;
use turso::params;

#[derive(Resource)]
struct InitialCameraGroundHeight(f32);

pub fn run(config: EngineConfig) -> Result<()> {
    let runtime_data = if config.benchmark_only {
        None
    } else {
        validate_runtime_assets(&config)?;
        let database_path = config.assets_dir.join("skyrim_world.db");
        let cache = CellCache::open(&config.assets_dir.join("cell_cache.rkyv"))?;
        let ground_height = initial_camera_ground_height(&config, &database_path, &cache)?;
        Some((
            WorldDatabase::open(&database_path)?,
            AssetCatalog::open(&database_path)?,
            cache,
            InitialCameraGroundHeight(ground_height),
        ))
    };
    let asset_path = config.assets_dir.to_string_lossy().into_owned();
    let window = (!config.headless).then(|| Window {
        title: "OpenSkyrim".into(),
        resolution: (1600, 900).into(),
        present_mode: if config.benchmark_frames.is_some()
            || config.benchmark_duration_secs.is_some()
        {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        },
        ..default()
    });
    let origin = RenderOrigin(IVec2::new(config.start_grid.0, config.start_grid.1));
    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(origin)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: window,
                    ..default()
                }),
        )
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            AcceptanceMetricsPlugin,
            ProfilingPlugin,
            RenderDiagnosticsPlugin,
        ))
        .add_plugins(VercidiumRendererPlugin)
        .add_systems(Startup, setup_world)
        .add_systems(Update, (fly_camera, capture_acceptance_screenshot));
    if let Some((database, catalog, cache, ground_height)) = runtime_data {
        app.insert_resource(database)
            .insert_resource(catalog)
            .insert_resource(cache)
            .insert_resource(ground_height)
            .add_plugins(StreamingPlugin);
    } else {
        app.add_systems(Startup, setup_synthetic_benchmark);
    }
    app.run();
    Ok(())
}

#[derive(Deserialize)]
struct RuntimeManifest {
    schema_version: u32,
    complete: bool,
}

#[derive(Deserialize)]
struct RuntimeIntegrationReport {
    schema_version: u32,
    passed: bool,
}

fn validate_runtime_assets(config: &EngineConfig) -> Result<()> {
    for required in ["skyrim_world.db", "cell_cache.rkyv"] {
        color_eyre::eyre::ensure!(
            config.assets_dir.join(required).is_file(),
            "converted asset set is missing {required}: {}",
            config.assets_dir.display()
        );
    }
    if config.allow_incomplete_assets {
        return Ok(());
    }
    let manifest_path = config.assets_dir.join("conversion-manifest.json");
    let manifest: RuntimeManifest = serde_json::from_slice(
        &std::fs::read(&manifest_path)
            .wrap_err_with(|| format!("failed to read {}", manifest_path.display()))?,
    )
    .wrap_err("invalid conversion manifest")?;
    color_eyre::eyre::ensure!(
        manifest.schema_version == converter_schema_version() && manifest.complete,
        "asset conversion is incomplete or stale; reconvert assets with converter schema {}",
        converter_schema_version()
    );
    let report_path = config.assets_dir.join("integration-report.json");
    let report: RuntimeIntegrationReport = serde_json::from_slice(
        &std::fs::read(&report_path)
            .wrap_err_with(|| format!("failed to read {}", report_path.display()))?,
    )
    .wrap_err("invalid integration report")?;
    color_eyre::eyre::ensure!(
        report.schema_version == shared::WORLD_DATABASE_SCHEMA_VERSION && report.passed,
        "asset integration report did not pass; inspect {}",
        report_path.display()
    );
    Ok(())
}

const fn converter_schema_version() -> u32 {
    // Kept in sync with converter::cache::CONVERTER_SCHEMA_VERSION without
    // linking the heavy converter crate into the runtime binary.
    5
}

fn setup_synthetic_benchmark(
    mut commands: Commands,
    config: Res<EngineConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = std::time::Instant::now();
    let mesh = Mesh3d(meshes.add(Cuboid::new(18.0, 60.0, 18.0)));
    let material = MeshMaterial3d(terrain_materials.add(TerrainMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.16, 0.36, 0.12),
            perceptual_roughness: 0.9,
            ..default()
        },
        extension: TerrainExtension::default(),
    }));
    let side = (config.synthetic_instances as f64).sqrt().ceil() as usize;
    commands.spawn_batch((0..config.synthetic_instances).map(move |index| {
        let x = index % side;
        let z = index / side;
        (
            mesh.clone(),
            material.clone(),
            Transform::from_xyz(x as f32 * 32.0, 30.0, -(z as f32 * 32.0)),
        )
    }));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1024.0, 1024.0))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base: StandardMaterial {
                base_color: Color::srgba(0.04, 0.18, 0.3, 0.7),
                metallic: 0.1,
                perceptual_roughness: 0.08,
                alpha_mode: AlphaMode::Blend,
                ..default()
            },
            extension: WaterExtension::default(),
        })),
        Transform::from_xyz(CELL_SIZE_HALF, 8.0, -CELL_SIZE_HALF),
        crate::world::components::WaterSurface,
        RenderLayers::layer(1),
    ));
    info!(
        instances = config.synthetic_instances,
        "synthetic indirect-render benchmark initialized"
    );
    profiler.increment("synthetic/instances", config.synthetic_instances as u64);
    profiler.record_elapsed("startup/synthetic_scene", started);
}

fn setup_world(
    mut commands: Commands,
    config: Res<EngineConfig>,
    ground_height: Option<Res<InitialCameraGroundHeight>>,
) {
    let ground_height = ground_height.as_deref().map_or(0.0, |height| height.0);
    let target = Vec3::new(CELL_SIZE_HALF, ground_height, -CELL_SIZE_HALF);
    let camera_position = target + Vec3::new(0.0, 1200.0, 2500.0);
    let far = crate::world::components::CELL_SIZE * (config.stream_radius.max(1) + 2) as f32 * 2.0;
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { far, ..default() }),
        Transform::from_translation(camera_position).looking_at(target, Vec3::Y),
        StreamingCamera,
        Msaa::Off,
        DepthPrepass,
        RenderLayers::from_layers(&[0, 1]),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.48, 0.55, 0.7),
        brightness: 160.0,
        ..default()
    });
    info!(
        assets = %config.assets_dir.display(),
        worldspace = format_args!("{:08X}", config.worldspace_id),
        ground_height,
        camera = ?camera_position,
        target = ?target,
        "OpenSkyrim runtime initialized"
    );
}

fn initial_camera_ground_height(
    config: &EngineConfig,
    database_path: &std::path::Path,
    cache: &CellCache,
) -> Result<f32> {
    let cell_id = tokio::runtime::Runtime::new()?.block_on(async {
        let connection = turso::Builder::new_local(&database_path.to_string_lossy())
            .build()
            .await?
            .connect()
            .wrap_err_with(|| format!("failed to open {}", database_path.display()))?;
        let row = connection
            .query(
                "SELECT id FROM cells WHERE worldspace_id=?1 AND grid_x=?2 AND grid_y=?3",
                params![
                    config.worldspace_id,
                    config.start_grid.0,
                    config.start_grid.1
                ],
            )
            .await?
            .next()
            .await?;
        Ok::<Option<u32>, color_eyre::Report>(match row {
            Some(row) => Some(row.get::<u32>(0)?),
            None => None,
        })
    })?;
    let Some(terrain) = cell_id.and_then(|cell_id| cache.terrain(cell_id)) else {
        return Ok(0.0);
    };
    let width = usize::from(terrain.width);
    let height = usize::from(terrain.height);
    let center = (height / 2)
        .checked_mul(width)
        .and_then(|row| row.checked_add(width / 2));
    Ok(center
        .and_then(|index| terrain.heights.get(index))
        .copied()
        .unwrap_or(0.0))
}

const CELL_SIZE_HALF: f32 = crate::world::components::CELL_SIZE * 0.5;

fn fly_camera(
    time: Res<Time>,
    config: Res<EngineConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<StreamingCamera>>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = std::time::Instant::now();
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }
    if keyboard.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }
    if config.auto_fly_speed > 0.0 {
        direction += *transform.forward();
    }
    let speed = if config.auto_fly_speed > 0.0 {
        config.auto_fly_speed
    } else if keyboard.pressed(KeyCode::ControlLeft) {
        4000.0
    } else {
        900.0
    };
    transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
    profiler.record_elapsed("world/fly_camera", started);
}

fn capture_acceptance_screenshot(
    mut commands: Commands,
    config: Res<EngineConfig>,
    mut frames: Local<u32>,
    windows: Query<(), With<Window>>,
) {
    let Some(path) = &config.acceptance_screenshot else {
        return;
    };
    *frames = frames.saturating_add(1);
    if *frames != config.benchmark_warmup_frames.saturating_add(10) || windows.is_empty() {
        return;
    }
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        error!(%error, path = %path.display(), "failed to create screenshot directory");
        return;
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_stale_or_incomplete_runtime_assets() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("skyrim_world.db"), []).unwrap();
        std::fs::write(directory.path().join("cell_cache.rkyv"), []).unwrap();
        std::fs::write(
            directory.path().join("conversion-manifest.json"),
            br#"{"schema_version":3,"complete":true}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("integration-report.json"),
            br#"{"schema_version":3,"passed":true}"#,
        )
        .unwrap();
        let config = EngineConfig {
            assets_dir: directory.path().to_owned(),
            ..default()
        };
        assert!(validate_runtime_assets(&config).is_err());
    }

    #[test]
    fn accepts_current_complete_runtime_assets() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("skyrim_world.db"), []).unwrap();
        std::fs::write(directory.path().join("cell_cache.rkyv"), []).unwrap();
        std::fs::write(
            directory.path().join("conversion-manifest.json"),
            br#"{"schema_version":4,"complete":true}"#,
        )
        .unwrap();
        std::fs::write(
            directory.path().join("integration-report.json"),
            br#"{"schema_version":3,"passed":true}"#,
        )
        .unwrap();
        let config = EngineConfig {
            assets_dir: directory.path().to_owned(),
            ..default()
        };
        validate_runtime_assets(&config).unwrap();
    }
}
