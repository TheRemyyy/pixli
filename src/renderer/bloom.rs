//! Bloom: extract bright pixels, blur, composite.

use super::pass_common::BlurParams;
#[allow(clippy::too_many_arguments)]
pub(super) fn run_bloom(
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    size: (u32, u32),
    enable_bloom: bool,
    extract_pipe: Option<&wgpu::RenderPipeline>,
    blur_pipe: Option<&wgpu::RenderPipeline>,
    bind_groups: Option<&(wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup)>,
    params_buf: Option<&wgpu::Buffer>,
    bloom_a: Option<&wgpu::Texture>,
    bloom_b: Option<&wgpu::Texture>,
    pvb: Option<&wgpu::Buffer>,
) {
    if enable_bloom {
        if let (
            Some(extract_pipe),
            Some(blur_pipe),
            Some((bg_extract, bg_blur_a, bg_blur_b)),
            Some(params_buf),
            Some(bloom_a),
            Some(bloom_b),
            Some(pvb),
        ) = (
            extract_pipe,
            blur_pipe,
            bind_groups,
            params_buf,
            bloom_a,
            bloom_b,
            pvb,
        ) {
            let (w, _h) = size;
            let texel = [1.0 / w as f32, 1.0 / size.1 as f32];
            let bloom_a_view = bloom_a.create_view(&Default::default());
            let bloom_b_view = bloom_b.create_view(&Default::default());

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Extract"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom_a_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(extract_pipe);
            pass.set_bind_group(0, bg_extract, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 1,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Blur H"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom_b_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(blur_pipe);
            pass.set_bind_group(0, bg_blur_a, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 0,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom Blur V"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom_a_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(blur_pipe);
            pass.set_bind_group(0, bg_blur_b, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);
        }
    } else if let Some(bloom_a) = bloom_a {
        let bloom_a_view = bloom_a.create_view(&Default::default());
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Bloom Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &bloom_a_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
