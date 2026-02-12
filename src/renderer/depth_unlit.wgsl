// Depth pre-pass pro unlit geometrii – stejné instance jako unlit, výstup jen depth

struct Instance {
    model: mat4x4<f32>,
    mvp: mat4x4<f32>,
}

@group(0) @binding(0)
var<storage, read> instances: array<Instance, 4096>;

struct BatchUniform {
    start: u32,
}

@group(0) @binding(1)
var<uniform> batch: BatchUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let idx = batch.start + instance_index;
    let inst = instances[idx];
    out.clip_position = inst.mvp * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main() {
    // Depth-only
}
