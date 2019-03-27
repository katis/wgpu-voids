use cgmath::{Deg, Matrix4, Point3, Vector3};

pub struct Camera {
    fov_y: Deg<f32>,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    position: Point3<f32>,
    target: Point3<f32>,
}

impl Camera {
    pub fn new(
        fov_y: Deg<f32>,
        aspect_ratio: f32,
        near: f32,
        far: f32,
        position: Point3<f32>,
        target: Point3<f32>,
    ) -> Camera {
        Camera {
            fov_y,
            aspect_ratio,
            near,
            far,
            position,
            target,
        }
    }

    pub fn set_aspect(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn projection(&self) -> Matrix4<f32> {
        cgmath::perspective(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at(
            self.position,
            self.target,
            -Vector3::unit_z(),
        )
    }

    pub fn translate(&mut self, movement: Vector3<f32>) {
        self.position += movement;
    }

    pub fn projection_view(&self) -> Matrix4<f32> {
        self.projection() * self.view()
    }
}