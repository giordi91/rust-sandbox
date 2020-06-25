pub mod shader;
pub mod camera;
pub mod api;
pub mod bindings;
pub mod model;
pub mod texture;
pub mod buffer;



#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
pub struct FrameData {
    view_proj: cgmath::Matrix4<f32>,
}

impl FrameData {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}

unsafe impl bytemuck::Pod for FrameData {}
unsafe impl bytemuck::Zeroable for FrameData {}

