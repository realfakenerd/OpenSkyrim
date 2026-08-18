use crate::{
    profiling::ProfilingState,
    world::{cache::TerrainSnapshot, database::AssetCatalog},
};
use bevy::{
    asset::embedded_asset,
    camera::{RenderTarget, visibility::RenderLayers},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainExtension>;
pub type WaterMaterial = ExtendedMaterial<StandardMaterial, WaterExtension>;

pub struct VercidiumRendererPlugin;

impl Plugin for VercidiumRendererPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/terrain.wgsl");
        embedded_asset!(app, "shaders/water.wgsl");
        app.add_plugins((
            MaterialPlugin::<TerrainMaterial>::default(),
            MaterialPlugin::<WaterMaterial>::default(),
        ))
        .add_systems(PostStartup, setup_water_reflection)
        .add_systems(
            Update,
            (animate_water_materials, update_water_reflection_camera),
        );
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TerrainExtension {
    #[texture(100)]
    #[sampler(101)]
    layer_0: Option<Handle<Image>>,
    #[texture(102)]
    #[sampler(103)]
    layer_1: Option<Handle<Image>>,
    #[texture(104)]
    #[sampler(105)]
    layer_2: Option<Handle<Image>>,
    #[texture(106)]
    #[sampler(107)]
    layer_3: Option<Handle<Image>>,
    #[texture(108)]
    #[sampler(109)]
    layer_4: Option<Handle<Image>>,
    #[texture(110)]
    #[sampler(111)]
    layer_5: Option<Handle<Image>>,
    #[uniform(112)]
    settings: TerrainSettings,
}

#[derive(ShaderType, Reflect, Debug, Clone)]
struct TerrainSettings {
    tiling_and_layer_count: Vec4,
    fallback_weights_0: Vec4,
    fallback_weights_1: Vec4,
}

impl TerrainExtension {
    pub fn from_terrain(
        terrain: &TerrainSnapshot,
        catalog: &AssetCatalog,
        asset_server: &AssetServer,
    ) -> Self {
        let mut textures: [Option<Handle<Image>>; 6] = std::array::from_fn(|_| None);
        let mut texture_ids = Vec::with_capacity(6);
        for layer in &terrain.layers {
            if !texture_ids.contains(&layer.texture_form_id) {
                texture_ids.push(layer.texture_form_id);
            }
            if texture_ids.len() == 6 {
                break;
            }
        }
        for (target, texture_id) in textures.iter_mut().zip(texture_ids.iter()) {
            *target = catalog
                .landscape_diffuse(*texture_id)
                .map(|path| asset_server.load(path.to_owned()));
        }
        let layer_count = texture_ids.len() as f32;
        Self {
            layer_0: textures[0].clone(),
            layer_1: textures[1].clone(),
            layer_2: textures[2].clone(),
            layer_3: textures[3].clone(),
            layer_4: textures[4].clone(),
            layer_5: textures[5].clone(),
            settings: TerrainSettings {
                tiling_and_layer_count: Vec4::new(8.0, 8.0, layer_count, 0.0),
                fallback_weights_0: Vec4::new(1.0, 0.0, 0.0, 0.0),
                fallback_weights_1: Vec4::ZERO,
            },
        }
    }
}

impl Default for TerrainExtension {
    fn default() -> Self {
        Self {
            layer_0: None,
            layer_1: None,
            layer_2: None,
            layer_3: None,
            layer_4: None,
            layer_5: None,
            settings: TerrainSettings {
                tiling_and_layer_count: Vec4::new(8.0, 8.0, 0.0, 0.0),
                fallback_weights_0: Vec4::X,
                fallback_weights_1: Vec4::ZERO,
            },
        }
    }
}

impl MaterialExtension for TerrainExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://engine/shaders/terrain.wgsl".into()
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct WaterExtension {
    #[uniform(100)]
    settings: WaterSettings,
    #[texture(101)]
    #[sampler(102)]
    reflection: Option<Handle<Image>>,
    #[texture(103)]
    #[sampler(104)]
    flow_normal: Option<Handle<Image>>,
}

#[derive(ShaderType, Reflect, Debug, Clone)]
struct WaterSettings {
    wave_scale_speed_strength: Vec4,
    flow_direction: Vec4,
}

impl Default for WaterExtension {
    fn default() -> Self {
        Self {
            settings: WaterSettings {
                wave_scale_speed_strength: Vec4::new(0.006, 0.15, 0.32, 0.0),
                flow_direction: Vec4::new(0.8, 0.35, 0.0, 0.0),
            },
            reflection: None,
            flow_normal: None,
        }
    }
}

impl WaterExtension {
    pub fn with_reflection(reflection: Handle<Image>, flow_normal: Option<Handle<Image>>) -> Self {
        let has_flow_normal = flow_normal.is_some() as u8 as f32;
        Self {
            reflection: Some(reflection),
            flow_normal,
            settings: WaterSettings {
                wave_scale_speed_strength: Vec4::new(0.006, 0.15, 0.32, 0.0),
                flow_direction: Vec4::new(0.8, 0.35, 0.0, has_flow_normal),
            },
        }
    }
}

impl MaterialExtension for WaterExtension {
    fn fragment_shader() -> ShaderRef {
        "embedded://engine/shaders/water.wgsl".into()
    }
}

fn animate_water_materials(
    time: Res<Time>,
    mut materials: ResMut<Assets<WaterMaterial>>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = std::time::Instant::now();
    let elapsed = time.elapsed_secs();
    for (_, material) in materials.iter_mut() {
        material.extension.settings.wave_scale_speed_strength.w = elapsed;
    }
    profiler.record_elapsed("render/water_animation", started);
}

#[derive(Resource, Clone)]
pub struct WaterReflectionTexture(pub Handle<Image>);

#[derive(Component)]
struct WaterReflectionCamera;

fn setup_water_reflection(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_target_texture(
        1024,
        1024,
        bevy::render::render_resource::TextureFormat::Rgba8Unorm,
        Some(bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb),
    ));
    commands.insert_resource(WaterReflectionTexture(image.clone()));
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            invert_culling: true,
            is_active: false,
            ..default()
        },
        RenderTarget::Image(image.into()),
        Transform::default(),
        RenderLayers::layer(0),
        WaterReflectionCamera,
    ));
}

fn update_water_reflection_camera(
    main_camera: Query<
        &GlobalTransform,
        (
            With<crate::world::components::StreamingCamera>,
            Without<WaterReflectionCamera>,
        ),
    >,
    water: Query<&GlobalTransform, With<crate::world::components::WaterSurface>>,
    mut reflection_camera: Query<(&mut Transform, &mut Camera), With<WaterReflectionCamera>>,
    mut profiler: ResMut<ProfilingState>,
) {
    let started = std::time::Instant::now();
    let (Ok(main), Ok((mut reflection, mut camera))) =
        (main_camera.single(), reflection_camera.single_mut())
    else {
        return;
    };
    let Some(surface) = water.iter().min_by(|left, right| {
        let left_distance = (left.translation().y - main.translation().y).abs();
        let right_distance = (right.translation().y - main.translation().y).abs();
        left_distance.total_cmp(&right_distance)
    }) else {
        camera.is_active = false;
        return;
    };
    let water_y = surface.translation().y;
    let mut position = main.translation();
    position.y = water_y * 2.0 - position.y;
    let mut forward = main.forward().as_vec3();
    forward.y = -forward.y;
    *reflection = Transform::from_translation(position).looking_to(forward, Vec3::Y);
    camera.is_active = true;
    profiler.record_elapsed("render/water_reflection_camera", started);
}
