//module
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

//imports
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use async_trait::async_trait;

use super::graphics::api;

use log;


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

#[async_trait(? Send)]
pub trait Application: 'static + Sized {
    async fn new(window: &Window, engine_runtime: EngineRuntime) -> Self;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self,command_buffers: &mut Vec<wgpu::CommandBuffer>);
    fn render(&mut self,command_buffers: Vec<wgpu::CommandBuffer>);
}


impl EngineRuntime {
    pub async fn new(window: &Window, swapchain_format: wgpu::TextureFormat) -> Self {
        let gpu_interfaces = api::GPUInterfaces::new(window, swapchain_format).await;
        Self {
            gpu_interfaces,
            resource_managers: api::ResourceManagers::default(),
        }
    }
}


async fn run<T: Application>(
    event_loop: EventLoop<()>,
    window: Window,
    swapchain_format: wgpu::TextureFormat,
) {
    //instantiating the engine innerworking and move it to the application
    let engine_runtime = EngineRuntime::new(&window, swapchain_format).await;

    let mut app = T::new(&window, engine_runtime).await;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        //This match statement is still slightly confusing for me, need to investigate a
        //bit more
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => println!("unhandled input {:?}", input),
                        },
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            app.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                let mut buffers = Vec::new();
                app.update(&mut buffers);
                app.render(buffers);

            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}


pub fn run_application<T: Application>(window_title: &str)
{
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(window_title);

    let plat_str = format!("Start up on platform: {:?}", core::get_platform());
    core::to_console(&plat_str[..]);

    #[cfg(not(target_arch = "wasm32"))]
        {
            env_logger::init();
            // Temporarily avoid srgb formats for the swapchain on the web
            // Since main can't be async, we're going to need to block
            futures::executor::block_on(run::<T>(
                event_loop,
                window,
                wgpu::TextureFormat::Bgra8Unorm,
            ));
        }

    #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            //console_log::init().expect("could not initialize logger");
            use winit::platform::web::WindowExtWebSys;
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
            //let performance  =  web_sys::window().unwrap().performance().unwrap();
            //core::get_time_callback = Some(Box::new(move || { performance.now()}));
            wasm_bindgen_futures::spawn_local(run::<T>(
                event_loop,
                window,
                wgpu::TextureFormat::Bgra8Unorm,
            ));
        }
}

