use async_trait::async_trait;
use winit::{event::*, window::Window};

use rust_sandbox::engine::graphics;
use rust_sandbox::engine::handle;
use rust_sandbox::engine::platform;

//This is a simple datastruct to represent per-object data in the shader
#[repr(C)] // We need this for Rust to store our data correctly for the shaders
#[derive(Debug, Copy, Clone)] // This is so we can store this in a buffer
pub struct ObjectBuffer {
    transform: cgmath::Matrix4<f32>,
    pad1: cgmath::Matrix4<f32>,
    pad2: cgmath::Matrix4<f32>,
    pad3: cgmath::Matrix4<f32>,
}
unsafe impl bytemuck::Pod for ObjectBuffer {}
unsafe impl bytemuck::Zeroable for ObjectBuffer {}

pub struct Sandbox {
    engine_runtime: platform::EngineRuntime,
    render_pipeline_handle: handle::ResourceHandle,
    depth_only_pipeline: handle::ResourceHandle,
    normal_pipeline: handle::ResourceHandle,
    camera: graphics::camera::Camera,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    normal_bgs: Vec<wgpu::BindGroup>,
    per_object_bind_groups: Vec<wgpu::BindGroup>,
    size: winit::dpi::PhysicalSize<u32>,
    color: f64,
    camera_controller: graphics::camera::CameraControllerFPS,
    per_frame_data: graphics::FrameData,
    time_stamp: u64,
    delta_time: u64,
    gltf_file: graphics::model::GltfFile,
    depth_texture: graphics::texture::Texture,
    matrices_buffer: wgpu::Buffer,
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

        let layout_handles = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_binding_group("resources/gltf_model.bg", gpu_interfaces)
            .await;

        let normal_layout_handle = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_binding_group("resources/normal.bg", gpu_interfaces)
            .await;

        let default_depth_format = wgpu::TextureFormat::Depth32Float;

        let render_pipeline_handle = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_pipeline(
                "resources/gltf_model.pipeline",
                &mut engine_runtime.resource_managers.shader_manager,
                &engine_runtime.gpu_interfaces,
                default_depth_format,
            )
            .await;

        let depth_only_pipeline = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_pipeline(
                "resources/depth_only.pipeline",
                &mut engine_runtime.resource_managers.shader_manager,
                &engine_runtime.gpu_interfaces,
                default_depth_format,
            )
            .await;

        let normal_pipeline = engine_runtime
            .resource_managers
            .pipeline_manager
            .load_pipeline(
                "resources/normal_depth.pipeline",
                &mut engine_runtime.resource_managers.shader_manager,
                &engine_runtime.gpu_interfaces,
                default_depth_format,
            )
            .await;

        platform::core::to_console("NEW3!");

        let bg_layout = engine_runtime
            .resource_managers
            .pipeline_manager
            .get_bind_group_from_handle(layout_handles.get(0).unwrap());

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

        //let ao_bg = engine_runtime
        //    .resource_managers
        //    .pipeline_manager
        //    .load_binding_group("resources/ao.bg", gpu_interfaces)
        //    .await;

        let depth_texture = graphics::texture::Texture::create_depth_texture(
            &engine_runtime.gpu_interfaces.device,
            &engine_runtime.gpu_interfaces.sc_desc,
            "swap-depth",
        );

        let normal_layout0 = engine_runtime
            .resource_managers
            .pipeline_manager
            .get_bind_group_from_handle(normal_layout_handle.get(0).unwrap());
        let normal_layout1 = engine_runtime
            .resource_managers
            .pipeline_manager
            .get_bind_group_from_handle(normal_layout_handle.get(1).unwrap());

        let normal_bg0 = gpu_interfaces
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &normal_layout0.unwrap(),
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        // FYI: you can share a single buffer between bindings.
                        range: 0..std::mem::size_of_val(&per_frame_data) as wgpu::BufferAddress,
                    },
                }],
                label: Some("normal binding group"),
            });

        let normal_bg1 = gpu_interfaces
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &normal_layout1.unwrap(),
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&depth_texture.sampler),
                    },
                ],
                label: Some("normal binding group 2"),
            });
        let normal_bgs = vec![normal_bg0,normal_bg1];

        let gltf_file = graphics::model::load_gltf_file(
            "resources/aoScene/aoScene.gltf",
            &gpu_interfaces,
            &mut engine_runtime.resource_managers.buffer_manager,
        )
        .await;
        //we are gonig to build a single buffer with all the necessary matrices
        let mut matrices = Vec::new();
        for model in &gltf_file.models {
            matrices.push(ObjectBuffer {
                transform: model.matrix,
                pad1: cgmath::SquareMatrix::identity(),
                pad2: cgmath::SquareMatrix::identity(),
                pad3: cgmath::SquareMatrix::identity(),
            });
        }
        let matrices_buffer = gpu_interfaces.device.create_buffer_with_data(
            bytemuck::cast_slice(&matrices[..]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );
        let matrices_size = matrices.len() * std::mem::size_of::<ObjectBuffer>();
        let matrix_size = std::mem::size_of::<ObjectBuffer>();

        //the second layout is the one on a per object basis, so we create one per
        //object
        let bg_layout_2 = engine_runtime
            .resource_managers
            .pipeline_manager
            .get_bind_group_from_handle(layout_handles.get(1).unwrap());

        let mut per_object_bind_groups = Vec::new();
        for i in 0..gltf_file.models.len() {
            let start = (i * matrix_size) as u64;
            let end = (i + 1) * matrix_size;
            let uniform_bind_group_2 =
                gpu_interfaces
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &bg_layout_2.unwrap(),
                        bindings: &[wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &matrices_buffer,
                                // FYI: you can share a single buffer between bindings.
                                range: start..end as wgpu::BufferAddress,
                            },
                        }],
                        label: Some("uniform_bind_group 2"),
                    });
            per_object_bind_groups.push(uniform_bind_group_2);
        }

        Self {
            engine_runtime,
            render_pipeline_handle,
            depth_only_pipeline,
            normal_pipeline,
            camera,
            uniform_buffer,
            uniform_bind_group,
            normal_bgs,
            per_object_bind_groups,
            size,
            color,
            camera_controller,
            per_frame_data,
            time_stamp: platform::core::get_time_in_micro(),
            delta_time: 0,
            gltf_file,
            depth_texture,
            matrices_buffer,
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

    fn update(&mut self, command_buffers: &mut Vec<wgpu::CommandBuffer>) {
        //let us update time
        let curr_time = platform::core::get_time_in_micro();
        self.delta_time = curr_time - self.time_stamp;
        self.time_stamp = curr_time;

        //not doing anything here yet
        self.camera_controller
            .update_camera(&mut self.camera, self.delta_time);
        self.per_frame_data.update_view_proj(&self.camera);

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

    fn render(&mut self, mut command_buffers: Vec<wgpu::CommandBuffer>) {
        let mut encoder = self
            .engine_runtime
            .gpu_interfaces
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("update encoder"),
            });
        //first we need to get the frame we can use from the swap chain so we can render to it
        let frame = self
            .engine_runtime
            .gpu_interfaces
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");

        //do the depth prepass
        {
            let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 0.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[],
                //color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                //    attachment: &frame.view,
                //    resolve_target: None,
                //    load_op: wgpu::LoadOp::Clear,
                //    store_op: wgpu::StoreOp::Store,
                //    clear_color: wgpu::Color {
                //        r: 0.1,
                //        g: 0.2,
                //        b: self.color,
                //        a: 1.0,
                //    },
                //}],
                depth_stencil_attachment,
            });

            //let render_pipeline = self
            //    .engine_runtime
            //    .resource_managers
            //    .pipeline_manager
            //    .get_pipeline_from_handle(&self.render_pipeline_handle);
            let render_pipeline = self
                .engine_runtime
                .resource_managers
                .pipeline_manager
                .get_pipeline_from_handle(&self.depth_only_pipeline);
            render_pass.set_pipeline(&render_pipeline.unwrap());
            //set the per frame binding group
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            let models = &self.gltf_file.models;
            let mut counter = 0;
            for model in models {
                let obj_bind = self.per_object_bind_groups.get(counter).unwrap();
                render_pass.set_bind_group(1, &obj_bind, &[]);
                for mesh in model.meshes.iter() {
                    let pos_mapper = mesh
                        .get_buffer_from_semantic(graphics::model::MeshBufferSemantic::Positions);
                    let pos_idx = pos_mapper.buffer_idx;
                    let pos_buff = self
                        .engine_runtime
                        .resource_managers
                        .buffer_manager
                        .get_buffer_from_handle(&pos_idx);

                    //let n_mapper =
                    //    mesh.get_buffer_from_semantic(graphics::model::MeshBufferSemantic::Normals);
                    //let n_idx = n_mapper.buffer_idx;
                    //let n_buff = self
                    //    .engine_runtime
                    //    .resource_managers
                    //    .buffer_manager
                    //    .get_buffer_from_handle(&n_idx);

                    let mut idx_count = 0;
                    match &mesh.index_buffer {
                        Some(idx_buff_map) => {
                            let idx = idx_buff_map.buffer_idx;
                            let idx_buff = self
                                .engine_runtime
                                .resource_managers
                                .buffer_manager
                                .get_buffer_from_handle(&idx);

                            render_pass.set_index_buffer(idx_buff, idx_buff_map.offset as u64, 0);
                            idx_count = idx_buff_map.count;
                        }
                        None => {}
                    }

                    //only need the positions
                    render_pass.set_vertex_buffer(0, pos_buff, pos_mapper.offset as u64, 0);
                    //render_pass.set_vertex_buffer(1, n_buff, n_mapper.offset as u64, 0);
                    render_pass.draw_indexed(0..idx_count, 0, 0..1);
                }
                counter += 1;
            }
        }

        //normal reconstruction
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

            //let render_pipeline = self
            //    .engine_runtime
            //    .resource_managers
            //    .pipeline_manager
            //    .get_pipeline_from_handle(&self.render_pipeline_handle);
            let render_pipeline = self
                .engine_runtime
                .resource_managers
                .pipeline_manager
                .get_pipeline_from_handle(&self.normal_pipeline);
            render_pass.set_pipeline(&render_pipeline.unwrap());
            //set the per frame binding group
            render_pass.set_bind_group(0, &self.normal_bgs.get(0).unwrap(), &[]);
            render_pass.set_bind_group(1, &self.normal_bgs.get(1).unwrap(), &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.color += 0.001;
        if self.color > 1.0 {
            self.color = 0.0;
        }

        command_buffers.push(encoder.finish());
        self.engine_runtime
            .gpu_interfaces
            .queue
            .submit(command_buffers);
    }
}
