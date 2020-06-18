use winit::{
    window::Window,
};

use super::shader::ShaderManager;
use super::bindings::PipelineManager;

pub struct GPUInterfaces {
    pub _instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub _adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

#[derive(Default)]
pub struct ResourceManagers
{
    pub shader_manager : ShaderManager,
    pub pipeline_manager : PipelineManager, 
}

impl GPUInterfaces {
    pub async fn new(window: &Window, swapchain_format: wgpu::TextureFormat) -> Self {
        let size = window.inner_size();

        let _instance = wgpu::Instance::new();
        let surface = unsafe { _instance.create_surface(window) };
        let _adapter = _instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        let (device, queue) = _adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: true,
                    },
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            _instance,
            surface,
            _adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }
}

