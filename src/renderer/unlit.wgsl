// Unlit shader – instanced, with distance fog.
// Binding 0: storage buffer (model + mvp per instance).
// Binding 1: uniform batch start index.
// Binding 2: uniform scene (camera, fog).
// Instance count must match MAX_UNLIT_DRAWS in renderer/mod.rs.

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

struct SceneUniform {
    camera_pos: vec3<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_color: vec3<f32>,
    _pad: f32,
}

@group(0) @binding(2)
var<uniform> scene: SceneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let idx = batch.start + instance_index;
    let inst = instances[idx];
    let pos4 = vec4<f32>(model.position, 1.0);
    out.clip_position = inst.mvp * pos4;
    out.world_position = (inst.model * pos4).xyz;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = distance(in.world_position, scene.camera_pos);
    let fog_t = clamp((dist - scene.fog_start) / (scene.fog_end - scene.fog_start), 0.0, 1.0);
    let col = mix(vec3<f32>(in.color), scene.fog_color, fog_t);
    return vec4<f32>(col, 1.0);
}
