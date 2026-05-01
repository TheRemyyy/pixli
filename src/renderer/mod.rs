//! Renderer: meshes, materials, textures, camera, lights.

mod bloom;
mod camera;
mod constants;
mod gpu_profiler;
mod init;
mod light;
mod material;
mod mesh;
mod pass_common;
mod post;
mod ssao;
mod texture;
mod types;

pub use camera::Camera;
pub use light::{Light, LightType};
pub use material::Material;
pub use mesh::{Mesh, Vertex};
pub use texture::Texture;
pub use types::{
    BloomSettings, GpuMesh, GpuVertex, GraphicsSettings, ShadowSettings, SsaoSettings, Uniforms,
    UnlitMesh, UnlitMeshRef, UnlitVertex,
};

use constants::*;
use gpu_profiler::{GpuPass, GpuProfiler};
use types::{LitInstance, MatrixInstance, SkyUniform, UnlitSceneUniform};

use crate::ecs::{Entity, World};
use crate::math::{Color, Mat4, Transform, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default)]
struct RenderCpuFrame {
    cull_sort: Duration,
    depth: Duration,
    shadow: Duration,
    main: Duration,
    bloom: Duration,
    ssao: Duration,
    post: Duration,
    submit: Duration,
    visible_lit: usize,
    visible_unlit: usize,
    culled_lit: usize,
    culled_unlit: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct DurationAccumulator {
    total: Duration,
    max: Duration,
}

impl DurationAccumulator {
    fn record(&mut self, duration: Duration) {
        self.total += duration;
        self.max = self.max.max(duration);
    }

    fn average_ms(self, samples: u32) -> f64 {
        if samples == 0 {
            return 0.0;
        }
        self.total.as_secs_f64() * 1000.0 / f64::from(samples)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RenderCpuStats {
    cull_sort: DurationAccumulator,
    depth: DurationAccumulator,
    shadow: DurationAccumulator,
    main: DurationAccumulator,
    bloom: DurationAccumulator,
    ssao: DurationAccumulator,
    post: DurationAccumulator,
    submit: DurationAccumulator,
    visible_lit: usize,
    visible_unlit: usize,
    culled_lit: usize,
    culled_unlit: usize,
}

impl RenderCpuStats {
    fn record(&mut self, frame: RenderCpuFrame) {
        self.cull_sort.record(frame.cull_sort);
        self.depth.record(frame.depth);
        self.shadow.record(frame.shadow);
        self.main.record(frame.main);
        self.bloom.record(frame.bloom);
        self.ssao.record(frame.ssao);
        self.post.record(frame.post);
        self.submit.record(frame.submit);
        self.visible_lit += frame.visible_lit;
        self.visible_unlit += frame.visible_unlit;
        self.culled_lit += frame.culled_lit;
        self.culled_unlit += frame.culled_unlit;
    }
}

struct RenderCpuProfiler {
    enabled: bool,
    report_interval_frames: u32,
    frames_since_report: u32,
    stats: RenderCpuStats,
    latest_summary: String,
}

impl RenderCpuProfiler {
    fn from_env() -> Self {
        let enabled = std::env::var("PIXLI_PROFILE").is_ok();
        let report_interval_frames = std::env::var("PIXLI_PROFILE_INTERVAL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(120);
        Self {
            enabled,
            report_interval_frames,
            frames_since_report: 0,
            stats: RenderCpuStats::default(),
            latest_summary: String::new(),
        }
    }

    fn latest_summary(&self) -> Option<String> {
        if self.latest_summary.is_empty() {
            None
        } else {
            Some(self.latest_summary.clone())
        }
    }

    fn record(&mut self, frame: RenderCpuFrame) {
        if !self.enabled {
            return;
        }
        self.stats.record(frame);
        self.frames_since_report += 1;
        if self.frames_since_report >= self.report_interval_frames {
            self.latest_summary = format_render_cpu_summary(self.stats, self.frames_since_report);
            log::info!("Renderer CPU profiler: {}", self.latest_summary);
            self.frames_since_report = 0;
            self.stats = RenderCpuStats::default();
        }
    }
}

fn format_render_cpu_summary(stats: RenderCpuStats, samples: u32) -> String {
    let samples_usize = samples as usize;
    format!(
        "cpu-pass avg ms cull={:.3} depth={:.3} shadow={:.3} main={:.3} bloom={:.3} ssao={:.3} post={:.3} submit={:.3}; visible lit={} unlit={} culled lit={} unlit={}",
        stats.cull_sort.average_ms(samples),
        stats.depth.average_ms(samples),
        stats.shadow.average_ms(samples),
        stats.main.average_ms(samples),
        stats.bloom.average_ms(samples),
        stats.ssao.average_ms(samples),
        stats.post.average_ms(samples),
        stats.submit.average_ms(samples),
        stats.visible_lit / samples_usize,
        stats.visible_unlit / samples_usize,
        stats.culled_lit / samples_usize,
        stats.culled_unlit / samples_usize,
    )
}

#[derive(Clone, Copy)]
struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    fn normalize(self) -> Self {
        let length = self.normal.length();
        if length <= f32::EPSILON {
            return self;
        }
        Self {
            normal: self.normal / length,
            distance: self.distance / length,
        }
    }

    fn signed_distance(self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    fn from_view_projection(matrix: Mat4) -> Self {
        let row0 = [matrix.x.x, matrix.y.x, matrix.z.x, matrix.w.x];
        let row1 = [matrix.x.y, matrix.y.y, matrix.z.y, matrix.w.y];
        let row2 = [matrix.x.z, matrix.y.z, matrix.z.z, matrix.w.z];
        let row3 = [matrix.x.w, matrix.y.w, matrix.z.w, matrix.w.w];
        Self {
            planes: [
                plane_from_rows(row3, row0, 1.0),
                plane_from_rows(row3, row0, -1.0),
                plane_from_rows(row3, row1, 1.0),
                plane_from_rows(row3, row1, -1.0),
                plane_from_rows(row3, row2, 1.0),
                plane_from_rows(row3, row2, -1.0),
            ],
        }
    }

    fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.signed_distance(center) >= -radius)
    }
}

fn plane_from_rows(row3: [f32; 4], row: [f32; 4], sign: f32) -> Plane {
    Plane {
        normal: Vec3::new(
            row3[0] + row[0] * sign,
            row3[1] + row[1] * sign,
            row3[2] + row[2] * sign,
        ),
        distance: row3[3] + row[3] * sign,
    }
    .normalize()
}

fn max_scale_component(scale: Vec3) -> f32 {
    scale.x.abs().max(scale.y.abs()).max(scale.z.abs())
}

/// Main renderer.
pub struct Renderer {
    pub camera: Camera,
    pub ambient_light: Color,
    pub directional_light: Option<Light>,
    pub clear_color: Color,
    /// Fog: color, start distance, end distance (unlit scene).
    pub fog_color: Color,
    pub fog_start: f32,
    pub fog_end: f32,

    /// Graphics quality and post processing toggles.
    pub graphics: GraphicsSettings,

    // Internal state.
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    pipeline: Option<wgpu::RenderPipeline>,
    /// Lit uniform buffer: one slot per entity (dynamic offset), stride is LIT_UNIFORM_STRIDE.
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    lit_instance_buffer: Option<wgpu::Buffer>,
    depth_instance_buffer: Option<wgpu::Buffer>,
    shadow_instance_buffer: Option<wgpu::Buffer>,
    depth_texture: Option<wgpu::TextureView>,
    msaa_texture: Option<wgpu::TextureView>,
    mesh_cache: HashMap<u64, GpuMesh>,
    surface_format: wgpu::TextureFormat,
    msaa_samples: u32,
    size: (u32, u32),
    lit_pipeline_layout: Option<wgpu::PipelineLayout>,

    // Unlit pipeline (position and color only, no lighting).
    unlit_pipeline: Option<wgpu::RenderPipeline>,
    unlit_storage_buffer: Option<wgpu::Buffer>,
    unlit_batch_start_buffer: Option<wgpu::Buffer>,
    unlit_scene_buffer: Option<wgpu::Buffer>,
    unlit_bind_group: Option<wgpu::BindGroup>,
    unlit_mesh_cache: HashMap<u64, GpuMesh>,
    // Sky gradient (fullscreen).
    sky_pipeline: Option<wgpu::RenderPipeline>,
    sky_uniform_buffer: Option<wgpu::Buffer>,
    sky_bind_group: Option<wgpu::BindGroup>,
    sky_vertex_buffer: Option<wgpu::Buffer>,
    /// Scratch buffer for packing unlit model and MVP per instance.
    unlit_uniform_scratch: Vec<u8>,
    /// Scratch buffer for packing lit uniforms (256 bytes per entity).
    lit_uniform_scratch: Vec<u8>,
    lit_entity_scratch: Vec<Entity>,
    lit_sort_scratch: Vec<(u64, Entity)>,
    lit_instance_scratch: Vec<LitInstance>,
    matrix_instance_scratch: Vec<MatrixInstance>,
    lit_batch_scratch: Vec<(u64, u32, u32)>,
    matrix_batch_scratch: Vec<(u64, u32, u32)>,
    // Shadow mapping.
    shadow_map_texture: Option<wgpu::Texture>,
    shadow_map_view: Option<wgpu::TextureView>,
    shadow_sampler: Option<wgpu::Sampler>,
    shadow_light_view_proj_buffer: Option<wgpu::Buffer>,
    shadow_pipeline: Option<wgpu::RenderPipeline>,
    lit_shadow_bind_group_layout: Option<wgpu::BindGroupLayout>,
    lit_shadow_bind_group: Option<wgpu::BindGroup>,
    // Normal map (bump mapping), default flat 1x1.
    default_normal_map: Option<wgpu::Texture>,
    default_normal_map_view: Option<wgpu::TextureView>,
    normal_map_sampler: Option<wgpu::Sampler>,
    lit_normal_bind_group: Option<wgpu::BindGroup>,
    // Post process: render to texture, then tone mapping to swapchain.
    scene_texture: Option<wgpu::Texture>,
    scene_texture_view: Option<wgpu::TextureView>,
    post_pipeline: Option<wgpu::RenderPipeline>,
    post_bind_group_layout: Option<wgpu::BindGroupLayout>,
    post_bind_group: Option<wgpu::BindGroup>,
    post_vertex_buffer: Option<wgpu::Buffer>,
    post_sampler: Option<wgpu::Sampler>,
    // Bloom
    bloom_texture_a: Option<wgpu::Texture>,
    bloom_texture_a_view: Option<wgpu::TextureView>,
    bloom_texture_b: Option<wgpu::Texture>,
    bloom_texture_b_view: Option<wgpu::TextureView>,
    bloom_extract_pipeline: Option<wgpu::RenderPipeline>,
    bloom_blur_pipeline: Option<wgpu::RenderPipeline>,
    bloom_blur_params_buffer: Option<wgpu::Buffer>,
    bloom_bind_groups: Option<(wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup)>,
    // SSAO
    depth_ssao_texture: Option<wgpu::Texture>,
    depth_ssao_view: Option<wgpu::TextureView>,
    depth_prepass_lit_pipeline: Option<wgpu::RenderPipeline>,
    depth_prepass_unlit_pipeline: Option<wgpu::RenderPipeline>,
    depth_prepass_unlit_bind_group: Option<wgpu::BindGroup>,
    ssao_texture: Option<wgpu::Texture>,
    ssao_texture_view: Option<wgpu::TextureView>,
    ssao_blur_texture: Option<wgpu::Texture>,
    ssao_blur_view: Option<wgpu::TextureView>,
    ssao_pipeline: Option<wgpu::RenderPipeline>,
    ssao_params_buffer: Option<wgpu::Buffer>,
    ssao_bind_groups: Option<(wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup)>,
    gpu_profiler: Option<GpuProfiler>,
    cpu_profiler: RenderCpuProfiler,
}

impl Renderer {
    pub fn new() -> Self {
        let graphics = GraphicsSettings::default();
        let msaa_samples = graphics.msaa_samples;
        Self {
            camera: Camera::new(),
            ambient_light: Color::new(0.3, 0.3, 0.3, 1.0),
            directional_light: Some(Light {
                light_type: LightType::Directional,
                color: Color::WHITE,
                intensity: 1.0,
                position: Vec3::ZERO,
                direction: Vec3::new(-0.5, -1.0, -0.3).normalized(),
                range: 0.0,
                spot_angle: 0.0,
            }),
            clear_color: Color::new(0.5, 0.7, 1.0, 1.0),
            fog_color: Color::new(0.6, 0.75, 0.95, 1.0),
            fog_start: 30.0,
            fog_end: 120.0,
            graphics,
            device: None,
            queue: None,
            pipeline: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            lit_instance_buffer: None,
            depth_instance_buffer: None,
            shadow_instance_buffer: None,
            depth_texture: None,
            msaa_texture: None,
            mesh_cache: HashMap::new(),
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            msaa_samples,
            size: (1, 1),
            lit_pipeline_layout: None,
            unlit_pipeline: None,
            unlit_storage_buffer: None,
            unlit_batch_start_buffer: None,
            unlit_scene_buffer: None,
            unlit_bind_group: None,
            unlit_mesh_cache: HashMap::new(),
            sky_pipeline: None,
            sky_uniform_buffer: None,
            sky_bind_group: None,
            sky_vertex_buffer: None,
            unlit_uniform_scratch: Vec::with_capacity(
                MAX_UNLIT_DRAWS * UNLIT_INSTANCE_SIZE as usize,
            ),
            lit_uniform_scratch: Vec::with_capacity(MAX_LIT_DRAWS * LIT_UNIFORM_STRIDE as usize),
            lit_entity_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            lit_sort_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            lit_instance_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            matrix_instance_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            lit_batch_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            matrix_batch_scratch: Vec::with_capacity(MAX_LIT_DRAWS),
            shadow_map_texture: None,
            shadow_map_view: None,
            shadow_sampler: None,
            shadow_light_view_proj_buffer: None,
            shadow_pipeline: None,
            lit_shadow_bind_group_layout: None,
            lit_shadow_bind_group: None,
            default_normal_map: None,
            default_normal_map_view: None,
            normal_map_sampler: None,
            lit_normal_bind_group: None,
            scene_texture: None,
            scene_texture_view: None,
            post_pipeline: None,
            post_bind_group_layout: None,
            post_bind_group: None,
            post_vertex_buffer: None,
            post_sampler: None,
            bloom_texture_a: None,
            bloom_texture_a_view: None,
            bloom_texture_b: None,
            bloom_texture_b_view: None,
            bloom_extract_pipeline: None,
            bloom_blur_pipeline: None,
            bloom_blur_params_buffer: None,
            bloom_bind_groups: None,
            depth_ssao_texture: None,
            depth_ssao_view: None,
            depth_prepass_lit_pipeline: None,
            depth_prepass_unlit_pipeline: None,
            depth_prepass_unlit_bind_group: None,
            ssao_texture: None,
            ssao_texture_view: None,
            ssao_blur_texture: None,
            ssao_blur_view: None,
            ssao_pipeline: None,
            ssao_params_buffer: None,
            ssao_bind_groups: None,
            gpu_profiler: None,
            cpu_profiler: RenderCpuProfiler::from_env(),
        }
    }

    /// Upload unlit mesh (position and color) to GPU. Returns mesh id for `UnlitMeshRef(id)`.
    pub fn upload_unlit_mesh(&mut self, mesh: &UnlitMesh) -> u64 {
        let Some(device) = &self.device else { return 0 };
        let id = mesh.id();
        if self.unlit_mesh_cache.contains_key(&id) {
            return id;
        }
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unlit Mesh Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.unlit_mesh_cache.insert(
            id,
            GpuMesh {
                vertex_buffer,
                vertex_count: mesh.vertices.len() as u32,
                bounding_radius: mesh.bounding_radius,
            },
        );
        id
    }

    /// Initialize GPU resources.
    pub fn init(
        &mut self,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());
        self.surface_format = format;

        let r = init::create_gpu_resources(
            device.as_ref(),
            queue.as_ref(),
            format,
            self.msaa_samples,
            self.graphics.shadow.map_size,
        );
        self.uniform_buffer = Some(r.uniform_buffer);
        self.uniform_bind_group = Some(r.uniform_bind_group);
        self.lit_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lit Instance Buffer"),
            size: LIT_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.depth_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Instance Buffer"),
            size: MATRIX_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.shadow_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Instance Buffer"),
            size: MATRIX_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.pipeline = Some(r.pipeline);
        self.lit_pipeline_layout = Some(r.lit_pipeline_layout);
        self.lit_shadow_bind_group_layout = Some(r.lit_shadow_bind_group_layout);
        self.unlit_pipeline = Some(r.unlit_pipeline);
        self.unlit_storage_buffer = Some(r.unlit_storage_buffer);
        self.unlit_batch_start_buffer = Some(r.unlit_batch_start_buffer);
        self.unlit_scene_buffer = Some(r.unlit_scene_buffer);
        self.unlit_bind_group = Some(r.unlit_bind_group);
        self.depth_prepass_unlit_bind_group = Some(r.depth_prepass_unlit_bind_group);
        self.sky_pipeline = Some(r.sky_pipeline);
        self.sky_uniform_buffer = Some(r.sky_uniform_buffer);
        self.sky_bind_group = Some(r.sky_bind_group);
        self.sky_vertex_buffer = Some(r.sky_vertex_buffer);
        self.shadow_map_texture = Some(r.shadow_map_texture);
        self.shadow_map_view = Some(r.shadow_map_view);
        self.shadow_sampler = Some(r.shadow_sampler);
        self.shadow_light_view_proj_buffer = Some(r.shadow_light_view_proj_buffer);
        self.shadow_pipeline = Some(r.shadow_pipeline);
        self.lit_shadow_bind_group = Some(r.lit_shadow_bind_group);
        self.default_normal_map = Some(r.default_normal_map);
        self.default_normal_map_view = Some(r.default_normal_map_view);
        self.normal_map_sampler = Some(r.normal_map_sampler);
        self.lit_normal_bind_group = Some(r.lit_normal_bind_group);
        self.post_pipeline = Some(r.post_pipeline);
        self.post_bind_group_layout = Some(r.post_bind_group_layout);
        self.post_vertex_buffer = Some(r.post_vertex_buffer);
        self.post_sampler = Some(r.post_sampler);
        self.bloom_extract_pipeline = Some(r.bloom_extract_pipeline);
        self.bloom_blur_pipeline = Some(r.bloom_blur_pipeline);
        self.bloom_blur_params_buffer = Some(r.bloom_blur_params_buffer);
        self.depth_prepass_lit_pipeline = Some(r.depth_prepass_lit_pipeline);
        self.depth_prepass_unlit_pipeline = Some(r.depth_prepass_unlit_pipeline);
        self.ssao_pipeline = Some(r.ssao_pipeline);
        self.ssao_params_buffer = Some(r.ssao_params_buffer);

        self.size = (width.max(1), height.max(1));
        self.gpu_profiler = Some(GpuProfiler::new(device.as_ref(), queue.as_ref()));
        self.resize(width, height);
    }

    pub fn latest_gpu_profile_summary(&self) -> Option<String> {
        self.gpu_profiler
            .as_ref()
            .and_then(GpuProfiler::latest_summary)
    }

    pub fn latest_cpu_profile_summary(&self) -> Option<String> {
        self.cpu_profiler.latest_summary()
    }

    /// Resize render targets.
    pub fn resize(&mut self, width: u32, height: u32) {
        let Some(device) = &self.device else { return };

        let window_width = width.max(1);
        let window_height = height.max(1);

        // Internal render resolution can be scaled down for performance.
        let scale = self.graphics.render_scale.clamp(0.25, 1.0);
        let internal_width = ((window_width as f32 * scale).round().max(1.0)) as u32;
        let internal_height = ((window_height as f32 * scale).round().max(1.0)) as u32;

        // Stored size is internal render resolution (used by post, bloom, SSAO).
        self.size = (internal_width, internal_height);

        // Camera aspect should match window aspect, not internal scaling.
        self.camera.aspect = window_width as f32 / window_height as f32;

        // Create depth texture.
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.msaa_samples,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_texture =
            Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

        // Create MSAA texture.
        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.msaa_samples,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.msaa_texture = Some(msaa_texture.create_view(&wgpu::TextureViewDescriptor::default()));

        // Scene texture for post process (1 sample, resolve target or direct target).
        let scene_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Scene Texture"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let scene_texture_view = scene_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.scene_texture = Some(scene_texture);

        // Bloom textures (ping pong for blur).
        let bloom_desc = wgpu::TextureDescriptor {
            label: Some("Bloom A"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let bloom_a = device.create_texture(&bloom_desc);
        let bloom_b = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom B"),
            ..bloom_desc
        });
        let bloom_a_view = bloom_a.create_view(&Default::default());
        let bloom_b_view = bloom_b.create_view(&Default::default());

        // Bloom bind groups: extract scene to bloom_a, blur between bloom_a and bloom_b.
        if let (Some(extract_pipe), Some(blur_pipe), Some(params_buf), Some(sampler)) = (
            &self.bloom_extract_pipeline,
            &self.bloom_blur_pipeline,
            &self.bloom_blur_params_buffer,
            &self.post_sampler,
        ) {
            let extract_bgl = extract_pipe.get_bind_group_layout(0);
            let blur_bgl = blur_pipe.get_bind_group_layout(0);
            let bg_extract = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &extract_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&scene_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: Some("bloom_extract_bg"),
            });
            let bg_blur_a_in = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&bloom_a_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buf.as_entire_binding(),
                    },
                ],
                label: Some("bloom_blur_a_in"),
            });
            let bg_blur_b_in = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&bloom_b_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buf.as_entire_binding(),
                    },
                ],
                label: Some("bloom_blur_b_in"),
            });
            self.bloom_bind_groups = Some((bg_extract, bg_blur_a_in, bg_blur_b_in));
        }

        self.scene_texture_view = Some(scene_texture_view);
        self.bloom_texture_a = Some(bloom_a);
        self.bloom_texture_b = Some(bloom_b);
        self.bloom_texture_a_view = Some(bloom_a_view);
        self.bloom_texture_b_view = Some(bloom_b_view);

        // SSAO textures: depth (1 sample), AO output, AO blur.
        let depth_ssao = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth SSAO"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_ssao_view = depth_ssao.create_view(&Default::default());
        self.depth_ssao_texture = Some(depth_ssao);
        self.depth_ssao_view = Some(depth_ssao_view);

        let ssao_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ssao_tex_view = ssao_tex.create_view(&Default::default());
        self.ssao_texture = Some(ssao_tex);
        self.ssao_texture_view = Some(ssao_tex_view);

        let ssao_blur = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Blur"),
            size: wgpu::Extent3d {
                width: internal_width,
                height: internal_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ssao_blur_view = ssao_blur.create_view(&Default::default());
        self.ssao_blur_texture = Some(ssao_blur);
        self.ssao_blur_view = Some(ssao_blur_view);

        if let (
            Some(depth_ssao_view_ref),
            Some(ssao_pipe),
            Some(params_buf),
            Some(sampler),
            Some(blur_pipe),
            Some(blur_params_buffer),
            Some(ssao_tex_view),
            Some(ssao_blur_view_ref),
        ) = (
            self.depth_ssao_view.as_ref(),
            &self.ssao_pipeline,
            &self.ssao_params_buffer,
            &self.post_sampler,
            &self.bloom_blur_pipeline,
            &self.bloom_blur_params_buffer,
            &self.ssao_texture_view,
            self.ssao_blur_view.as_ref(),
        ) {
            let ssao_bgl = ssao_pipe.get_bind_group_layout(0);
            let ssao_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &ssao_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(depth_ssao_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buf.as_entire_binding(),
                    },
                ],
                label: Some("ssao_bg"),
            });
            let ssao_blur_bgl = blur_pipe.get_bind_group_layout(0);
            let ssao_blur_bg_in = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &ssao_blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(ssao_tex_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: blur_params_buffer.as_entire_binding(),
                    },
                ],
                label: Some("ssao_blur_in"),
            });
            let ssao_blur_bg_out = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &ssao_blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(ssao_blur_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: blur_params_buffer.as_entire_binding(),
                    },
                ],
                label: Some("ssao_blur_out"),
            });
            self.ssao_bind_groups = Some((ssao_bg, ssao_blur_bg_in, ssao_blur_bg_out));
        }

        if let (Some(layout), Some(sampler), Some(sv), Some(bloom_view), Some(ssao_view)) = (
            &self.post_bind_group_layout,
            &self.post_sampler,
            &self.scene_texture_view,
            &self.bloom_texture_a_view,
            &self.ssao_texture_view,
        ) {
            self.post_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(sv),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(bloom_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(ssao_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: Some("post_bind_group"),
            }));
        }
    }

    /// Upload a mesh to GPU.
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> u64 {
        let Some(device) = &self.device else { return 0 };

        let id = mesh.id();

        if self.mesh_cache.contains_key(&id) {
            return id;
        }

        let vertices: Vec<GpuVertex> = mesh
            .vertices
            .iter()
            .map(|v| GpuVertex {
                position: v.position.to_array(),
                normal: v.normal.to_array(),
                tangent: v.tangent.to_array(),
                uv: v.uv.to_array(),
                color: v.color.to_array(),
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.mesh_cache.insert(
            id,
            GpuMesh {
                vertex_buffer,
                vertex_count: vertices.len() as u32,
                bounding_radius: mesh.bounding_radius,
            },
        );

        id
    }

    /// Build unlit batches and fill instance scratch; caller must upload via queue. Returns (batches, entity_order).
    fn build_unlit_batches(
        &mut self,
        world: &World,
        view_proj: Mat4,
    ) -> (Vec<(u64, u32, u32)>, Vec<Entity>) {
        let frustum = Frustum::from_view_projection(view_proj);
        let mut unlit_entities: Vec<(u64, Entity)> = world
            .query::<(&Transform, &UnlitMeshRef)>()
            .iter()
            .filter_map(|entity| {
                let transform = world.get::<Transform>(entity)?;
                let unlit_ref = world.get::<UnlitMeshRef>(entity)?;
                let mesh = self.unlit_mesh_cache.get(&unlit_ref.0)?;
                if !frustum.contains_sphere(
                    transform.position,
                    mesh.bounding_radius * max_scale_component(transform.scale),
                ) {
                    return None;
                }
                Some((unlit_ref.0, entity))
            })
            .take(MAX_UNLIT_DRAWS)
            .collect();
        unlit_entities.sort_unstable_by_key(|(mesh_id, _)| *mesh_id);
        let mut batches: Vec<(u64, u32, u32)> = Vec::new();
        let mut entity_order: Vec<Entity> = Vec::new();
        let mut current_mesh_id = None;
        for (mesh_id, entity) in unlit_entities {
            let instance_index = entity_order.len() as u32;
            match current_mesh_id {
                Some(active_mesh_id) if active_mesh_id == mesh_id => {
                    if let Some((_, _, count)) = batches.last_mut() {
                        *count += 1;
                    }
                }
                _ => {
                    batches.push((mesh_id, instance_index, 1));
                    current_mesh_id = Some(mesh_id);
                }
            }
            entity_order.push(entity);
        }
        if !entity_order.is_empty() {
            let n = entity_order.len();
            self.unlit_uniform_scratch
                .resize(n * UNLIT_INSTANCE_SIZE as usize, 0);
            for (idx, &entity) in entity_order.iter().enumerate() {
                let Some(transform) = world.get::<Transform>(entity) else {
                    continue;
                };
                let model = transform.matrix();
                let mvp = view_proj * model;
                let model_arr = model.to_cols_array();
                let mvp_arr = mvp.to_cols_array();
                let slot_start = idx * UNLIT_INSTANCE_SIZE as usize;
                self.unlit_uniform_scratch[slot_start..slot_start + 64]
                    .copy_from_slice(bytemuck::bytes_of(&model_arr));
                self.unlit_uniform_scratch[slot_start + 64..slot_start + 128]
                    .copy_from_slice(bytemuck::bytes_of(&mvp_arr));
            }
        }
        (batches, entity_order)
    }

    /// Render the world (unlit and lit entities in one pass).
    pub fn render(&mut self, world: &World, view: &wgpu::TextureView) {
        let mut frame_profile = RenderCpuFrame::default();
        let view_matrix = self.camera.view_matrix();
        let proj_matrix = self.camera.projection_matrix();
        let view_proj = proj_matrix * view_matrix;
        let cull_sort_start = Instant::now();
        let total_unlit = world.query::<(&Transform, &UnlitMeshRef)>().iter().count();
        let (batches, entity_order) = self.build_unlit_batches(world, view_proj);
        self.lit_entity_scratch.clear();
        self.lit_sort_scratch.clear();
        let frustum = Frustum::from_view_projection(view_proj);
        let total_lit = world.query::<(&Transform, &Mesh)>().iter().count();
        self.lit_sort_scratch
            .extend(
                world
                    .query::<(&Transform, &Mesh)>()
                    .iter()
                    .filter_map(|entity| {
                        let transform = world.get::<Transform>(entity)?;
                        let mesh = world.get::<Mesh>(entity)?;
                        if !frustum.contains_sphere(
                            transform.position,
                            mesh.bounding_radius * max_scale_component(transform.scale),
                        ) {
                            return None;
                        }
                        Some((mesh.id(), entity))
                    }),
            );
        self.lit_sort_scratch
            .sort_unstable_by_key(|(mesh_id, _)| *mesh_id);
        self.lit_entity_scratch.extend(
            self.lit_sort_scratch
                .iter()
                .take(MAX_LIT_DRAWS)
                .map(|(_, entity)| *entity),
        );
        let lit_entities = &self.lit_entity_scratch;
        frame_profile.visible_lit = lit_entities.len();
        frame_profile.visible_unlit = entity_order.len();
        frame_profile.culled_lit = total_lit.saturating_sub(lit_entities.len());
        frame_profile.culled_unlit = total_unlit.saturating_sub(entity_order.len());
        frame_profile.cull_sort = cull_sort_start.elapsed();

        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };
        let Some(pipeline) = &self.pipeline else {
            return;
        };
        let Some(uniform_buffer) = &self.uniform_buffer else {
            return;
        };
        let Some(uniform_bind_group) = &self.uniform_bind_group else {
            return;
        };
        let Some(lit_instance_buffer) = &self.lit_instance_buffer else {
            return;
        };
        let Some(depth_instance_buffer) = &self.depth_instance_buffer else {
            return;
        };
        let Some(shadow_instance_buffer) = &self.shadow_instance_buffer else {
            return;
        };
        let Some(depth_texture) = &self.depth_texture else {
            return;
        };
        let Some(msaa_texture) = &self.msaa_texture else {
            return;
        };
        let Some(unlit_pipeline) = &self.unlit_pipeline else {
            return;
        };
        let Some(unlit_bind_group) = &self.unlit_bind_group else {
            return;
        };
        let Some(unlit_storage_buffer) = &self.unlit_storage_buffer else {
            return;
        };
        let Some(unlit_batch_start_buffer) = &self.unlit_batch_start_buffer else {
            return;
        };
        let Some(unlit_scene_buffer) = &self.unlit_scene_buffer else {
            return;
        };
        let Some(sky_pipeline) = &self.sky_pipeline else {
            return;
        };
        let Some(sky_bind_group) = &self.sky_bind_group else {
            return;
        };
        let Some(sky_vertex_buffer) = &self.sky_vertex_buffer else {
            return;
        };
        let Some(sky_uniform_buffer) = &self.sky_uniform_buffer else {
            return;
        };
        let Some(scene_texture_view) = &self.scene_texture_view else {
            return;
        };
        let Some(post_pipeline) = &self.post_pipeline else {
            return;
        };
        let Some(post_bind_group) = &self.post_bind_group else {
            return;
        };
        let Some(post_vertex_buffer) = &self.post_vertex_buffer else {
            return;
        };
        let shadow_map_view = self.shadow_map_view.as_ref();
        let shadow_light_view_proj_buffer = self.shadow_light_view_proj_buffer.as_ref();
        let shadow_pipeline = self.shadow_pipeline.as_ref();
        let lit_shadow_bind_group = self.lit_shadow_bind_group.as_ref();

        let (light_dir, light_color, light_intensity) =
            if let Some(ref light) = self.directional_light {
                (light.direction, light.color, light.intensity)
            } else {
                (Vec3::new(0.0, -1.0, 0.0), Color::WHITE, 1.0)
            };

        if !entity_order.is_empty() {
            let n = entity_order.len();
            queue.write_buffer(
                unlit_storage_buffer,
                0,
                &self.unlit_uniform_scratch[..n * UNLIT_INSTANCE_SIZE as usize],
            );
            for (batch_idx, (_, start, _)) in batches.iter().enumerate() {
                let offset = (batch_idx as u64) * UNLIT_BATCH_START_STRIDE;
                queue.write_buffer(unlit_batch_start_buffer, offset, &start.to_le_bytes());
            }
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        if let Some(gpu_profiler) = self.gpu_profiler.as_mut() {
            gpu_profiler.collect(device);
            if gpu_profiler.is_enabled() {
                gpu_profiler.begin_frame(&mut encoder);
            }
        }

        // Depth pre pass for SSAO: 1 sample depth (independent of MSAA).
        let depth_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Depth);
        }
        if self.graphics.enable_ssao {
            if let (
                Some(depth_ssao_view),
                Some(depth_lit_pipe),
                Some(depth_unlit_pipe),
                Some(depth_unlit_bg),
            ) = (
                self.depth_ssao_view.as_ref(),
                &self.depth_prepass_lit_pipeline,
                &self.depth_prepass_unlit_pipeline,
                &self.depth_prepass_unlit_bind_group,
            ) {
                run_depth_prepass_ssao(
                    &mut encoder,
                    queue,
                    world,
                    view_proj,
                    lit_entities,
                    &batches,
                    &entity_order,
                    &mut self.matrix_instance_scratch,
                    &mut self.matrix_batch_scratch,
                    depth_ssao_view,
                    depth_lit_pipe,
                    depth_unlit_pipe,
                    depth_unlit_bg,
                    depth_instance_buffer,
                    &self.mesh_cache,
                    &self.unlit_mesh_cache,
                );
            }
        }
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Depth);
        }
        frame_profile.depth = depth_start.elapsed();

        // Shadow pass: render lit geometry from light view into shadow map.
        let shadow_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Shadow);
        }
        if let (Some(light), Some(sv), Some(slb), Some(sp)) = (
            self.directional_light.as_ref(),
            shadow_map_view,
            shadow_light_view_proj_buffer,
            shadow_pipeline,
        ) {
            run_shadow_pass(
                &mut encoder,
                queue,
                world,
                lit_entities,
                light,
                self.camera.position,
                self.graphics.enable_shadows,
                &mut self.matrix_instance_scratch,
                &mut self.matrix_batch_scratch,
                sv,
                slb,
                shadow_instance_buffer,
                sp,
                &self.mesh_cache,
            );
        }
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Shadow);
        }
        frame_profile.shadow = shadow_start.elapsed();

        let main_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Main);
        }
        run_main_pass(
            &mut encoder,
            queue,
            world,
            view_proj,
            lit_entities,
            &batches,
            &entity_order,
            self.msaa_samples,
            msaa_texture,
            scene_texture_view,
            depth_texture,
            self.clear_color,
            self.graphics.enable_sky,
            self.graphics.enable_fog,
            self.fog_start,
            self.fog_end,
            self.fog_color,
            self.camera.position,
            sky_uniform_buffer,
            sky_pipeline,
            sky_bind_group,
            sky_vertex_buffer,
            unlit_scene_buffer,
            unlit_pipeline,
            unlit_bind_group,
            &self.unlit_mesh_cache,
            &mut self.lit_uniform_scratch,
            &mut self.lit_instance_scratch,
            &mut self.lit_batch_scratch,
            uniform_buffer,
            uniform_bind_group,
            lit_instance_buffer,
            pipeline,
            lit_shadow_bind_group,
            self.lit_normal_bind_group.as_ref(),
            &self.mesh_cache,
            self.ambient_light,
            light_dir,
            light_color,
            light_intensity,
        );
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Main);
        }
        frame_profile.main = main_start.elapsed();

        // Bloom: extract bright, blur horizontal, blur vertical.
        let bloom_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Bloom);
        }
        bloom::run_bloom(
            &mut encoder,
            queue,
            self.size,
            self.graphics.bloom.blur_passes,
            self.graphics.enable_bloom,
            self.bloom_extract_pipeline.as_ref(),
            self.bloom_blur_pipeline.as_ref(),
            self.bloom_bind_groups.as_ref(),
            self.bloom_blur_params_buffer.as_ref(),
            self.bloom_texture_a_view.as_ref(),
            self.bloom_texture_b_view.as_ref(),
            self.post_vertex_buffer.as_ref(),
        );
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Bloom);
        }
        frame_profile.bloom = bloom_start.elapsed();

        // SSAO pass and blur.
        let ssao_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Ssao);
        }
        ssao::run_ssao_pass(
            &mut encoder,
            queue,
            self.size,
            self.camera.projection_matrix(),
            self.camera.view_matrix(),
            self.graphics.ssao,
            self.graphics.enable_ssao,
            self.ssao_pipeline.as_ref(),
            self.ssao_params_buffer.as_ref(),
            self.ssao_bind_groups.as_ref(),
            self.bloom_blur_pipeline.as_ref(),
            self.bloom_blur_params_buffer.as_ref(),
            self.ssao_texture_view.as_ref(),
            self.ssao_blur_view.as_ref(),
            self.post_vertex_buffer.as_ref(),
            self.ssao_texture.as_ref(),
        );
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Ssao);
        }
        frame_profile.ssao = ssao_start.elapsed();

        // Post pass: scene, bloom, SSAO composite, tone mapping to swapchain.
        let post_start = Instant::now();
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.begin_pass(&mut encoder, GpuPass::Post);
        }
        post::run_post_pass(
            &mut encoder,
            view,
            post_pipeline,
            post_bind_group,
            post_vertex_buffer,
        );
        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            gpu_profiler.end_pass(&mut encoder, GpuPass::Post);
        }
        frame_profile.post = post_start.elapsed();

        if let Some(gpu_profiler) = self.gpu_profiler.as_ref() {
            if gpu_profiler.is_enabled() {
                gpu_profiler.end_frame(&mut encoder);
            }
        }
        let submit_start = Instant::now();
        queue.submit(std::iter::once(encoder.finish()));
        frame_profile.submit = submit_start.elapsed();
        if let Some(gpu_profiler) = self.gpu_profiler.as_mut() {
            gpu_profiler.begin_readback();
        }
        self.cpu_profiler.record(frame_profile);
    }

    /// Set MSAA sample count (1, 2, 4, or 8). After init, changing this recreates pipelines and targets.
    pub fn set_msaa(&mut self, samples: u32) {
        let new_samples = samples.clamp(1, 8);
        if new_samples == self.msaa_samples {
            return;
        }
        self.msaa_samples = new_samples;
        self.graphics.msaa_samples = new_samples;
        if self.device.is_some() {
            self.resize(self.size.0, self.size.1);
            self.recreate_pipelines();
        }
    }

    fn recreate_pipelines(&mut self) {
        if let (Some(device), Some(layout)) =
            (self.device.as_ref(), self.lit_pipeline_layout.as_ref())
        {
            let (pipeline, unlit_pipeline, sky_pipeline) = init::recreate_msaa_pipelines(
                device,
                self.surface_format,
                self.msaa_samples,
                layout,
            );
            self.pipeline = Some(pipeline);
            self.unlit_pipeline = Some(unlit_pipeline);
            self.sky_pipeline = Some(sky_pipeline);
        } else {
            log::warn!("recreate_pipelines: device or pipeline layout missing, skipping");
        }
    }
}

fn build_lit_matrix_batches(
    world: &World,
    lit_entities: &[Entity],
    matrix_instance_scratch: &mut Vec<MatrixInstance>,
    matrix_batch_scratch: &mut Vec<(u64, u32, u32)>,
    transform_matrix: impl Fn(&Transform) -> Mat4,
) {
    matrix_instance_scratch.clear();
    matrix_batch_scratch.clear();
    let mut current_mesh_id = None;
    for entity in lit_entities.iter() {
        let Some(transform) = world.get::<Transform>(*entity) else {
            continue;
        };
        let Some(mesh) = world.get::<Mesh>(*entity) else {
            continue;
        };
        let mesh_id = mesh.id();
        let instance_index = matrix_instance_scratch.len() as u32;
        match current_mesh_id {
            Some(active_mesh_id) if active_mesh_id == mesh_id => {
                if let Some((_, _, count)) = matrix_batch_scratch.last_mut() {
                    *count += 1;
                }
            }
            _ => {
                matrix_batch_scratch.push((mesh_id, instance_index, 1));
                current_mesh_id = Some(mesh_id);
            }
        }
        matrix_instance_scratch.push(MatrixInstance {
            transform: transform_matrix(transform).to_cols_array(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn run_depth_prepass_ssao(
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    world: &World,
    view_proj: Mat4,
    lit_entities: &[Entity],
    batches: &[(u64, u32, u32)],
    entity_order: &[Entity],
    matrix_instance_scratch: &mut Vec<MatrixInstance>,
    matrix_batch_scratch: &mut Vec<(u64, u32, u32)>,
    depth_ssao_view: &wgpu::TextureView,
    depth_lit_pipe: &wgpu::RenderPipeline,
    depth_unlit_pipe: &wgpu::RenderPipeline,
    depth_unlit_bg: &wgpu::BindGroup,
    depth_instance_buffer: &wgpu::Buffer,
    mesh_cache: &HashMap<u64, GpuMesh>,
    unlit_mesh_cache: &HashMap<u64, GpuMesh>,
) {
    let mut depth_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Depth Prepass SSAO"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_ssao_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    if !entity_order.is_empty() {
        depth_pass.set_pipeline(depth_unlit_pipe);
        for (batch_idx, (mesh_id, _start, instance_count)) in batches.iter().enumerate() {
            depth_pass.set_bind_group(
                0,
                depth_unlit_bg,
                &[((batch_idx as u64) * UNLIT_BATCH_START_STRIDE) as u32],
            );
            if let Some(gpu_mesh) = unlit_mesh_cache.get(mesh_id) {
                depth_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                depth_pass.draw(0..gpu_mesh.vertex_count, 0..*instance_count);
            }
        }
    }
    if !lit_entities.is_empty() {
        build_lit_matrix_batches(
            world,
            lit_entities,
            matrix_instance_scratch,
            matrix_batch_scratch,
            |transform| view_proj * transform.matrix(),
        );
        if matrix_instance_scratch.is_empty() {
            return;
        }
        queue.write_buffer(
            depth_instance_buffer,
            0,
            bytemuck::cast_slice(matrix_instance_scratch),
        );
        depth_pass.set_pipeline(depth_lit_pipe);
        depth_pass.set_vertex_buffer(1, depth_instance_buffer.slice(..));
        for (mesh_id, start, count) in matrix_batch_scratch.iter().copied() {
            if let Some(gpu_mesh) = mesh_cache.get(&mesh_id) {
                depth_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                depth_pass.draw(0..gpu_mesh.vertex_count, start..start + count);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_shadow_pass(
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    world: &World,
    lit_entities: &[Entity],
    light: &Light,
    camera_position: Vec3,
    enable_shadows: bool,
    matrix_instance_scratch: &mut Vec<MatrixInstance>,
    matrix_batch_scratch: &mut Vec<(u64, u32, u32)>,
    shadow_map_view: &wgpu::TextureView,
    shadow_light_view_proj_buffer: &wgpu::Buffer,
    shadow_instance_buffer: &wgpu::Buffer,
    shadow_pipeline: &wgpu::RenderPipeline,
    mesh_cache: &HashMap<u64, GpuMesh>,
) {
    if enable_shadows && !lit_entities.is_empty() && light.light_type == LightType::Directional {
        let light_dir = light.direction;
        let light_eye = camera_position - light_dir * 70.0;
        let target = camera_position;
        let up = if light_dir.y.abs() > 0.99 {
            Vec3::RIGHT
        } else {
            Vec3::UP
        };
        let light_view = Mat4::look_at(light_eye, target, up);
        let light_proj = Mat4::orthographic(-55.0, 55.0, -55.0, 55.0, 1.0, 200.0);
        let light_view_proj = light_proj * light_view;
        queue.write_buffer(
            shadow_light_view_proj_buffer,
            0,
            bytemuck::cast_slice(&light_view_proj.to_cols_array()),
        );
        build_lit_matrix_batches(
            world,
            lit_entities,
            matrix_instance_scratch,
            matrix_batch_scratch,
            |transform| light_view_proj * transform.matrix(),
        );
        if matrix_instance_scratch.is_empty() {
            clear_shadow_map(encoder, shadow_map_view);
            return;
        }
        queue.write_buffer(
            shadow_instance_buffer,
            0,
            bytemuck::cast_slice(matrix_instance_scratch),
        );
        let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_map_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        shadow_pass.set_pipeline(shadow_pipeline);
        shadow_pass.set_vertex_buffer(1, shadow_instance_buffer.slice(..));
        for (mesh_id, start, count) in matrix_batch_scratch.iter().copied() {
            if let Some(gpu_mesh) = mesh_cache.get(&mesh_id) {
                shadow_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                shadow_pass.draw(0..gpu_mesh.vertex_count, start..start + count);
            }
        }
    } else {
        clear_shadow_map(encoder, shadow_map_view);
    }
}

fn clear_shadow_map(encoder: &mut wgpu::CommandEncoder, shadow_map_view: &wgpu::TextureView) {
    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Shadow Pass Clear"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: shadow_map_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });
}

#[allow(clippy::too_many_arguments)]
fn run_main_pass(
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    world: &World,
    view_proj: Mat4,
    lit_entities: &[Entity],
    batches: &[(u64, u32, u32)],
    entity_order: &[Entity],
    msaa_samples: u32,
    msaa_texture: &wgpu::TextureView,
    scene_texture_view: &wgpu::TextureView,
    depth_texture: &wgpu::TextureView,
    clear_color: Color,
    enable_sky: bool,
    enable_fog: bool,
    fog_start: f32,
    fog_end: f32,
    fog_color: Color,
    camera_position: Vec3,
    sky_uniform_buffer: &wgpu::Buffer,
    sky_pipeline: &wgpu::RenderPipeline,
    sky_bind_group: &wgpu::BindGroup,
    sky_vertex_buffer: &wgpu::Buffer,
    unlit_scene_buffer: &wgpu::Buffer,
    unlit_pipeline: &wgpu::RenderPipeline,
    unlit_bind_group: &wgpu::BindGroup,
    unlit_mesh_cache: &HashMap<u64, GpuMesh>,
    lit_uniform_scratch: &mut Vec<u8>,
    lit_instance_scratch: &mut Vec<LitInstance>,
    lit_batch_scratch: &mut Vec<(u64, u32, u32)>,
    uniform_buffer: &wgpu::Buffer,
    uniform_bind_group: &wgpu::BindGroup,
    lit_instance_buffer: &wgpu::Buffer,
    pipeline: &wgpu::RenderPipeline,
    lit_shadow_bind_group: Option<&wgpu::BindGroup>,
    lit_normal_bind_group: Option<&wgpu::BindGroup>,
    mesh_cache: &HashMap<u64, GpuMesh>,
    ambient_light: Color,
    light_dir: Vec3,
    light_color: Color,
    light_intensity: f32,
) {
    let (color_view, resolve_target) = if msaa_samples > 1 {
        (msaa_texture, Some(scene_texture_view))
    } else {
        (scene_texture_view, None)
    };
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: clear_color.r as f64,
                    g: clear_color.g as f64,
                    b: clear_color.b as f64,
                    a: clear_color.a as f64,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_texture,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    if enable_sky {
        let sky_top = [
            clear_color.r * 0.5,
            clear_color.g * 0.6,
            clear_color.b * 0.95,
        ];
        let sky_bottom = [clear_color.r, clear_color.g, clear_color.b];
        let sky_uniform = SkyUniform {
            top_color: sky_top,
            _pad0: 0.0,
            bottom_color: sky_bottom,
            _pad1: 0.0,
        };
        queue.write_buffer(sky_uniform_buffer, 0, bytemuck::bytes_of(&sky_uniform));
        render_pass.set_pipeline(sky_pipeline);
        render_pass.set_bind_group(0, sky_bind_group, &[]);
        render_pass.set_vertex_buffer(0, sky_vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }

    if !entity_order.is_empty() {
        let (fs, fe, fcol) = if enable_fog {
            (fog_start, fog_end, [fog_color.r, fog_color.g, fog_color.b])
        } else {
            (1e9, 2e9, [0.0, 0.0, 0.0])
        };
        let scene_uniform = UnlitSceneUniform {
            camera_pos: [camera_position.x, camera_position.y, camera_position.z],
            fog_start: fs,
            fog_end: fe,
            fog_color: fcol,
            _pad: 0.0,
        };
        queue.write_buffer(unlit_scene_buffer, 0, bytemuck::bytes_of(&scene_uniform));
        render_pass.set_pipeline(unlit_pipeline);
        for (batch_idx, (mesh_id, _start, instance_count)) in batches.iter().enumerate() {
            let dyn_offset = (batch_idx as u64) * UNLIT_BATCH_START_STRIDE;
            render_pass.set_bind_group(0, unlit_bind_group, &[dyn_offset as u32]);
            if let Some(gpu_mesh) = unlit_mesh_cache.get(mesh_id) {
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass.draw(0..gpu_mesh.vertex_count, 0..*instance_count);
            }
        }
    }

    if !lit_entities.is_empty() {
        let (fs, fe, fcol) = if enable_fog {
            (fog_start, fog_end, [fog_color.r, fog_color.g, fog_color.b])
        } else {
            (1e9, 2e9, [0.0, 0.0, 0.0])
        };
        let scene_uniform = Uniforms {
            mvp: Mat4::IDENTITY.to_cols_array(),
            model: Mat4::IDENTITY.to_cols_array(),
            view_pos: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            color: Color::WHITE.to_array(),
            ambient: ambient_light.to_array(),
            light_dir: [light_dir.x, light_dir.y, light_dir.z, 0.0],
            light_color: [light_color.r, light_color.g, light_color.b, light_intensity],
            fog_start: fs,
            fog_end: fe,
            fog_color: fcol,
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0, 1.0],
            emission_strength: 0.0,
            _pad: [0.0, 0.0],
        };
        let scene_uniforms = [scene_uniform];
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&scene_uniforms));

        lit_instance_scratch.clear();
        lit_batch_scratch.clear();
        let mut current_mesh_id = None;
        for entity in lit_entities.iter() {
            let Some(transform) = world.get::<Transform>(*entity) else {
                continue;
            };
            let Some(mesh) = world.get::<Mesh>(*entity) else {
                continue;
            };
            let material = world.get::<Material>(*entity);
            let color = material.map(|m| m.color).unwrap_or(Color::WHITE);
            let model = transform.matrix();
            let mvp = view_proj * model;
            let (emission, emission_strength) = material
                .map(|m| (m.emission.to_array(), m.emission_strength))
                .unwrap_or(([0.0, 0.0, 0.0, 1.0], 0.0));
            let mesh_id = mesh.id();
            let instance_index = lit_instance_scratch.len() as u32;
            match current_mesh_id {
                Some(active_mesh_id) if active_mesh_id == mesh_id => {
                    if let Some((_, _, count)) = lit_batch_scratch.last_mut() {
                        *count += 1;
                    }
                }
                _ => {
                    lit_batch_scratch.push((mesh_id, instance_index, 1));
                    current_mesh_id = Some(mesh_id);
                }
            }
            lit_instance_scratch.push(LitInstance {
                mvp: mvp.to_cols_array(),
                model: model.to_cols_array(),
                color: color.to_array(),
                material: [
                    material.map(|m| m.metallic).unwrap_or(0.0),
                    material.map(|m| m.roughness).unwrap_or(0.5),
                    emission_strength,
                    0.0,
                ],
                emission,
            });
        }
        if !lit_instance_scratch.is_empty() {
            lit_uniform_scratch.clear();
            queue.write_buffer(
                lit_instance_buffer,
                0,
                bytemuck::cast_slice(lit_instance_scratch),
            );
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, uniform_bind_group, &[0]);
            if let Some(lit_shadow_bg) = lit_shadow_bind_group {
                render_pass.set_bind_group(1, lit_shadow_bg, &[]);
            }
            if let Some(lit_normal_bg) = lit_normal_bind_group {
                render_pass.set_bind_group(2, lit_normal_bg, &[]);
            }
            render_pass.set_vertex_buffer(1, lit_instance_buffer.slice(..));
            for (mesh_id, start, count) in lit_batch_scratch.iter().copied() {
                if let Some(gpu_mesh) = mesh_cache.get(&mesh_id) {
                    render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                    render_pass.draw(0..gpu_mesh.vertex_count, start..start + count);
                }
            }
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

use wgpu::util::DeviceExt;
