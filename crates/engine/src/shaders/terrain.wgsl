#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
}

struct TerrainSettings {
    tiling_and_layer_count: vec4<f32>,
    fallback_weights_0: vec4<f32>,
    fallback_weights_1: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var layer_0: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var layer_0_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var layer_1: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var layer_1_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var layer_2: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var layer_2_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var layer_3: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(107) var layer_3_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(108) var layer_4: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(109) var layer_4_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var layer_5: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(111) var layer_5_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(112) var<uniform> terrain: TerrainSettings;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    var weights = array<f32, 6>(
        terrain.fallback_weights_0.x,
        terrain.fallback_weights_0.y,
        terrain.fallback_weights_0.z,
        terrain.fallback_weights_0.w,
        terrain.fallback_weights_1.x,
        terrain.fallback_weights_1.y,
    );
#ifdef VERTEX_COLORS
    weights[0] = max(0.0, 1.0 - in.color.r - in.color.g - in.color.b);
    weights[1] = in.color.r;
    weights[2] = in.color.g;
    weights[3] = in.color.b;
#endif
#ifdef VERTEX_UVS_B
    weights[4] = in.uv_b.x;
    weights[5] = in.uv_b.y;
#endif
    weights[0] = max(0.0, 1.0 - weights[1] - weights[2] - weights[3] - weights[4] - weights[5]);
    let total = max(0.0001, weights[0] + weights[1] + weights[2] + weights[3] + weights[4] + weights[5]);
    let uv = in.uv * terrain.tiling_and_layer_count.xy;
    var color = textureSample(layer_0, layer_0_sampler, uv) * weights[0];
    color += textureSample(layer_1, layer_1_sampler, uv) * weights[1];
    color += textureSample(layer_2, layer_2_sampler, uv) * weights[2];
    color += textureSample(layer_3, layer_3_sampler, uv) * weights[3];
    color += textureSample(layer_4, layer_4_sampler, uv) * weights[4];
    color += textureSample(layer_5, layer_5_sampler, uv) * weights[5];
    pbr_input.material.base_color *= color / total;
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
