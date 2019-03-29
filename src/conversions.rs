use cgmath::{Matrix3, Matrix4};

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

