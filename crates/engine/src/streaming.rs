use crate::{
    config::EngineConfig,
    profiling::ProfilingState,
    render::{
        TerrainExtension, TerrainMaterial, WaterExtension, WaterMaterial, WaterReflectionTexture,
    },
    world::{
        cache::{CellCache, TerrainSnapshot},
        components::{
            CELL_SIZE, CellRef, ExteriorCellGrid, FormId, InstanceBounds, MeshHandle,
            StreamedCellRoot, StreamingCamera, TerrainPatch, WaterSurface, WorldPosition,
            WorldTransform,
        },
        database::{AssetCatalog, CellKey, CellPayload, DatabaseRequest, WorldDatabase},
    },
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub struct StreamingPlugin;

impl Plugin for StreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StreamingWorld>()
            .init_resource::<StreamingMetrics>()
            .add_systems(
                Update,
                (
                    plan_cells,
                    collect_cells,
                    track_asset_readiness,
                    update_render_origin,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct StreamingWorld {
    generation: u64,
    cells: HashMap<CellKey, CellStatus>,
}

#[derive(Resource, Debug, Clone, Default, Serialize)]
pub struct StreamingMetrics {
    pub requests_submitted: u64,
    pub responses_received: u64,
    pub stale_responses: u64,
    pub failed_cells: u64,
    pub unloaded_cells: u64,
    pub resident_cells: usize,
    pub loading_cells: usize,
    pub peak_resident_cells: usize,
    pub peak_loading_cells: usize,
    pub total_query_micros: u64,
    pub max_query_micros: u64,
    pub max_commit_micros: u64,
    pub total_queue_wait_micros: u64,
    pub max_queue_wait_micros: u64,
    pub total_request_micros: u64,
    pub max_request_micros: u64,
    pub total_rows_loaded: u64,
    pub assets_ready: u64,
    pub max_asset_ready_micros: u64,
}

enum CellStatus {
    Loading { generation: u64 },
    Resident { root: Entity },
    Failed,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct RenderOrigin(pub IVec2);

#[allow(clippy::too_many_arguments)]
fn plan_cells(
    mut commands: Commands,
    config: Res<EngineConfig>,
    database: Res<WorldDatabase>,
    origin: Res<RenderOrigin>,
    camera: Query<&Transform, With<StreamingCamera>>,
    mut streaming: ResMut<StreamingWorld>,
    mut metrics: ResMut<StreamingMetrics>,
    mut profiler: ResMut<ProfilingState>,
) {
    let plan_started = Instant::now();
    let Ok(camera) = camera.single() else {
        return;
    };
    let global_x = camera.translation.x + origin.0.x as f32 * CELL_SIZE;
    let global_y = -camera.translation.z + origin.0.y as f32 * CELL_SIZE;
    let center = IVec2::new(
        (global_x / CELL_SIZE).floor() as i32,
        (global_y / CELL_SIZE).floor() as i32,
    );
    let mut wanted = HashSet::new();
    for y in -config.stream_radius..=config.stream_radius {
        for x in -config.stream_radius..=config.stream_radius {
            wanted.insert(CellKey::Exterior {
                worldspace_id: config.worldspace_id,
                grid_x: center.x + x,
                grid_y: center.y + y,
            });
        }
    }
    for key in &wanted {
        if !streaming.cells.contains_key(key) {
            streaming.generation = streaming.generation.wrapping_add(1);
            let generation = streaming.generation;
            if database
                .request(DatabaseRequest::Load {
                    generation,
                    key: *key,
                    queued_at: Instant::now(),
                })
                .is_ok()
            {
                metrics.requests_submitted += 1;
                profiler.increment("streaming/requests", 1);
                profiler.event(format!("{key:?}"), "requested", None);
                streaming
                    .cells
                    .insert(*key, CellStatus::Loading { generation });
            }
        }
    }
    streaming.cells.retain(|key, status| {
        let keep = match *key {
            CellKey::Exterior { grid_x, grid_y, .. } => {
                (grid_x - center.x).abs() <= config.unload_radius
                    && (grid_y - center.y).abs() <= config.unload_radius
            }
            CellKey::Interior(_) => true,
        };
        if !keep {
            metrics.unloaded_cells += 1;
            if let CellStatus::Resident { root } = status {
                commands.entity(*root).despawn();
            }
        }
        keep
    });
    metrics.resident_cells = streaming
        .cells
        .values()
        .filter(|status| matches!(status, CellStatus::Resident { .. }))
        .count();
    metrics.loading_cells = streaming
        .cells
        .values()
        .filter(|status| matches!(status, CellStatus::Loading { .. }))
        .count();
    metrics.peak_resident_cells = metrics.peak_resident_cells.max(metrics.resident_cells);
    metrics.peak_loading_cells = metrics.peak_loading_cells.max(metrics.loading_cells);
    profiler.set_gauge("streaming/resident_cells", metrics.resident_cells as f64);
    profiler.set_gauge("streaming/loading_cells", metrics.loading_cells as f64);
    profiler.record_elapsed("streaming/plan_cells", plan_started);
}

#[allow(clippy::too_many_arguments)]
fn collect_cells(
    mut commands: Commands,
    config: Res<EngineConfig>,
    database: Res<WorldDatabase>,
    cache: Res<CellCache>,
    origin: Res<RenderOrigin>,
    asset_server: Res<AssetServer>,
    catalog: Res<AssetCatalog>,
    reflection: Res<WaterReflectionTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut streaming: ResMut<StreamingWorld>,
    mut metrics: ResMut<StreamingMetrics>,
    mut profiler: ResMut<ProfilingState>,
) {
    for _ in 0..config.max_cell_commits_per_frame {
        let Some(response) = database.try_response() else {
            break;
        };
        metrics.responses_received += 1;
        metrics.total_query_micros = metrics
            .total_query_micros
            .saturating_add(response.query_micros);
        metrics.max_query_micros = metrics.max_query_micros.max(response.query_micros);
        metrics.total_queue_wait_micros = metrics
            .total_queue_wait_micros
            .saturating_add(response.queue_wait_micros);
        metrics.max_queue_wait_micros = metrics
            .max_queue_wait_micros
            .max(response.queue_wait_micros);
        metrics.total_request_micros = metrics
            .total_request_micros
            .saturating_add(response.total_request_micros);
        metrics.max_request_micros = metrics
            .max_request_micros
            .max(response.total_request_micros);
        metrics.total_rows_loaded = metrics
            .total_rows_loaded
            .saturating_add(response.row_count as u64);
        profiler.record_micros("streaming/db_queue_wait", response.queue_wait_micros);
        profiler.record_micros("streaming/db_query", response.query_micros);
        profiler.record_micros("streaming/db_request_total", response.total_request_micros);
        let Some(CellStatus::Loading { generation }) = streaming.cells.get(&response.key) else {
            metrics.stale_responses += 1;
            continue;
        };
        if *generation != response.generation {
            metrics.stale_responses += 1;
            continue;
        }
        let commit_started = std::time::Instant::now();
        match response.result {
            Ok(payload) => {
                let terrain = cache.terrain(payload.cell_id);
                let root = spawn_cell(
                    &mut commands,
                    &asset_server,
                    &catalog,
                    &reflection,
                    &mut meshes,
                    &mut terrain_materials,
                    &mut water_materials,
                    origin.0,
                    payload,
                    terrain,
                    &mut profiler,
                );
                streaming
                    .cells
                    .insert(response.key, CellStatus::Resident { root });
            }
            Err(error) => {
                debug!(?response.key, %error, "cell could not be streamed");
                streaming.cells.insert(response.key, CellStatus::Failed);
                metrics.failed_cells += 1;
                profiler.increment("streaming/failed_cells", 1);
            }
        }
        let commit_micros = commit_started
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX)) as u64;
        metrics.max_commit_micros = metrics.max_commit_micros.max(commit_micros);
        profiler.record_micros("streaming/cell_commit", commit_micros);
        profiler.event(
            format!("{:?}", response.key),
            "committed",
            Some(commit_micros as f64 / 1000.0),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_cell(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &AssetCatalog,
    reflection: &WaterReflectionTexture,
    meshes: &mut Assets<Mesh>,
    terrain_materials: &mut Assets<TerrainMaterial>,
    water_materials: &mut Assets<WaterMaterial>,
    origin: IVec2,
    payload: CellPayload,
    terrain: Option<TerrainSnapshot>,
    profiler: &mut ProfilingState,
) -> Entity {
    let spawn_started = Instant::now();
    let reference_count = payload.references.len();
    let root_translation = cell_translation(payload.key, origin);
    let mut root_commands = commands.spawn((
        Name::new(format!("Cell {:08X}", payload.cell_id)),
        CellRef(payload.cell_id),
        StreamedCellRoot,
        Transform::from_translation(root_translation),
        Visibility::default(),
    ));
    if let CellKey::Exterior { grid_x, grid_y, .. } = payload.key {
        root_commands.insert(ExteriorCellGrid(IVec2::new(grid_x, grid_y)));
    }
    let root = root_commands.id();
    commands.entity(root).with_children(|parent| {
        if let Some(terrain) = terrain
            && let Some(mesh) = {
                let started = Instant::now();
                let mesh = build_terrain_mesh(&terrain);
                profiler.record_elapsed("streaming/terrain_mesh", started);
                mesh
            }
        {
            let material = terrain_materials.add(TerrainMaterial {
                base: StandardMaterial {
                    base_color: Color::srgb(0.28, 0.38, 0.18),
                    perceptual_roughness: 0.92,
                    cull_mode: None,
                    double_sided: true,
                    ..default()
                },
                extension: TerrainExtension::from_terrain(&terrain, catalog, asset_server),
            });
            parent.spawn((
                Name::new("Terrain"),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::default(),
                TerrainPatch,
            ));
            if let Some(height) = terrain
                .water_height
                .filter(|height| height.is_finite() && height.abs() < 1.0e7)
            {
                let water_mesh = meshes.add(Plane3d::default().mesh().size(CELL_SIZE, CELL_SIZE));
                let water_material = water_materials.add(WaterMaterial {
                    base: StandardMaterial {
                        base_color: Color::srgba(0.05, 0.2, 0.32, 0.68),
                        metallic: 0.15,
                        perceptual_roughness: 0.06,
                        reflectance: 0.9,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    },
                    extension: WaterExtension::with_reflection(
                        reflection.0.clone(),
                        terrain
                            .water_type_form_id
                            .and_then(|form_id| catalog.water_flow(form_id))
                            .map(|path| asset_server.load(path.to_owned())),
                    ),
                });
                parent.spawn((
                    Name::new("Water"),
                    Mesh3d(water_mesh),
                    MeshMaterial3d(water_material),
                    Transform::from_translation(Vec3::new(
                        CELL_SIZE * 0.5,
                        height,
                        -CELL_SIZE * 0.5,
                    )),
                    WaterSurface,
                    bevy::camera::visibility::RenderLayers::layer(1),
                ));
            }
        }
        for reference in payload.references {
            let creation_position = Vec3::from_array(reference.position);
            let world_position = WorldPosition::from_creation_units(creation_position);
            let translation = match payload.key {
                CellKey::Exterior { grid_x, grid_y, .. } => {
                    let cell_origin = IVec2::new(grid_x, grid_y);
                    creation_to_bevy(world_position.relative_to(cell_origin))
                }
                CellKey::Interior(_) => creation_to_bevy(creation_position),
            };
            let rotation = Quat::from_euler(
                EulerRot::ZYX,
                reference.rotation[2],
                reference.rotation[1],
                reference.rotation[0],
            );
            let transform = Transform::from_translation(translation)
                .with_rotation(rotation)
                .with_scale(Vec3::splat(reference.scale));
            let bounds = reference.bounds_valid.then(|| {
                InstanceBounds::transformed(
                    Vec3::from_array(reference.bounds_min),
                    Vec3::from_array(reference.bounds_max),
                    transform.to_matrix(),
                )
            });
            let mut entity = parent.spawn((
                Name::new(format!("Reference {:08X}", reference.form_id)),
                FormId(reference.form_id),
                CellRef(reference.cell_id),
                world_position,
                WorldTransform(transform.to_matrix()),
                transform,
            ));
            if let Some(bounds) = bounds {
                entity.insert(bounds);
            }
            if let Some(path) = reference.model_path.and_then(converted_model_path) {
                entity.insert((
                    MeshHandle(path.clone()),
                    WorldAssetRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(path.clone())),
                    ),
                    PendingAssetProfile {
                        started: Instant::now(),
                        path,
                    },
                ));
            }
        }
    });
    profiler.increment("streaming/references_spawned", reference_count as u64);
    profiler.record_elapsed("streaming/spawn_cell", spawn_started);
    root
}

#[derive(Component)]
struct PendingAssetProfile {
    started: Instant,
    path: String,
}

fn track_asset_readiness(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pending: Query<(Entity, &WorldAssetRoot, &PendingAssetProfile)>,
    mut metrics: ResMut<StreamingMetrics>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = Instant::now();
    for (entity, root, pending) in &pending {
        if asset_server.is_loaded_with_dependencies(root.0.id()) {
            let micros = pending
                .started
                .elapsed()
                .as_micros()
                .min(u128::from(u64::MAX)) as u64;
            metrics.assets_ready = metrics.assets_ready.saturating_add(1);
            metrics.max_asset_ready_micros = metrics.max_asset_ready_micros.max(micros);
            profiler.record_micros("assets/model_ready", micros);
            profiler.event(&pending.path, "asset_ready", Some(micros as f64 / 1000.0));
            commands.entity(entity).remove::<PendingAssetProfile>();
        }
    }
    profiler.record_elapsed("assets/readiness_scan", started);
}

fn cell_translation(key: CellKey, origin: IVec2) -> Vec3 {
    match key {
        CellKey::Exterior { grid_x, grid_y, .. } => Vec3::new(
            (grid_x - origin.x) as f32 * CELL_SIZE,
            0.0,
            -(grid_y - origin.y) as f32 * CELL_SIZE,
        ),
        CellKey::Interior(_) => Vec3::ZERO,
    }
}

fn creation_to_bevy(position: Vec3) -> Vec3 {
    Vec3::new(position.x, position.z, -position.y)
}

fn converted_model_path(path: String) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let without_prefix = normalized
        .strip_prefix("meshes/")
        .or_else(|| normalized.strip_prefix("Meshes/"))
        .unwrap_or(&normalized);
    if without_prefix.is_empty() {
        return None;
    }
    let mut converted = std::path::PathBuf::from("meshes").join(without_prefix);
    converted.set_extension("glb");
    Some(converted.to_string_lossy().replace('\\', "/"))
}

fn build_terrain_mesh(terrain: &TerrainSnapshot) -> Option<Mesh> {
    let width = usize::from(terrain.width);
    let height = usize::from(terrain.height);
    if width < 2 || height < 2 || terrain.heights.len() != width * height {
        return None;
    }
    let step_x = CELL_SIZE / (width - 1) as f32;
    let step_z = CELL_SIZE / (height - 1) as f32;
    let mut positions = Vec::with_capacity(width * height);
    let mut normals = Vec::with_capacity(width * height);
    let mut uvs = Vec::with_capacity(width * height);
    let mut colors = vec![[0.0, 0.0, 0.0, 1.0]; width * height];
    let mut extra_weights = vec![[0.0, 0.0]; width * height];
    let mut texture_ids = Vec::with_capacity(6);
    for layer in &terrain.layers {
        if !texture_ids.contains(&layer.texture_form_id) {
            texture_ids.push(layer.texture_form_id);
        }
        if texture_ids.len() == 6 {
            break;
        }
    }
    for layer in &terrain.layers {
        let Some(layer_index) = texture_ids
            .iter()
            .position(|texture_id| *texture_id == layer.texture_form_id)
        else {
            continue;
        };
        if layer_index == 0 {
            continue;
        }
        let quadrant_x = usize::from(layer.quadrant % 2) * 16;
        let quadrant_y = usize::from(layer.quadrant / 2) * 16;
        for &(vertex, opacity) in &layer.weights {
            let local = usize::from(vertex);
            let x = quadrant_x + local % 17;
            let y = quadrant_y + local / 17;
            if x < width && y < height {
                let opacity = opacity.clamp(0.0, 1.0);
                match layer_index {
                    1..=3 => colors[y * width + x][layer_index - 1] = opacity,
                    4..=5 => extra_weights[y * width + x][layer_index - 4] = opacity,
                    _ => {}
                }
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;
            positions.push([
                x as f32 * step_x,
                terrain.heights[index],
                -(y as f32 * step_z),
            ]);
            if terrain.normals.len() >= (index + 1) * 3 {
                let normal = Vec3::new(
                    terrain.normals[index * 3] as f32,
                    terrain.normals[index * 3 + 2] as f32,
                    -(terrain.normals[index * 3 + 1] as f32),
                )
                .normalize_or(Vec3::Y);
                normals.push(normal.to_array());
            } else {
                normals.push(Vec3::Y.to_array());
            }
            uvs.push([
                x as f32 / (width - 1) as f32,
                y as f32 / (height - 1) as f32,
            ]);
        }
    }
    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let a = (y * width + x) as u32;
            let b = a + 1;
            let c = a + width as u32;
            let d = c + 1;
            // Z decreases as LAND rows advance, so clockwise index order in
            // X/Z space is the upward-facing order in Bevy's Y-up space.
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, extra_weights);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    Some(mesh)
}

fn update_render_origin(
    mut origin: ResMut<RenderOrigin>,
    mut camera: Query<&mut Transform, With<StreamingCamera>>,
    mut roots: Query<(&ExteriorCellGrid, &mut Transform), Without<StreamingCamera>>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = Instant::now();
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    let shift = IVec2::new(
        (camera.translation.x / CELL_SIZE).trunc() as i32,
        (-camera.translation.z / CELL_SIZE).trunc() as i32,
    );
    if shift == IVec2::ZERO {
        return;
    }
    origin.0 += shift;
    camera.translation.x -= shift.x as f32 * CELL_SIZE;
    camera.translation.z += shift.y as f32 * CELL_SIZE;
    for (grid, mut transform) in &mut roots {
        transform.translation = Vec3::new(
            (grid.0.x - origin.0.x) as f32 * CELL_SIZE,
            0.0,
            -(grid.0.y - origin.0.y) as f32 * CELL_SIZE,
        );
    }
    profiler.increment("streaming/origin_rebases", 1);
    profiler.record_elapsed("streaming/render_origin_rebase", started);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_nif_paths_to_converted_glb_paths() {
        assert_eq!(
            converted_model_path("meshes\\architecture\\wall.nif".into()).as_deref(),
            Some("meshes/architecture/wall.glb")
        );
    }

    #[test]
    fn creates_expected_terrain_triangles() {
        let terrain = TerrainSnapshot {
            cell_id: 1,
            width: 2,
            height: 2,
            heights: vec![0.0; 4],
            normals: vec![],
            vertex_colors: vec![],
            layers: vec![],
            water_height: None,
            water_type_form_id: None,
        };
        let mesh = build_terrain_mesh(&terrain).unwrap();
        assert_eq!(mesh.count_vertices(), 4);
        assert_eq!(
            mesh.indices().unwrap(),
            &Indices::U32(vec![0, 1, 2, 1, 3, 2])
        );
    }
}
