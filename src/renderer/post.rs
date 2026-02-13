//! Post-process: composite scene, bloom, SSAO and tone mapping to swapchain.

pub(super) fn run_post_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    post_pipeline: &wgpu::RenderPipeline,
    post_bind_group: &wgpu::BindGroup,
    post_vertex_buffer: &wgpu::Buffer,
) {
    let mut post_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Post Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });
    post_pass.set_pipeline(post_pipeline);
    post_pass.set_bind_group(0, post_bind_group, &[]);
    post_pass.set_vertex_buffer(0, post_vertex_buffer.slice(..));
    post_pass.draw(0..3, 0..1);
}
