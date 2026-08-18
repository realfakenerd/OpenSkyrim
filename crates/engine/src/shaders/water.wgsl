#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
}

struct WaterSettings {
    wave_scale_speed_strength: vec4<f32>,
    flow_direction: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> water: WaterSettings;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var water_reflection: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var water_reflection_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var water_flow_normal: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var water_flow_normal_sampler: sampler;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let scale = water.wave_scale_speed_strength.x;
    let phase = water.wave_scale_speed_strength.w * water.wave_scale_speed_strength.y;
    let strength = water.wave_scale_speed_strength.z;
    let p = in.world_position.xz * scale + water.flow_direction.xy * phase;
    var dx = cos(p.x + sin(p.y * 1.7)) * strength;
    var dz = cos(p.y * 1.3 + sin(p.x)) * strength;
    if water.flow_direction.w > 0.5 {
        let flow = textureSample(water_flow_normal, water_flow_normal_sampler, fract(p * 0.25)).xy * 2.0 - 1.0;
        dx += flow.x * strength;
        dz += flow.y * strength;
    }
    pbr_input.N = normalize(vec3<f32>(-dx, 1.0, -dz));
    let reflection_size = vec2<f32>(textureDimensions(water_reflection));
    let reflection_uv = vec2<f32>(in.position.x / reflection_size.x, 1.0 - in.position.y / reflection_size.y);
    let reflection_color = textureSample(water_reflection, water_reflection_sampler, reflection_uv);
    let fresnel = pow(1.0 - clamp(dot(normalize(pbr_input.V), pbr_input.N), 0.0, 1.0), 5.0);
    pbr_input.material.base_color = mix(pbr_input.material.base_color, reflection_color, 0.25 + fresnel * 0.55);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
