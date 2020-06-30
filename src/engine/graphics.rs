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
    pub view_proj: cgmath::Matrix4<f32>,
    pub view_proj_inverse: cgmath::Matrix4<f32>,
    pub screen_size: cgmath::Vector2<u32>
}

impl FrameData {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity(),
            view_proj_inverse: cgmath::Matrix4::identity(),
            screen_size: cgmath::Vector2::<u32>::new(0,0),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix();
        self.view_proj_inverse = camera.build_proj_inverse();
    }
}

unsafe impl bytemuck::Pod for FrameData {}
unsafe impl bytemuck::Zeroable for FrameData {}

