use crate::assets::Assets;
use cgmath::{Matrix4, Vector3};
use crate::renderer::camera::Camera;
use crate::mesh::Vertex;
use crate::conversions::{GpuBuffer, AsBytes};

mod camera;

pub struct Renderer {
    camera: Camera,
    vertex_buf: GpuBuffer,
    index_buf: GpuBuffer,
    bind_group: wgpu::BindGroup,
    projection_view_uniform_buf: GpuBuffer,
    pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device, assets: &Assets) -> Renderer {
        use std::mem;

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let vertex_buf =  GpuBuffer::new(device, wgpu::BufferUsageFlags::VERTEX, &assets.cube.vertices);
        let index_buf = GpuBuffer::new(device, wgpu::BufferUsageFlags::INDEX, &assets.cube.indices);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStageFlags::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStageFlags::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer,
                },
                /* Not needed, no texture
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStageFlags::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
                */
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            r_address_mode: wgpu::AddressMode::ClampToEdge,
            s_address_mode: wgpu::AddressMode::ClampToEdge,
            t_address_mode: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            max_anisotropy: 0,
            compare_function: wgpu::CompareFunction::Always,
            border_color: wgpu::BorderColor::TransparentBlack,
        });

        let aspect_ratio = sc_desc.width as f32 / sc_desc.height as f32;
        let camera = Camera::new(
            cgmath::Deg(45.0),
            aspect_ratio,
            1.0,
            10.0,
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
        );

        /*
        let mx_total = camera.projection_view();
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let projection_view_uniform_buf = device
            .create_buffer_mapped(
                16,
                wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            )
            .fill_from_slice(mx_ref);



        let normal_view = camera.normal_view();
        let normal_view_ref: &[f32; 9] = normal_view.as_ref();
        let normal_buf = device
            .create_buffer_mapped(
                9,
                wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            )
            .fill_from_slice(normal_view_ref);
            */

        let projection_view_uniform_buf = GpuBuffer::from_bytes(
            device,
            wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            camera.projection_view().as_bytes()
        );

        let normal_buf = GpuBuffer::from_bytes(
            device,
            wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            camera.normal_view().as_bytes()
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                projection_view_uniform_buf.binding(0),
                normal_buf.binding(1),
                /*
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                */
            ],
        });

        let vs_module = device.create_shader_module(&assets.vertex_shader);
        let fs_module = device.create_shader_module(&assets.fragment_shader);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::PipelineStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: wgpu::PipelineStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            },
            rasterization_state: wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color: wgpu::BlendDescriptor::REPLACE,
                alpha: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWriteFlags::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[Vertex::buffer_descriptor()],
            sample_count: 1,
        });

        // Done
        let init_command_buf = init_encoder.finish();
        device.get_queue().submit(&[init_command_buf]);

        Renderer {
            camera,
            vertex_buf,
            index_buf,
            bind_group,
            projection_view_uniform_buf,
            pipeline,
        }
    }

    pub fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device) {
        self.camera.set_aspect(sc_desc.width as f32 / sc_desc.height as f32);

        let temp_buf = GpuBuffer::from_bytes(
            device,
            wgpu::BufferUsageFlags::TRANSFER_SRC,
            self.camera.projection_view().as_bytes()
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        temp_buf.copy_to_buffer(&mut encoder, &self.projection_view_uniform_buf);
        device.get_queue().submit(&[encoder.finish()]);
    }

    pub fn render(&mut self, frame: &wgpu::SwapChainOutput, device: &mut wgpu::Device) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group);
            rpass.set_index_buffer(&self.index_buf.buffer(), 0);
            rpass.set_vertex_buffers(&[(&self.vertex_buf.buffer(), 0)]);
            rpass.draw_indexed(0..self.index_buf.len, 0, 0..1);
        }

        device.get_queue().submit(&[encoder.finish()]);
    }
}
