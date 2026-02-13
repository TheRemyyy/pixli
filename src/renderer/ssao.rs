//! SSAO: screen-space ambient occlusion pass and blur.

use super::pass_common::BlurParams;
use crate::math::Mat4;

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

#[allow(clippy::too_many_arguments)]
pub(super) fn run_ssao_pass(
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    size: (u32, u32),
    proj: Mat4,
    view: Mat4,
    enable_ssao: bool,
    ssao_pipe: Option<&wgpu::RenderPipeline>,
    ssao_params_buf: Option<&wgpu::Buffer>,
    ssao_bind_groups: Option<&(wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup)>,
    blur_pipe: Option<&wgpu::RenderPipeline>,
    blur_params_buf: Option<&wgpu::Buffer>,
    ssao_texture_view: Option<&wgpu::TextureView>,
    ssao_blur_view: Option<&wgpu::TextureView>,
    pvb: Option<&wgpu::Buffer>,
    ssao_texture: Option<&wgpu::Texture>,
) {
    if enable_ssao {
        if let (
            Some(ssao_pipe),
            Some(ssao_params_buf),
            Some((ssao_bg, ssao_blur_in, ssao_blur_out)),
            Some(blur_pipe),
            Some(blur_params_buf),
            Some(ssao_view),
            Some(ssao_blur_view),
            Some(pvb),
        ) = (
            ssao_pipe,
            ssao_params_buf,
            ssao_bind_groups,
            blur_pipe,
            blur_params_buf,
            ssao_texture_view,
            ssao_blur_view,
            pvb,
        ) {
            let (w, h) = size;
            let proj_inv = proj.inverse();
            let view_inv = view.inverse();
            let ssao_params = SSAOParams {
                proj_inv: proj_inv.to_cols_array(),
                proj: proj.to_cols_array(),
                view_inv: view_inv.to_cols_array(),
                sample_radius: 1.0,
                bias: 0.025,
                intensity: 0.8,
                max_dist: 100.0,
            };
            queue.write_buffer(ssao_params_buf, 0, bytemuck::bytes_of(&ssao_params));
            let texel = [1.0 / w as f32, 1.0 / h as f32];

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ssao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(ssao_pipe);
            pass.set_bind_group(0, ssao_bg, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                blur_params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 1,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Blur H"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ssao_blur_view,
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
            pass.set_bind_group(0, ssao_blur_in, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                blur_params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 0,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Blur V"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ssao_view,
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
            pass.set_bind_group(0, ssao_blur_out, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                blur_params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 1,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Blur H2"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ssao_blur_view,
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
            pass.set_bind_group(0, ssao_blur_in, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);

            queue.write_buffer(
                blur_params_buf,
                0,
                bytemuck::bytes_of(&BlurParams {
                    texel_size: texel,
                    is_horizontal: 0,
                    _pad: 0,
                }),
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Blur V2"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: ssao_view,
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
            pass.set_bind_group(0, ssao_blur_out, &[]);
            pass.set_vertex_buffer(0, pvb.slice(..));
            pass.draw(0..3, 0..1);
        }
    } else if let Some(ssao_tex) = ssao_texture {
        let ssao_view = ssao_tex.create_view(&Default::default());
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SSAO Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &ssao_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
