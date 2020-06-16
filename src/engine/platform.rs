//module
#[cfg(target_arch = "wasm32")] mod wasm;
#[cfg(target_arch = "wasm32")] pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))] mod native;
#[cfg(not(target_arch = "wasm32"))] pub use native::*;

//imports
use winit::window::Window;

#[derive(Debug)]
pub enum Platform
{
    NATIVE,
    BROWSER,
}

use super::graphics::api;

pub struct EngineRuntime {
    pub gpu_interfaces: api::GPUInterfaces,
    pub resource_managers: api::ResourceManagers,
}

impl EngineRuntime {
    pub async fn new(window: &Window, swapchain_format: wgpu::TextureFormat) -> Self {
        let gpu_interfaces = api::GPUInterfaces::new(window,swapchain_format).await;
        Self{
            gpu_interfaces,
            resource_managers: api::ResourceManagers::new(),
        }
    }
}