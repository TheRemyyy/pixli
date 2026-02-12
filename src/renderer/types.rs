//! Renderer types: vertices, uniforms, settings.

use bytemuck::{Pod, Zeroable};

/// Vertex for unlit shader (position and color only, no lighting).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UnlitVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl UnlitVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UnlitVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

static UNLIT_MESH_ID_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Unlit mesh data (position and color). Upload via `Renderer::upload_unlit_mesh`, then use `UnlitMeshRef(id)` on entities.
#[derive(Clone)]
pub struct UnlitMesh {
    id: u64,
    pub vertices: Vec<UnlitVertex>,
}

impl UnlitMesh {
    pub fn from_vertices(vertices: Vec<UnlitVertex>) -> Self {
        Self {
            id: UNLIT_MESH_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            vertices,
        }
    }
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Component: reference to an uploaded unlit mesh. Use the id returned by `Renderer::upload_unlit_mesh`.
#[derive(Clone, Copy, Debug)]
pub struct UnlitMeshRef(pub u64);

/// GPU vertex format (full lighting and tangent for normal mapping).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl GpuVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x3,
            3 => Float32x2,
            4 => Float32x4,
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

/// Unlit scene uniform (camera and fog).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct UnlitSceneUniform {
    pub camera_pos: [f32; 3],
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 3],
    pub _pad: f32,
}

/// Sky gradient uniform (top and bottom color).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SkyUniform {
    pub top_color: [f32; 3],
    pub _pad0: f32,
    pub bottom_color: [f32; 3],
    pub _pad1: f32,
}

/// Uniform data sent to GPU for lit pipeline.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub view_pos: [f32; 4],
    pub color: [f32; 4],
    pub ambient: [f32; 4],
    pub light_dir: [f32; 4],
    pub light_color: [f32; 4],
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 4],
    pub emission_strength: f32,
    pub _pad: [f32; 2],
}

/// Mesh handle for GPU resources.
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

/// Graphics configuration (MSAA, post process, resolution, and related options).
#[derive(Clone, Debug)]
pub struct GraphicsSettings {
    pub msaa_samples: u32,
    pub render_scale: f32,
    pub enable_sky: bool,
    pub enable_shadows: bool,
    pub enable_ssao: bool,
    pub enable_bloom: bool,
    pub enable_fog: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            msaa_samples: 1,
            render_scale: 1.0,
            enable_sky: false,
            enable_shadows: false,
            enable_ssao: false,
            enable_bloom: false,
            enable_fog: false,
        }
    }
}
