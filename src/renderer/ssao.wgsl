// SSAO – Screen Space Ambient Occlusion

@group(0) @binding(0)
var depth_texture: texture_depth_2d;
@group(0) @binding(1)
var depth_sampler: sampler;

struct SSAOParams {
    proj_inv: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,
    sample_radius: f32,
    bias: f32,
    intensity: f32,
    max_dist: f32,
}

@group(0) @binding(2)
var<uniform> params: SSAOParams;

// 32 náhodných směrů v polokouli (poisson disk)
const SAMPLE_COUNT: u32 = 32u;
const SAMPLE_KERNEL: array<vec3<f32>, 32> = array<vec3<f32>, 32>(
    vec3<f32>(0.04977, 0.04297, 0.06461),
    vec3<f32>(-0.01276, -0.02940, 0.05008),
    vec3<f32>(0.07772, 0.07606, 0.09390),
    vec3<f32>(-0.08328, 0.06309, 0.10393),
    vec3<f32>(0.03626, -0.09010, 0.05547),
    vec3<f32>(-0.01357, 0.01890, 0.03853),
    vec3<f32>(0.09587, -0.03516, 0.04045),
    vec3<f32>(-0.03902, -0.07792, 0.04698),
    vec3<f32>(0.01372, 0.08445, 0.05811),
    vec3<f32>(-0.08858, -0.04429, 0.04071),
    vec3<f32>(0.02907, 0.02252, 0.04836),
    vec3<f32>(-0.06259, 0.02745, 0.05131),
    vec3<f32>(0.06829, -0.05995, 0.04798),
    vec3<f32>(-0.03966, 0.06624, 0.05388),
    vec3<f32>(0.01224, -0.07889, 0.04254),
    vec3<f32>(-0.07920, -0.00924, 0.03812),
    vec3<f32>(0.05331, 0.05228, 0.07133),
    vec3<f32>(-0.02664, -0.04844, 0.04125),
    vec3<f32>(0.08483, 0.00451, 0.05055),
    vec3<f32>(-0.05206, 0.08312, 0.05842),
    vec3<f32>(0.02142, -0.04631, 0.03567),
    vec3<f32>(-0.07611, 0.03951, 0.05183),
    vec3<f32>(0.04693, -0.08217, 0.05261),
    vec3<f32>(-0.00120, 0.05607, 0.04429),
    vec3<f32>(0.06082, 0.03041, 0.05672),
    vec3<f32>(-0.03474, -0.02524, 0.03277),
    vec3<f32>(0.03120, 0.06875, 0.06028),
    vec3<f32>(-0.06821, -0.06416, 0.04694),
    vec3<f32>(0.07209, -0.02314, 0.04692),
    vec3<f32>(-0.02139, 0.03789, 0.03922),
    vec3<f32>(0.00779, -0.03563, 0.02895),
    vec3<f32>(-0.05437, -0.01754, 0.03442)
);

// Rotation noise – world-space seed = stabilní při pohybu kamery
fn get_noise_rotation(world_pos: vec3<f32>) -> vec3<f32> {
    let seed = world_pos.x * 7.0 + world_pos.y * 13.0 + world_pos.z * 11.0;
    let angle = (seed - floor(seed)) * 6.283185;
    return vec3<f32>(cos(angle), sin(angle), 0.0);
}

fn get_view_pos_from_depth(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let view_pos = params.proj_inv * ndc;
    return view_pos.xyz / view_pos.w;
}

struct VertexInput {
    @location(0) ndc_xy: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.ndc_xy, 0.0, 1.0);
    let uv_raw = in.ndc_xy * 0.5 + 0.5;
    out.uv = vec2<f32>(uv_raw.x, 1.0 - uv_raw.y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_texture, depth_sampler, in.uv);
    if (depth >= 0.995) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    let tex_size = vec2<f32>(textureDimensions(depth_texture));
    let texel = 1.0 / tex_size;
    let depth_x = textureSample(depth_texture, depth_sampler, in.uv + vec2<f32>(texel.x, 0.0));
    let depth_y = textureSample(depth_texture, depth_sampler, in.uv + vec2<f32>(0.0, texel.y));
    let depth_x_safe = select(depth, depth_x, depth_x < 0.995);
    let depth_y_safe = select(depth, depth_y, depth_y < 0.995);
    let frag_pos = get_view_pos_from_depth(in.uv, depth);
    let dist = length(frag_pos);
    if (dist > params.max_dist) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    let pos_dx = get_view_pos_from_depth(in.uv + vec2<f32>(texel.x, 0.0), depth_x_safe) - frag_pos;
    let pos_dy = get_view_pos_from_depth(in.uv + vec2<f32>(0.0, texel.y), depth_y_safe) - frag_pos;
    let cross_val = cross(pos_dx, pos_dy);
    let cross_len = length(cross_val);
    if (cross_len < 0.0001) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    var normal = normalize(cross_val);
    if (dot(normal, frag_pos) > 0.0) {
        normal = -normal;
    }
    let world_pos = (params.view_inv * vec4<f32>(frag_pos, 1.0)).xyz;
    let tangent = get_noise_rotation(world_pos);
    let T = normalize(tangent - normal * dot(tangent, normal));
    let B = cross(normal, T);
    let TBN = mat3x3<f32>(T, B, normal);
    let view_depth = abs(frag_pos.z);
    let effective_radius = params.sample_radius * view_depth * 0.08;
    var occluded = 0.0;
    for (var i = 0u; i < SAMPLE_COUNT; i++) {
        let sample_offset = TBN * SAMPLE_KERNEL[i];
        let sample_pos = frag_pos + sample_offset * effective_radius;
        let sample_clip = params.proj * vec4<f32>(sample_pos, 1.0);
        let sample_ndc_xy = sample_clip.xy / sample_clip.w;
        let sample_uv_raw = sample_ndc_xy * 0.5 + 0.5;
        let sample_uv_flip = vec2<f32>(sample_uv_raw.x, 1.0 - sample_uv_raw.y);
        if (sample_uv_flip.x < 0.0 || sample_uv_flip.x > 1.0 || sample_uv_flip.y < 0.0 || sample_uv_flip.y > 1.0) {
            continue;
        }
        let sample_depth = textureSample(depth_texture, depth_sampler, sample_uv_flip);
        let sample_view_pos = get_view_pos_from_depth(sample_uv_flip, sample_depth);
        let diff = sample_view_pos.z - frag_pos.z;
        let range_check = smoothstep(0.0, 1.0, effective_radius / abs(diff));
        if (diff > -params.bias && diff < effective_radius) {
            occluded += range_check;
        }
    }
    var ao = 1.0 - (occluded / f32(SAMPLE_COUNT)) * params.intensity;
    let depth_delta = max(abs(depth - depth_x), abs(depth - depth_y));
    let edge_fade = 1.0 - smoothstep(0.08, 0.25, depth_delta);
    ao = mix(1.0, ao, edge_fade);
    let depth_fade = smoothstep(0.8, 0.96, depth);
    let sky_neighbor = select(0.0, 0.9, depth_x >= 0.99 || depth_y >= 0.99);
    let horizon_fade = max(depth_fade, sky_neighbor);
    ao = mix(ao, 1.0, horizon_fade);
    return vec4<f32>(ao, ao, ao, 1.0);
}
