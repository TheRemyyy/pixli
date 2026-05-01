// Depth pre-pass pro SSAO – 1-sample depth buffer (nezávislý na MSAA)
// Pouze depth output, žádná barva

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(5) mvp_0: vec4<f32>,
    @location(6) mvp_1: vec4<f32>,
    @location(7) mvp_2: vec4<f32>,
    @location(8) mvp_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let mvp = mat4x4<f32>(in.mvp_0, in.mvp_1, in.mvp_2, in.mvp_3);
    out.clip_position = mvp * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main() {
    // Depth-only – depth z rasterizer
}
