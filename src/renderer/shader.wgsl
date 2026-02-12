// Lit shader: lighting + shadow mapping + distance fog

struct Uniforms {
    mvp: mat4x4<f32>,
    model: mat4x4<f32>,
    view_pos: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
    emission: vec4<f32>,
    emission_strength: f32,
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Shadow map: light view-projection + depth texture
@group(1) @binding(0)
var<uniform> light_view_proj: mat4x4<f32>;
@group(1) @binding(1)
var shadow_map: texture_depth_2d;
@group(1) @binding(2)
var shadow_sampler: sampler_comparison;

// Normal map (bump mapping) – skupina 2
@group(2) @binding(0)
var normal_map: texture_2d<f32>;
@group(2) @binding(1)
var normal_sampler: sampler;

// PCF: 9 samples in a 3x3 grid for soft shadows
const PCF_RADIUS: f32 = 1.5;
const PCF_TEXEL_SIZE: f32 = 1.0 / 2048.0; // SHADOW_MAP_SIZE

fn pcf_shadow(uv: vec2<f32>, depth_compare: f32) -> f32 {
    var sum = 0.0;
    let texel = PCF_RADIUS * PCF_TEXEL_SIZE;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            sum += textureSampleCompare(shadow_map, shadow_sampler, uv + offset, depth_compare);
        }
    }
    return sum / 9.0;
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_tangent: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_tangent = normalize((uniforms.model * vec4<f32>(in.tangent, 0.0)).xyz);
    out.uv = in.uv;
    out.color = in.color;
    
    return out;
}

// PBR: F0 for Fresnel (dielectric 0.04, metal = base_color)
fn pbr_f0(base_color: vec3<f32>, metallic: f32) -> vec3<f32> {
    return mix(vec3<f32>(0.04, 0.04, 0.04), base_color.rgb, metallic);
}
// Simplified Cook-Torrance specular (Blinn-Phong NdotH with roughness)
fn pbr_specular(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, F0: vec3<f32>, roughness: f32, light_color: vec3<f32>) -> vec3<f32> {
    let H = normalize(V + L);
    let NdotH = max(dot(N, H), 0.0);
    let NdotV = max(dot(N, V), 0.0001);
    let NdotL = max(dot(N, L), 0.0);
    let a = roughness * roughness * roughness * roughness;
    let a2 = a * a;
    let d = NdotH * NdotH * (a2 - 1.0) + 1.0;
    let D = a2 / max(3.14159265 * d * d, 0.0001);
    let F = F0 + (1.0 - F0) * pow(1.0 - max(dot(V, H), 0.0), 5.0);
    let G = 1.0;
    let spec = (D * F * G) / max(4.0 * NdotV * NdotL, 0.0001);
    return spec * light_color * NdotL;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Base color
    let base_color = uniforms.color * in.color;
    let metallic = clamp(uniforms.metallic, 0.0, 1.0);
    let roughness = clamp(uniforms.roughness, 0.04, 1.0);
    
    // Normal mapping: sample normal map, build TBN, transform to world space
    let N_raw = textureSample(normal_map, normal_sampler, in.uv).rgb;
    let N_ts = N_raw * 2.0 - 1.0; // tangent space [0,1] -> [-1,1]
    let T = normalize(in.world_tangent);
    let N_geom = normalize(in.world_normal);
    let B = cross(N_geom, T); // bitangent
    let TBN = mat3x3<f32>(T, B, N_geom);
    let N = normalize(TBN * N_ts);
    
    // Ambient lighting
    let ambient = uniforms.ambient.rgb * uniforms.ambient.a;
    
    // Directional light
    let light_dir = normalize(-uniforms.light_dir.xyz);
    let L = light_dir;
    let V = normalize(uniforms.view_pos.xyz - in.world_position);
    let NdotL = max(dot(N, L), 0.0);
    
    // PBR: diffuse (non-metals) + specular (F0, roughness)
    let F0 = pbr_f0(base_color.rgb, metallic);
    let diffuse_lambert = base_color.rgb * (1.0 - metallic) * NdotL * uniforms.light_color.rgb * uniforms.light_color.a;
    let specular = pbr_specular(N, V, L, F0, roughness, uniforms.light_color.rgb * uniforms.light_color.a);
    let diffuse = diffuse_lambert;
    
    // Shadow: PCF soft shadows – 9 samples, průměr pro měkké hrany
    let light_clip = light_view_proj * vec4<f32>(in.world_position, 1.0);
    let light_ndc = light_clip.xy / light_clip.w;
    let shadow_uv = light_ndc * 0.5 + 0.5;
    let depth_ndc = light_clip.z / light_clip.w;
    let depth_compare = depth_ndc * 0.5 + 0.5;
    let uv_in_bounds = (shadow_uv.x >= 0.0 && shadow_uv.x <= 1.0 && shadow_uv.y >= 0.0 && shadow_uv.y <= 1.0);
    let shadow_sample = pcf_shadow(shadow_uv, depth_compare);
    // shadow_sample: 1 = fully lit, 0 = fully shadowed (PCF dává průměr)
    let in_shadow = select(0.0, 1.0 - shadow_sample, uv_in_bounds);
    let direct_lighting = (diffuse + specular) * (1.0 - in_shadow * 0.85);
    
    // Combine PBR: ambient * base + direct (diffuse/specular already have base/F0)
    var final_color = ambient * base_color.rgb + direct_lighting;
    
    // Emissive: přidej svítící barvu (neony, výstřely)
    let emissive = uniforms.emission.rgb * uniforms.emission_strength;
    final_color += emissive;
    
    // Distance fog (same as unlit)
    let dist = distance(in.world_position, uniforms.view_pos.xyz);
    let fog_t = clamp((dist - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    final_color = mix(final_color, uniforms.fog_color, fog_t);
    
    return vec4<f32>(final_color, base_color.a);
}
