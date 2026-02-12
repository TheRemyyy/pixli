// Fullscreen sky gradient – jeden velký trojúhelník přes celou obrazovku (žádné čáry).
// Vertex: NDC pozice (-1,-1), (3,-1), (-1,3). Fragment: gradient podle y.

struct SkyUniform {
    top_color: vec3<f32>,
    _pad0: f32,
    bottom_color: vec3<f32>,
    _pad1: f32,
}

@group(0) @binding(0)
var<uniform> sky: SkyUniform;

struct VertexInput {
    @location(0) ndc_xy: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc_y: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.ndc_xy, 1.0, 1.0);
    out.ndc_y = in.ndc_xy.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = (in.ndc_y + 1.0) * 0.5;
    let col = mix(sky.bottom_color, sky.top_color, t);
    return vec4<f32>(col, 1.0);
}
