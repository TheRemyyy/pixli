// Depth pre-pass pro SSAO – 1-sample depth buffer (nezávislý na MSAA)
// Pouze depth output, žádná barva

struct DepthEntity {
    mvp: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> entity: DepthEntity;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = entity.mvp * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main() {
    // Depth-only – depth z rasterizer
}
