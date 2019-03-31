use crate::assets::Assets;
use cgmath::{Matrix4, Vector3};
use crate::renderer::camera::Camera;
use crate::model_data::Vertex;
use crate::conversions::{GpuBuffer, AsBytes};

mod camera;

pub struct Renderer {
    camera: Camera,
    vertex_buf: GpuBuffer,
    index_buf: GpuBuffer,
    bind_group: wgpu::BindGroup,
    projection_view: GpuBuffer,
    normal_view: GpuBuffer,
    pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Light {
    position: Vector3<f32>,
    intensities: Vector3<f32>,
}

impl Renderer {
    pub fn init(sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device, assets: &Assets) -> Renderer {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        let light = Light {
            position: Vector3::new(10.0, 0.0, 3.0),
            intensities: Vector3::new(2.0, 2.0, 2.0),
        };

        let cube = assets.models.find("cube").unwrap();

        let vertex_buf = GpuBuffer::new(device, wgpu::BufferUsageFlags::VERTEX, &cube.vertices);
        let index_buf = GpuBuffer::new(device, wgpu::BufferUsageFlags::INDEX, &cube.indices);

        let light_buf = GpuBuffer::new(
            device,
            wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            &[light],
        );

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
                wgpu::BindGroupLayoutBinding {
                    binding: 2,
                    visibility: wgpu::ShaderStageFlags::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 3,
                    visibility: wgpu::ShaderStageFlags::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture,
                },
                wgpu::BindGroupLayoutBinding {
                    binding: 4,
                    visibility: wgpu::ShaderStageFlags::FRAGMENT,
                    ty: wgpu::BindingType::Sampler,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        // Create texture

        let texture_extent = cube.texture.extent();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            array_size: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::TRANSFER_DST,
        });
        let texture_view = texture.create_default_view();
        let temp_buf = device
            .create_buffer_mapped(cube.texture.pixels.len(), wgpu::BufferUsageFlags::TRANSFER_SRC)
            .fill_from_slice(&cube.texture.pixels);

        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * cube.texture.width,
                image_height: cube.texture.height,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                level: 0,
                slice: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            texture_extent,
        );

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
        let mut camera = Camera::new(
            cgmath::Deg(45.0),
            aspect_ratio,
            1.0,
            10.0,
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
        );

        let projection_view = GpuBuffer::from_bytes(
            device,
            wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            camera.projection_view().as_bytes(),
        );

        let normal_view_buf = GpuBuffer::from_byte_slices(
            device,
            wgpu::BufferUsageFlags::UNIFORM | wgpu::BufferUsageFlags::TRANSFER_DST,
            &[
                camera.view().as_bytes(),
                camera.normal_view().as_bytes(),
            ],
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                projection_view.binding(0),
                normal_view_buf.binding(1),
                light_buf.binding(2),
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let vertex_shader = assets.shaders.find("vertex").unwrap();
        let fragment_shader = assets.shaders.find("fragment").unwrap();

        let vs_module = device.create_shader_module(vertex_shader);
        let fs_module = device.create_shader_module(fragment_shader);

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
            projection_view,
            normal_view: normal_view_buf,
            pipeline,
        }
    }

    pub fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device) {
        self.camera.set_aspect(sc_desc.width as f32 / sc_desc.height as f32);
        self.update_camera(device);
    }

    pub fn move_camera(&mut self, device: &mut wgpu::Device, movement: Vector3<f32>) {
        self.camera.translate(movement);
        self.update_camera(device);
    }

    fn update_camera(&mut self, device: &mut wgpu::Device) {
         let projection_view_src = GpuBuffer::from_bytes(
            device,
            wgpu::BufferUsageFlags::TRANSFER_SRC,
            self.camera.projection_view().as_bytes(),
        );

        let normal_view_src = GpuBuffer::from_byte_slices(
            device,
            wgpu::BufferUsageFlags::TRANSFER_SRC,
            &[
                self.camera.view().as_bytes(),
                self.camera.normal_view().as_bytes(),
            ],
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        projection_view_src.copy_to_buffer(&mut encoder, &self.projection_view);
        normal_view_src.copy_to_buffer(&mut encoder, &self.normal_view);
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
