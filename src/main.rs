use std::io::{self, Write};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

/*
fn run(event_loop: EventLoop<()>, window: Window)
{
    let mut state = State::new(&window);
    //TODO this is a closure, investigate
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            state.render();
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        }
        //we first the event to the input and if is not handled ( meaning returning false)
        //we go down to normal match statement
        if window_id == window.id() => if !state.input(event) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input,
                ..
            } => {
                match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &mut so w have to dereference it twice
                state.resize(**new_inner_size);
            }
            _ => {}
        }
    }
    _ => {}
    });


}
*/
struct State {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    size: winit::dpi::PhysicalSize<u32>,
    color: f64
}
impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new();
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            //TODO hardocded
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let color = 0.0;

        Self {
            instance,
            surface,
            adapter,
            device: device,
            queue: queue,
            sc_desc,
            swap_chain,
            size,
            color,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    // input() won't deal with GPU code, so it can be synchronous
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        //not doing anything here yet
    }

    fn render(&mut self) {
        //first we need to get the frame we can use from the swap chain so we can render to it
        let frame = self
            .swap_chain
            .get_next_frame()
            .expect("Failed to acquire next swap chain texture")
            .output;

        //this is the command buffer we use to record commands
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: self.color,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }
       self. color += 0.001;
        if self.color > 1.0 {
            self.color = 0.0;
        }
        self.queue.submit(Some(encoder.finish()));
    }
}

pub async fn run(event_loop: EventLoop<()>, window: Window, swapchain_format: wgpu::TextureFormat) {
    let size = window.inner_size();

    let mut state = State::new(&window).await;

    let mut color: f64 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        // force ownership by the closure
        let _ = (&state,);

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                state.sc_desc.width = size.width;
                state.sc_desc.height = size.height;
                state.swap_chain = state
                    .device
                    .create_swap_chain(&state.surface, &state.sc_desc);
            }
            Event::RedrawRequested(_) => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
            }
            Event::RedrawEventsCleared => {
                /*
                let frame = state.swap_chain
                    .get_next_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;
                //this is the command buffer we use to record commands
                let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: color,
                                a: 1.0,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                }
                color += 0.001;
                if color > 1.0
                {color = 0.0;}
                state.queue.submit(Some(encoder.finish()));
                */
                state.render();
            }
            _ => {
                //println!("{:?}",event)
            }
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Rust Sandbox v0.0.1");

    //env_logger::init();
    // Temporarily avoid srgb formats for the swapchain on the web
    futures::executor::block_on(run(event_loop, window, wgpu::TextureFormat::Bgra8UnormSrgb));

    // Since main can't be async, we're going to need to block
}
