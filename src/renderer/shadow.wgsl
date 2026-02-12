// Shadow map pass – depth only, from light's view.
// Group 0: binding 0 = entity (model, light_mvp) dynamic offset, binding 1 = light_view_proj.

struct ShadowEntity {
    model: mat4x4<f32>,
    light_mvp: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> entity: ShadowEntity;

@group(0) @binding(1)
var<uniform> light_view_proj: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = entity.light_mvp * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main() {
    // Depth-only pass – depth comes from vertex clip_position.z
}
