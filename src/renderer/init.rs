//! GPU resource creation for the renderer (pipelines, buffers, bind groups).

use super::constants::*;
use super::types::{GpuVertex, UnlitVertex};
use wgpu::util::DeviceExt;

pub(super) struct GpuResources {
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub lit_pipeline_layout: wgpu::PipelineLayout,
    pub lit_shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub unlit_pipeline: wgpu::RenderPipeline,
    pub unlit_storage_buffer: wgpu::Buffer,
    pub unlit_batch_start_buffer: wgpu::Buffer,
    pub unlit_scene_buffer: wgpu::Buffer,
    pub unlit_bind_group: wgpu::BindGroup,
    pub depth_prepass_unlit_bind_group: wgpu::BindGroup,
    pub sky_pipeline: wgpu::RenderPipeline,
    pub sky_uniform_buffer: wgpu::Buffer,
    pub sky_bind_group: wgpu::BindGroup,
    pub sky_vertex_buffer: wgpu::Buffer,
    pub shadow_map_texture: wgpu::Texture,
    pub shadow_map_view: wgpu::TextureView,
    pub shadow_sampler: wgpu::Sampler,
    pub shadow_light_view_proj_buffer: wgpu::Buffer,
    pub shadow_entity_buffer: wgpu::Buffer,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_bind_group: wgpu::BindGroup,
    pub lit_shadow_bind_group: wgpu::BindGroup,
    pub default_normal_map: wgpu::Texture,
    pub default_normal_map_view: wgpu::TextureView,
    pub normal_map_sampler: wgpu::Sampler,
    pub lit_normal_bind_group: wgpu::BindGroup,
    pub post_pipeline: wgpu::RenderPipeline,
    pub post_bind_group_layout: wgpu::BindGroupLayout,
    pub post_vertex_buffer: wgpu::Buffer,
    pub post_sampler: wgpu::Sampler,
    pub bloom_extract_pipeline: wgpu::RenderPipeline,
    pub bloom_blur_pipeline: wgpu::RenderPipeline,
    pub bloom_blur_params_buffer: wgpu::Buffer,
    pub depth_prepass_lit_pipeline: wgpu::RenderPipeline,
    pub depth_prepass_unlit_pipeline: wgpu::RenderPipeline,
    pub depth_ssao_entity_buffer: wgpu::Buffer,
    pub depth_prepass_lit_bind_group: wgpu::BindGroup,
    pub ssao_pipeline: wgpu::RenderPipeline,
    pub ssao_params_buffer: wgpu::Buffer,
}

pub(super) fn create_gpu_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    msaa_samples: u32,
) -> GpuResources {
    // Create shader.
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Renderer Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // Lit uniform buffer: multiple slots for dynamic offset (one per lit entity).
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: (MAX_LIT_DRAWS as u64) * LIT_UNIFORM_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: std::num::NonZeroU64::new(LIT_UNIFORM_STRIDE),
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    });

    let uniform_slot = wgpu::BufferBinding {
        buffer: &uniform_buffer,
        offset: 0,
        size: std::num::NonZeroU64::new(LIT_UNIFORM_STRIDE),
    };
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniform_slot),
        }],
        label: Some("uniform_bind_group"),
    });

    // Shadow bind group for main pass (lit): light view proj, shadow map, sampler.
    let lit_shadow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
        label: Some("lit_shadow_bind_group_layout"),
    });

    // Normal map bind group (group 2), flat default for bump mapping.
    let lit_normal_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("lit_normal_bind_group_layout"),
    });

    // Default flat normal map (128,128,255) maps to (0.5,0.5,1), no perturbation.
    let default_normal_data: [u8; 4] = [128, 128, 255, 255];
    let default_normal_map = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Default Normal Map"),
        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture { texture: &default_normal_map, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        &default_normal_data,
        wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
    );
    let default_normal_map_view = default_normal_map.create_view(&Default::default());
    let normal_map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Normal Map Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let lit_normal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &lit_normal_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&default_normal_map_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&normal_map_sampler) },
        ],
        label: Some("lit_normal_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout, &lit_shadow_bind_group_layout, &lit_normal_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[GpuVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });


    // Unlit pipeline: instanced, storage buffer (MVPs) and uniform (batch start).
    let unlit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unlit Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("unlit.wgsl").into()),
    });
    let unlit_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Unlit MVP Storage"),
        size: UNLIT_STORAGE_SIZE,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let unlit_batch_start_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Unlit Batch Start"),
        size: UNLIT_BATCH_START_BUFFER_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let unlit_scene_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Unlit Scene"),
        size: UNLIT_SCENE_UNIFORM_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let unlit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_STORAGE_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_BATCH_START_STRIDE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_SCENE_UNIFORM_SIZE),
                },
                count: None,
            },
        ],
        label: Some("unlit_bind_group_layout"),
    });
    let unlit_batch_slot = wgpu::BufferBinding {
        buffer: &unlit_batch_start_buffer,
        offset: 0,
        size: std::num::NonZeroU64::new(UNLIT_BATCH_START_STRIDE),
    };
    let unlit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &unlit_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: unlit_storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(unlit_batch_slot),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: unlit_scene_buffer.as_entire_binding(),
            },
        ],
        label: Some("unlit_bind_group"),
    });
    let unlit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[&unlit_bind_group_layout],
        push_constant_ranges: &[],
    });
    let unlit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unlit Render Pipeline"),
        layout: Some(&unlit_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &unlit_shader,
            entry_point: Some("vs_main"),
            buffers: &[UnlitVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &unlit_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });
    let depth_prepass_unlit_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_STORAGE_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_BATCH_START_STRIDE),
                },
                count: None,
            },
        ],
        label: Some("depth_unlit_bgl"),
    });
    let depth_prepass_unlit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &depth_prepass_unlit_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: unlit_storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &unlit_batch_start_buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(UNLIT_BATCH_START_STRIDE),
                }),
            },
        ],
        label: Some("depth_prepass_unlit_bg"),
    });

    // Sky gradient (fullscreen triangle).
    let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sky Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("sky.wgsl").into()),
    });
    let sky_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Sky Uniform"),
        size: SKY_UNIFORM_SIZE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let sky_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(SKY_UNIFORM_SIZE),
            },
            count: None,
        }],
        label: Some("sky_bind_group_layout"),
    });
    let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &sky_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: sky_uniform_buffer.as_entire_binding(),
        }],
        label: Some("sky_bind_group"),
    });
    let sky_vertices: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
    let sky_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sky Vertex Buffer"),
        contents: bytemuck::cast_slice(&sky_vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sky Pipeline Layout"),
        bind_group_layouts: &[&sky_bind_group_layout],
        push_constant_ranges: &[],
    });
    let sky_vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
    let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sky Pipeline"),
        layout: Some(&sky_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &sky_shader,
            entry_point: Some("vs_main"),
            buffers: &[sky_vertex_buffer_layout],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &sky_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    // Shadow map: texture and comparison sampler.
    let shadow_map_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Shadow Map"),
        size: wgpu::Extent3d { width: SHADOW_MAP_SIZE, height: SHADOW_MAP_SIZE, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let shadow_map_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Shadow Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        ..Default::default()
    });
    let shadow_light_view_proj_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Shadow Light View Proj"),
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shadow_entity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Shadow Entity Buffer"),
        size: (MAX_LIT_DRAWS as u64) * SHADOW_ENTITY_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shadow Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()),
    });
    let shadow_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(SHADOW_ENTITY_STRIDE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(64),
                },
                count: None,
            },
        ],
        label: Some("shadow_bind_group_layout"),
    });
    let shadow_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&shadow_bind_group_layout],
        push_constant_ranges: &[],
        label: Some("Shadow Pipeline Layout"),
    });
    let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Shadow Pipeline"),
        layout: Some(&shadow_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shadow_shader,
            entry_point: Some("vs_main"),
            buffers: &[GpuVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shadow_shader,
            entry_point: Some("fs_main"),
            targets: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &shadow_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_entity_buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(SHADOW_ENTITY_STRIDE),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_light_view_proj_buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(64),
                }),
            },
        ],
        label: Some("shadow_bind_group"),
    });
    let lit_shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &lit_shadow_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_light_view_proj_buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&shadow_map_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
        ],
        label: Some("lit_shadow_bind_group"),
    });

    // Post process: tone mapping (fullscreen pass).
    let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Post Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Post Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
    });
    let post_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("post_bind_group_layout"),
    });
    let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&post_bind_group_layout],
        push_constant_ranges: &[],
        label: Some("Post Pipeline Layout"),
    });
    let post_vertices: [[f32; 2]; 3] = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
    let post_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Post Vertex Buffer"),
        contents: bytemuck::cast_slice(&post_vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Post Pipeline"),
        layout: Some(&post_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &post_shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &post_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // Bloom pipelines.
    let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Bloom Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("bloom.wgsl").into()),
    });
    let bloom_extract_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("bloom_extract_bgl"),
    });
    let bloom_blur_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(16),
                },
                count: None,
            },
        ],
        label: Some("bloom_blur_bgl"),
    });
    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct BlurParams { texel_size: [f32; 2], is_horizontal: u32, _pad: u32 }
    let bloom_blur_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Bloom Blur Params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let post_fullscreen_vbl = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
    let post_fullscreen_vbl2 = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
    let bloom_extract_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bloom_extract_bgl],
        push_constant_ranges: &[],
        label: Some("bloom_extract_pl"),
    });
    let bloom_extract_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bloom Extract"),
        layout: Some(&bloom_extract_pl),
        vertex: wgpu::VertexState {
            module: &bloom_shader,
            entry_point: Some("vs_main"),
            buffers: &[post_fullscreen_vbl],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &bloom_shader,
            entry_point: Some("fs_extract"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let bloom_blur_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bloom_blur_bgl],
        push_constant_ranges: &[],
        label: Some("bloom_blur_pl"),
    });
    let bloom_blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Bloom Blur"),
        layout: Some(&bloom_blur_pl),
        vertex: wgpu::VertexState {
            module: &bloom_shader,
            entry_point: Some("vs_main"),
            buffers: &[post_fullscreen_vbl2],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &bloom_shader,
            entry_point: Some("fs_blur"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // SSAO: depth pre pass, AO pass, blur.
    let depth_prepass_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Depth Prepass"),
        source: wgpu::ShaderSource::Wgsl(include_str!("depth_prepass.wgsl").into()),
    });
    let depth_unlit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Depth Unlit"),
        source: wgpu::ShaderSource::Wgsl(include_str!("depth_unlit.wgsl").into()),
    });
    let depth_entity_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: std::num::NonZeroU64::new(64),
            },
            count: None,
        }],
        label: Some("depth_entity_bgl"),
    });
    let depth_ssao_entity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Depth SSAO Entity"),
        size: (MAX_LIT_DRAWS as u64) * SHADOW_ENTITY_STRIDE,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let depth_prepass_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&depth_entity_bgl],
        push_constant_ranges: &[],
        label: Some("depth_prepass_pl"),
    });
    let depth_prepass_lit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Depth Prepass Lit"),
        layout: Some(&depth_prepass_pl),
        vertex: wgpu::VertexState {
            module: &depth_prepass_shader,
            entry_point: Some("vs_main"),
            buffers: &[GpuVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &depth_prepass_shader,
            entry_point: Some("fs_main"),
            targets: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let depth_prepass_unlit_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&depth_prepass_unlit_bgl],
        push_constant_ranges: &[],
        label: Some("depth_prepass_unlit_pl"),
    });
    let depth_prepass_unlit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Depth Prepass Unlit"),
        layout: Some(&depth_prepass_unlit_pl),
        vertex: wgpu::VertexState {
            module: &depth_unlit_shader,
            entry_point: Some("vs_main"),
            buffers: &[UnlitVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &depth_unlit_shader,
            entry_point: Some("fs_main"),
            targets: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SSAO"),
        source: wgpu::ShaderSource::Wgsl(include_str!("ssao.wgsl").into()),
    });
    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct SSAOParams {
        proj_inv: [[f32; 4]; 4],
        proj: [[f32; 4]; 4],
        view_inv: [[f32; 4]; 4],
        sample_radius: f32,
        bias: f32,
        intensity: f32,
        max_dist: f32,
    }
    let ssao_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("SSAO Params"),
        size: 208,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let ssao_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(208),
                },
                count: None,
            },
        ],
        label: Some("ssao_bgl"),
    });
    let ssao_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&ssao_bgl],
        push_constant_ranges: &[],
        label: Some("ssao_pl"),
    });
    let post_fullscreen_vbl3 = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
    let ssao_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("SSAO"),
        layout: Some(&ssao_pl),
        vertex: wgpu::VertexState {
            module: &ssao_shader,
            entry_point: Some("vs_main"),
            buffers: &[post_fullscreen_vbl3],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &ssao_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let depth_prepass_lit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &depth_entity_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &depth_ssao_entity_buffer,
                offset: 0,
                size: std::num::NonZeroU64::new(64),
            }),
        }],
        label: Some("depth_prepass_lit_bg"),
    });
    GpuResources {
        uniform_buffer,
        uniform_bind_group,
        pipeline,
        lit_pipeline_layout: pipeline_layout,
        lit_shadow_bind_group_layout,
        unlit_pipeline,
        unlit_storage_buffer,
        unlit_batch_start_buffer,
        unlit_scene_buffer,
        unlit_bind_group,
        depth_prepass_unlit_bind_group,
        sky_pipeline,
        sky_uniform_buffer,
        sky_bind_group,
        sky_vertex_buffer,
        shadow_map_texture,
        shadow_map_view,
        shadow_sampler,
        shadow_light_view_proj_buffer,
        shadow_entity_buffer,
        shadow_pipeline,
        shadow_bind_group_layout,
        shadow_bind_group,
        lit_shadow_bind_group,
        default_normal_map,
        default_normal_map_view,
        normal_map_sampler,
        lit_normal_bind_group,
        post_pipeline,
        post_bind_group_layout,
        post_vertex_buffer,
        post_sampler,
        bloom_extract_pipeline,
        bloom_blur_pipeline,
        bloom_blur_params_buffer,
        depth_prepass_lit_pipeline,
        depth_prepass_unlit_pipeline,
        depth_ssao_entity_buffer,
        depth_prepass_lit_bind_group,
        ssao_pipeline,
        ssao_params_buffer,
    }
}

/// Recreate lit, unlit, and sky pipelines after MSAA change (same layouts, new sample count).
pub(super) fn recreate_msaa_pipelines(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    msaa: u32,
    lit_pipeline_layout: &wgpu::PipelineLayout,
) -> (wgpu::RenderPipeline, wgpu::RenderPipeline, wgpu::RenderPipeline) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Renderer Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(lit_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[GpuVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let unlit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unlit Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("unlit.wgsl").into()),
    });
    let unlit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_STORAGE_SIZE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_BATCH_START_STRIDE),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(UNLIT_SCENE_UNIFORM_SIZE),
                },
                count: None,
            },
        ],
        label: Some("unlit_bind_group_layout"),
    });
    let unlit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[&unlit_bind_group_layout],
        push_constant_ranges: &[],
    });
    let unlit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unlit Render Pipeline"),
        layout: Some(&unlit_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &unlit_shader,
            entry_point: Some("vs_main"),
            buffers: &[UnlitVertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &unlit_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sky Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("sky.wgsl").into()),
    });
    let sky_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(SKY_UNIFORM_SIZE),
            },
            count: None,
        }],
        label: Some("sky_bind_group_layout"),
    });
    let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sky Pipeline Layout"),
        bind_group_layouts: &[&sky_bind_group_layout],
        push_constant_ranges: &[],
    });
    let sky_vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    };
    let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sky Pipeline"),
        layout: Some(&sky_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &sky_shader,
            entry_point: Some("vs_main"),
            buffers: &[sky_vertex_buffer_layout],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &sky_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: msaa,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    (pipeline, unlit_pipeline, sky_pipeline)
}