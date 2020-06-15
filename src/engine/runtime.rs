use winit::{
    window::Window,
};

use super::graphics::api;

pub struct Runtime {
    pub gpu_interfaces: api::GPUInterfaces,
}

impl Runtime {
    pub async fn new(window: &Window, swapchain_format: wgpu::TextureFormat) -> Self {
        let gpu_interfaces = api::GPUInterfaces::new(window,swapchain_format).await;
        Self{
            gpu_interfaces,
        }
    }
}
