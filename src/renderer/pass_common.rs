//! Shared types for render passes (blur, post, etc.).

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct BlurParams {
    pub texel_size: [f32; 2],
    pub is_horizontal: u32,
    pub _pad: u32,
}
