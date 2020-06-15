//module
#[cfg(target_arch = "wasm32")] mod wasm;
#[cfg(target_arch = "wasm32")] pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))] mod native;
#[cfg(not(target_arch = "wasm32"))] pub use native::*;

//imports
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};


use super::graphics::api;

#[derive(Debug)]
pub enum Platform
{
    NATIVE,
    BROWSER,
}
/*

pub async fn run<T>(event_loop: EventLoop<()>, window: Window, swapchain_format: wgpu::TextureFormat) {
    let gpu_interfaces = api::GPUInterfaces::new(&window, swapchain_format).await;

    let mut app = T::new(&window, gpu_interfaces).await;

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
                            app.gpu_interfaces.resize(**new_inner_size);
                            app.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                app.update();
                app.render();
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


pub fn entry_point<T>( app_name:&str, config : application::ApplicationConfig)
{
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(app_name);

    let plat_str = format!("Start up on platform: {:?}", core::get_platform());
    core::to_console(&plat_str[..]);

    #[cfg(not(target_arch = "wasm32"))]
    {
        //env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        // Since main can't be async, we're going to need to block
        futures::executor::block_on(run::<T>(event_loop, window, wgpu::TextureFormat::Bgra8Unorm));
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
        wasm_bindgen_futures::spawn_local(run::<T>(event_loop, window, wgpu::TextureFormat::Bgra8Unorm));
    }


}
*/