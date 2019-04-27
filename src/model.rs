use crate::conversions::{GpuBuffer};
use cgmath::{Decomposed, Deg, Matrix4, Quaternion, Rotation3, Vector3, Vector4};
use std::mem::size_of;
use wgpu::BufferUsageFlags;

pub struct ModelGroup {
    pub name: String,
    pub index_buf: GpuBuffer,
    pub vertex_buf: GpuBuffer,
    pub bind_group: wgpu::BindGroup,
    pub models: Vec<Model>,
    mvp_buffer: Option<GpuBuffer>,
}

impl ModelGroup {
    pub fn new(
        name: impl Into<String>,
        index_buf: GpuBuffer,
        vertex_buf: GpuBuffer,
        bind_group: wgpu::BindGroup,
    ) -> ModelGroup {
        ModelGroup {
            name: name.into(),
            index_buf,
            vertex_buf,
            bind_group,
            models: Vec::new(),
            mvp_buffer: None,
        }
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn update_mvp_buffer(&mut self, device: &wgpu::Device) {
         self.mvp_buffer = Some(GpuBuffer::from_transformed_slice(
            device,
            BufferUsageFlags::TRANSFER_SRC,
            &self.models,
            |model| model.model_matrix(),
        ));
    }

    pub fn mvp_buffer(&self) -> &wgpu::Buffer {
        match &self.mvp_buffer {
            Some(buf) => buf.buffer(),
            None => unimplemented!(),
        }
    }

    pub fn buffer_descriptor() -> wgpu::VertexBufferDescriptor<'static> {
        wgpu::VertexBufferDescriptor {
            stride: size_of::<Matrix4<f32>>() as u32,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 3,
                    format: wgpu::VertexFormat::Float4,
                    offset: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 4,
                    format: wgpu::VertexFormat::Float4,
                    offset: (size_of::<f32>() * 4) as u32,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 5,
                    format: wgpu::VertexFormat::Float4,
                    offset: (size_of::<f32>() * 4 * 2) as u32,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 6,
                    format: wgpu::VertexFormat::Float4,
                    offset: (size_of::<f32>() * 4 * 3) as u32,
                },
            ],
        }
    }
}

pub struct Model {
    transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
}

impl Model {
    pub fn new(position: Vector3<f32>) -> Model {
        Model {
            transform: Decomposed {
                scale: 1.0,
                rot: Quaternion::from_angle_y(Deg(0.0f32)),
                disp: position,
            },
        }
    }

    pub fn translate(&mut self, movement: Vector3<f32>) {
        self.transform.disp += movement;
    }

    pub fn model_matrix(&self) -> Matrix4<f32> {
        self.transform.into()
    }
}
