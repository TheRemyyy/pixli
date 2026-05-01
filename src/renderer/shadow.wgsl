// Shadow map pass – depth only, from light's view.

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(5) light_mvp_0: vec4<f32>,
    @location(6) light_mvp_1: vec4<f32>,
    @location(7) light_mvp_2: vec4<f32>,
    @location(8) light_mvp_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let light_mvp = mat4x4<f32>(in.light_mvp_0, in.light_mvp_1, in.light_mvp_2, in.light_mvp_3);
    out.clip_position = light_mvp * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main() {
    // Depth-only pass – depth comes from vertex clip_position.z
}
