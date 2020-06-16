//module
#[cfg(target_arch = "wasm32")] mod wasm;
#[cfg(target_arch = "wasm32")] pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))] mod native;
#[cfg(not(target_arch = "wasm32"))] pub use native::*;

//imports
use winit::{event::*, window::Window};
use async_trait::async_trait;

use super::graphics::api;

#[derive(Debug)]
pub enum Platform
{
    NATIVE,
    BROWSER,
}

pub struct EngineRuntime {
    pub gpu_interfaces: api::GPUInterfaces,
    pub resource_managers: api::ResourceManagers,
}

#[async_trait(?Send)]
pub trait Application: 'static + Sized {
    async fn new(window: &Window, engine_runtime: EngineRuntime) -> Self;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self);
    fn render(&mut self);
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