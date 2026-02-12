// Post-processing: Bloom + SSAO + Reinhard tone mapping
// Scene + bloom, aplikace SSAO (ztmavení), tone map -> swapchain

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;
@group(0) @binding(1)
var bloom_texture: texture_2d<f32>;
@group(0) @binding(2)
var ssao_texture: texture_2d<f32>;
@group(0) @binding(3)
var scene_sampler: sampler;

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

const SSAO_STRENGTH: f32 = 0.22;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_col = textureSample(scene_texture, scene_sampler, in.uv);
    let bloom_col = textureSample(bloom_texture, scene_sampler, in.uv);
    let ao = textureSample(ssao_texture, scene_sampler, in.uv).r;
    // Composite: scene + bloom, pak SSAO (ztmavení v rozích)
    var col = scene_col.rgb + bloom_col.rgb;
    col *= mix(1.0, ao, SSAO_STRENGTH);
    // Reinhard tone mapping
    let tone_mapped = col / (1.0 + col);
    return vec4<f32>(tone_mapped, scene_col.a);
}
