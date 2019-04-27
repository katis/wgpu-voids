use cgmath::{Matrix3, Matrix4};
use std::collections::Bound;
use std::mem::size_of;
use std::ops::Range;

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for Matrix4<f32> {
    fn as_bytes(&self) -> &[u8] {
        let slice: &[f32; 16] = self.as_ref();
        unsafe {
            ::std::slice::from_raw_parts(slice.as_ptr() as *const u8, 16 * 4)
        }
    }
}

impl AsBytes for Matrix3<f32> {
    fn as_bytes(&self) -> &[u8] {
        let slice: &[f32; 9] = self.as_ref();
        unsafe {
            ::std::slice::from_raw_parts(slice.as_ptr() as *const u8, 9 * 4)
        }
    }
}

pub struct GpuBuffer {
    pub len: u32,
    buffer: wgpu::Buffer,
}

impl GpuBuffer {
    pub fn new<T: 'static + Copy>(
        device: &wgpu::Device,
        buffer_usage: wgpu::BufferUsageFlags,
        contents: &[T],
    ) -> GpuBuffer {
        let buffer = device
            .create_buffer_mapped(contents.len(), buffer_usage)
            .fill_from_slice(contents);
        GpuBuffer {
            len: contents.len() as u32,
            buffer,
        }
    }

    pub fn from_single<T: 'static + Copy>(
        device: &wgpu::Device,
        buffer_usage: wgpu::BufferUsageFlags,
        item: T,
    ) -> GpuBuffer {
        let builder = device.create_buffer_mapped(1, buffer_usage);
        builder.data[0] = item;
        GpuBuffer {
            len: size_of::<T>() as u32,
            buffer: builder.finish(),
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        buffer_usage: wgpu::BufferUsageFlags,
        bytes: &[u8],
    ) -> GpuBuffer {
        let buffer = device
            .create_buffer_mapped(bytes.len(), buffer_usage)
            .fill_from_slice(bytes);
        GpuBuffer {
            len: bytes.len() as u32,
            buffer,
        }
    }

    pub fn from_byte_slices(
        device: &wgpu::Device,
        buffer_usage: wgpu::BufferUsageFlags,
        bytes: &[&[u8]],
    ) -> GpuBuffer {
        let len: usize = bytes.iter().map(|s| s.len()).sum();
        let builder = device.create_buffer_mapped(len, buffer_usage);
        let mut i: usize = 0;
        for slice in bytes {
            let end = i + slice.len();
            builder.data[i..end].copy_from_slice(slice);
            i += slice.len();
        }

        GpuBuffer {
            len: len as u32,
            buffer: builder.finish(),
        }
    }

    pub fn from_transformed_slice<A, B: 'static + Copy>(
        device: &wgpu::Device,
        buffer_usage: wgpu::BufferUsageFlags,
        items: &[A],
        transform: impl Fn(&A) -> B,
    ) -> GpuBuffer {
        let builder: wgpu::CreateBufferMapped<'_, B> =
            device.create_buffer_mapped(items.len(), buffer_usage);

        for (i, item) in items.iter().map(transform).enumerate() {
            builder.data[i] = item;
        }

        GpuBuffer {
            len: items.len() as u32,
            buffer: builder.finish(),
        }
    }

    pub fn binding(&self, binding_index: u32) -> wgpu::Binding {
        wgpu::Binding {
            binding: binding_index,
            resource: wgpu::BindingResource::Buffer {
                buffer: &self.buffer,
                range: 0..self.len,
            },
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, destination: &GpuBuffer) {
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &destination.buffer, 0, self.len);
    }
}
