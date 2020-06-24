
use winit::{event::*, window::Window};

use rust_sandbox::engine::graphics;
use rust_sandbox::engine::handle;
use rust_sandbox::engine::platform;

use async_trait::async_trait;

pub struct HelloTriangle {
    engine_runtime: platform::EngineRuntime,
    render_pipeline_handle: handle::ResourceHandle,
    camera: graphics::camera::Camera,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    size: winit::dpi::PhysicalSize<u32>,
    color: f64,
    camera_controller: graphics::camera::CameraControllerFPS,
    per_frame_data: graphics::FrameData,
    time_stamp: u64,
    delta_time: u64
}

#[async_trait(?Send)]
impl platform::Application for HelloTriangle {
    async fn new(window: &Window, mut engine_runtime: platform::EngineRuntime) -> Self {
        let size = window.inner_size();

        let color = 0.0;
        let gpu_interfaces = &engine_runtime.gpu_interfaces;

        let camera = graphics::camera::Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (3.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: gpu_interfaces.sc_desc.width as f32 / gpu_interfaces.sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = graphics::camera::CameraControllerFPS::new(10.0);

        let mut per_frame_data = graphics::FrameData::new();
        per_frame_data.update_view_proj(&camera);

        let uniform_buffer = gpu_interfaces.device.create_buffer_with_data(
            bytemuck::cast_slice(&[per_frame_data]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let layout_handle = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_binding_group("resources/hello-triangle.bg", gpu_interfaces)
            .await;

        let render_pipeline_handle = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_pipeline(
                "resources/hello-triangle.pipeline",
                &mut engine_runtime.resource_managers.shader_manager,
                &engine_runtime.gpu_interfaces,
                wgpu::TextureFormat::Depth32Float
            )
            .await;

        let bg_layout = engine_runtime
            .resource_managers
            .pipeline_manager
            .get_bind_group_from_handle(layout_handle);

        let uniform_bind_group =
            gpu_interfaces
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bg_layout.unwrap(),
                    bindings: &[wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniform_buffer,
                            // FYI: you can share a single buffer between bindings.
                            range: 0..std::mem::size_of_val(&per_frame_data) as wgpu::BufferAddress,
                        },
                    }],
                    label: Some("uniform_bind_group"),
                });


        Self {
            engine_runtime,
            render_pipeline_handle,
            camera,
            uniform_buffer,
            uniform_bind_group,
            size,
            color,
            camera_controller,
            per_frame_data,
            time_stamp:platform::core::get_time_in_micro(),
            delta_time: 0,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.engine_runtime.gpu_interfaces.resize(new_size);
        self.size = new_size;
    }

    // input() won't deal with GPU code, so it can be synchronous
    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self, command_buffers : &mut Vec<wgpu::CommandBuffer> ) {
        //let us update time
        let curr_time = platform::core::get_time_in_micro();
        self.delta_time = curr_time - self.time_stamp;
        self.time_stamp = curr_time;

        //not doing anything here yet
        self.camera_controller.update_camera(&mut self.camera, self.delta_time);
        self.per_frame_data.update_view_proj(&self.camera);

        // Copy operation's are performed on the gpu, so we'll need
        // a CommandEncoder for that
        let mut encoder = self
            .engine_runtime
            .gpu_interfaces
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("update encoder"),
            });

        let staging_buffer = self
            .engine_runtime
            .gpu_interfaces
            .device
            .create_buffer_with_data(
                bytemuck::cast_slice(&[self.per_frame_data]),
                wgpu::BufferUsage::COPY_SRC,
            );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<graphics::FrameData>() as wgpu::BufferAddress,
        );

        command_buffers.push(encoder.finish());
    }

    fn render(&mut self, mut command_buffers: Vec<wgpu::CommandBuffer> ) {
        //first we need to get the frame we can use from the swap chain so we can render to it
        let frame = self
            .engine_runtime
            .gpu_interfaces
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        //this is the command buffer we use to record commands
        let mut encoder = self
            .engine_runtime
            .gpu_interfaces
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            let render_pipeline = self
                .engine_runtime
                .resource_managers
                .pipeline_manager
                .get_pipeline_from_handle(&self.render_pipeline_handle);
            render_pass.set_pipeline(&render_pipeline.unwrap());
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        self.color += 0.001;
        if self.color > 1.0 {
            self.color = 0.0;
        }

        //self.queue.submit(&[
        //    encoder.finish()
        //]);
        command_buffers.push(encoder.finish());
        self.engine_runtime
            .gpu_interfaces
            .queue
            .submit(command_buffers);
    }
}

fn main()
{
    platform::run_application::<HelloTriangle>("HelloTriangle v1.0.0");
}