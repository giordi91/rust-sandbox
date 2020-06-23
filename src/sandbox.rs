use async_trait::async_trait;
use winit::{event::*, window::Window};

use rust_sandbox::engine::graphics;
use rust_sandbox::engine::handle;
use rust_sandbox::engine::platform;

pub struct Sandbox {
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
    delta_time: u64,
    gltf_file: graphics::model::GltfFile,
    depth_texture: graphics::texture::Texture,
}

#[async_trait(?Send)]
impl platform::Application for Sandbox {
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
            znear: 100.0,
            zfar: 0.1,
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

        let default_depth_format = wgpu::TextureFormat::Depth32Float;

        let render_pipeline_handle = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_pipeline(
                "resources/gltf_model.pipeline",
                &mut engine_runtime.resource_managers.shader_manager,
                &engine_runtime.gpu_interfaces,
                default_depth_format
            )
            .await;

        platform::core::to_console("NEW3!");

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

        let gltf_file = graphics::model::load_gltf_file(
            "resources/external/dragon/dragon.gltf",
            &gpu_interfaces,
        )
        .await;

        let depth_texture = graphics::texture::Texture::create_depth_texture(
            &engine_runtime.gpu_interfaces.device,
            &engine_runtime.gpu_interfaces.sc_desc,
            "swap-depth",
        );

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
            time_stamp: platform::core::get_time_in_micro(),
            delta_time: 0,
            gltf_file,
            depth_texture,
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
        //let us update time
        let curr_time = platform::core::get_time_in_micro();
        self.delta_time = curr_time - self.time_stamp;
        self.time_stamp = curr_time;

        //not doing anything here yet
        self.camera_controller
            .update_camera(&mut self.camera, self.delta_time);
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

        // We need to remember to submit our CommandEncoder's output
        // otherwise we won't see any change.
        //self.queue.submit(&[encoder.finish()]);
        self.engine_runtime
            .gpu_interfaces
            .queue
            .submit(Some(encoder.finish()));
    }

    fn render(&mut self) {
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

            let depth_stencil_attachment =Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 0.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }) ;


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
                depth_stencil_attachment,
            });

            let render_pipeline = self
                .engine_runtime
                .resource_managers
                .pipeline_manager
                .get_pipeline_from_handle(&self.render_pipeline_handle);
            render_pass.set_pipeline(&render_pipeline.unwrap());
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            let model = self.gltf_file.models.get(0).unwrap();
            let mesh = model.meshes.get(0).unwrap();
            let pos_mapper =
                mesh.get_buffer_from_semantic(graphics::model::MeshBufferSemantic::Positions);
            let pos_idx = pos_mapper.buffer_idx;
            let pos_buff = self.gltf_file.buffers.get(&pos_idx).unwrap();

            let n_mapper =
                mesh.get_buffer_from_semantic(graphics::model::MeshBufferSemantic::Normals);
            let n_idx = n_mapper.buffer_idx;
            let n_buff = self.gltf_file.buffers.get(&n_idx).unwrap();

            let mut idx_count = 0;
            match &mesh.index_buffer {
                Some(idx_buff_map) => {
                    let idx = idx_buff_map.buffer_idx;
                    let idx_buff = self.gltf_file.buffers.get(&idx).unwrap();
                    render_pass.set_index_buffer(idx_buff, idx_buff_map.offset as u64, 0);
                    idx_count = idx_buff_map.count;
                }
                None => {}
            }

            render_pass.set_vertex_buffer(0, pos_buff, pos_mapper.offset as u64, 0);
            render_pass.set_vertex_buffer(1, n_buff, n_mapper.offset as u64, 0);
            render_pass.draw_indexed(0..idx_count, 0, 0..1);
        }

        self.color += 0.001;
        if self.color > 1.0 {
            self.color = 0.0;
        }

        //new way to submit, when i will be able to move to master branch again
        //self.queue.submit(&[
        //    encoder.finish()
        //]);
        self.engine_runtime
            .gpu_interfaces
            .queue
            .submit(Some(encoder.finish()));
    }
}
