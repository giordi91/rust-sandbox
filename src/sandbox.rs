use winit::{
    event::*,
    window::Window,
};

use rust_sandbox::engine::graphics;
use rust_sandbox::engine::graphics::api;
use rust_sandbox::engine::graphics::shader;
use rust_sandbox::engine::runtime;

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait App : 'static + Sized
{
    async fn new(window: &Window, engine_runtime: runtime::Runtime) -> Self; 
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>); 
    fn input(&mut self, event: &WindowEvent) -> bool; 
    fn update(&mut self);
    fn render(&mut self); 
}


#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
pub struct Uniforms {
    view_proj: cgmath::Matrix4<f32>,
}

impl Uniforms {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &graphics::camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}


pub struct Sandbox {
    pub engine_runtime : runtime::Runtime,
    render_pipeline: wgpu::RenderPipeline,
    camera: graphics::camera::Camera,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    size: winit::dpi::PhysicalSize<u32>,
    color: f64,
    shader_manager: shader::ShaderManager,
    camera_controller: graphics::camera::CameraControllerFPS,
    uniforms: Uniforms,
}

#[async_trait(?Send)]
impl App for Sandbox
{
    async fn new(window: &Window, engine_runtime: runtime::Runtime) -> Self {
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

        let camera_controller = graphics::camera::CameraControllerFPS::new(0.02);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = gpu_interfaces.device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            gpu_interfaces
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    }],
                    label: Some("uniform_bind_group_layout"),
                });

        let uniform_bind_group =
            gpu_interfaces
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &uniform_bind_group_layout,
                    bindings: &[wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniform_buffer,
                            // FYI: you can share a single buffer between bindings.
                            range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                        },
                    }],
                    label: Some("uniform_bind_group"),
                });

        let vs_module: Result<&wgpu::ShaderModule, &str>;
        let fs_module: Result<&wgpu::ShaderModule, &str>;
        let mut shader_manager = shader::ShaderManager::new();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let vs_handle = shader_manager
                .load_shader_type(
                    &gpu_interfaces.device,
                    "resources/shader",
                    shader::ShaderType::VERTEX,
                )
                .await;
            let fs_handle = shader_manager
                .load_shader_type(
                    &gpu_interfaces.device,
                    "resources/shader",
                    shader::ShaderType::FRAGMENT,
                )
                .await;
            vs_module = shader_manager.get_shader_module(&vs_handle);
            fs_module = shader_manager.get_shader_module(&fs_handle);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let vs_handle = shader_manager
                .load_shader_type(&gpu_interfaces.device, "resources/shader", shader::ShaderType::VERTEX)
                .await;
            let fs_handle = shader_manager
                .load_shader_type(&gpu_interfaces.device, "resources/shader", shader::ShaderType::FRAGMENT)
                .await;
            vs_module = shader_manager.get_shader_module(&vs_handle);
            fs_module = shader_manager.get_shader_module(&fs_handle);
        }

        let render_pipeline_layout =
            gpu_interfaces
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&uniform_bind_group_layout],
                });

        let render_pipeline =
            gpu_interfaces
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    layout: &render_pipeline_layout,
                    vertex_stage: wgpu::ProgrammableStageDescriptor {
                        module: (vs_module.unwrap()),
                        entry_point: "main",
                    },
                    //frag is optional so we wrap it into an optioal
                    fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                        module: (fs_module.unwrap()),
                        entry_point: "main",
                    }),
                    rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::Back,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                    }),
                    color_states: &[wgpu::ColorStateDescriptor {
                        format: gpu_interfaces.sc_desc.format,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                    primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                    depth_stencil_state: None,
                    vertex_state: wgpu::VertexStateDescriptor {
                        index_format: wgpu::IndexFormat::Uint16,
                        vertex_buffers: &[],
                    },
                    sample_count: 1,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                });

        Self {
            engine_runtime,
            render_pipeline,
            camera,
            uniform_buffer,
            uniform_bind_group,
            size,
            color,
            shader_manager,
            camera_controller,
            uniforms,
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

    fn update(&mut self) {
        //not doing anything here yet
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);

        // Copy operation's are performed on the gpu, so we'll need
        // a CommandEncoder for that
        let mut encoder =
            self.engine_runtime.gpu_interfaces
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("update encoder"),
                });

        let staging_buffer = self.engine_runtime.gpu_interfaces.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );

        // We need to remember to submit our CommandEncoder's output
        // otherwise we won't see any change.
        //self.queue.submit(&[encoder.finish()]);
        self.engine_runtime.gpu_interfaces.queue.submit(Some(encoder.finish()));
    }

    fn render(&mut self) {
        //first we need to get the frame we can use from the swap chain so we can render to it
        let frame = self
            .engine_runtime.gpu_interfaces
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        //this is the command buffer we use to record commands
        let mut encoder =
            self.engine_runtime.gpu_interfaces
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

            render_pass.set_pipeline(&self.render_pipeline);
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
        self.engine_runtime.gpu_interfaces.queue.submit(Some(encoder.finish()));
    }

}