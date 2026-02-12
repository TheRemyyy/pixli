// Bloom: extract bright pixels (emission, neony) + Gaussian blur
// Pass 1: extract (threshold) - scene -> bloom_a
// Pass 2: blur horizontal - bloom_a -> bloom_b
// Pass 3: blur vertical - bloom_b -> bloom_a (nebo obráceně)

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;

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

// Extract: pouze pixely nad prahem (emissive, neony, muzzle flash)
const BLOOM_THRESHOLD: f32 = 0.85;
const BLOOM_INTENSITY: f32 = 0.6;

@fragment
fn fs_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(input_texture, input_sampler, in.uv);
    let luminance = dot(col.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let bright = select(vec3<f32>(0.0, 0.0, 0.0), col.rgb, luminance > BLOOM_THRESHOLD);
    return vec4<f32>(bright * BLOOM_INTENSITY, col.a);
}

// Uniform pro blur směr
struct BlurParams {
    texel_size: vec2<f32>,
    is_horizontal: u32,
    _pad: u32,
}

@group(0) @binding(2)
var<uniform> blur_params: BlurParams;

// Gaussian blur (9-tap separable)
const BLUR_WEIGHTS: array<f32, 5> = array<f32, 5>(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
const BLUR_OFFSET: f32 = 1.3846153846;

@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    var col = textureSample(input_texture, input_sampler, in.uv).rgb * BLUR_WEIGHTS[0];
    let offset = blur_params.texel_size * BLUR_OFFSET;
    
    if (blur_params.is_horizontal == 1u) {
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x, 0.0)).rgb * BLUR_WEIGHTS[1];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(offset.x, 0.0)).rgb * BLUR_WEIGHTS[1];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x * 2.0, 0.0)).rgb * BLUR_WEIGHTS[2];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(offset.x * 2.0, 0.0)).rgb * BLUR_WEIGHTS[2];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x * 3.0, 0.0)).rgb * BLUR_WEIGHTS[3];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(offset.x * 3.0, 0.0)).rgb * BLUR_WEIGHTS[3];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(offset.x * 4.0, 0.0)).rgb * BLUR_WEIGHTS[4];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(offset.x * 4.0, 0.0)).rgb * BLUR_WEIGHTS[4];
    } else {
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, offset.y)).rgb * BLUR_WEIGHTS[1];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(0.0, offset.y)).rgb * BLUR_WEIGHTS[1];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, offset.y * 2.0)).rgb * BLUR_WEIGHTS[2];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(0.0, offset.y * 2.0)).rgb * BLUR_WEIGHTS[2];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, offset.y * 3.0)).rgb * BLUR_WEIGHTS[3];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(0.0, offset.y * 3.0)).rgb * BLUR_WEIGHTS[3];
        col += textureSample(input_texture, input_sampler, in.uv + vec2<f32>(0.0, offset.y * 4.0)).rgb * BLUR_WEIGHTS[4];
        col += textureSample(input_texture, input_sampler, in.uv - vec2<f32>(0.0, offset.y * 4.0)).rgb * BLUR_WEIGHTS[4];
    }
    
    return vec4<f32>(col, 1.0);
}
