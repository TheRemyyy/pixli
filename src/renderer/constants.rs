//! Renderer constants (buffer sizes, strides, limits).

/// Maximum unlit instances per frame. Must match array size in unlit.wgsl (array<Instance, N>).
pub const MAX_UNLIT_DRAWS: usize = 4096;
/// Unlit instancing: storage holds model and MVP, 128 bytes per instance.
pub const UNLIT_INSTANCE_SIZE: u64 = 128;
pub const UNLIT_STORAGE_SIZE: u64 = (MAX_UNLIT_DRAWS as u64) * UNLIT_INSTANCE_SIZE;
/// Batch start uniform (u32) per batch, 256 byte stride.
pub const UNLIT_BATCH_START_STRIDE: u64 = 256;
pub const UNLIT_BATCH_START_BUFFER_SIZE: u64 = (MAX_UNLIT_DRAWS as u64) * UNLIT_BATCH_START_STRIDE;

pub const UNLIT_SCENE_UNIFORM_SIZE: u64 = 64;
pub const SKY_UNIFORM_SIZE: u64 = 32;

pub const MAX_LIT_DRAWS: usize = 256;
/// Stride for lit uniform buffer (alignment typically 256).
pub const LIT_UNIFORM_STRIDE: u64 = 512;

/// Shadow map size for directional light.
pub const SHADOW_MAP_SIZE: u32 = 2048;
/// Stride per entity for shadow uniform (at least min_uniform_buffer_offset_alignment, typically 256).
pub const SHADOW_ENTITY_STRIDE: u64 = 256;
