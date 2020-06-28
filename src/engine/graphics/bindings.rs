use serde_json::Value;
use std::collections::HashMap;

use super::super::handle;
use crate::engine::graphics;
use crate::engine::platform::file_system;

#[derive(Default)]
pub struct PipelineManager {
    bg_mapper: HashMap<u64, wgpu::BindGroupLayout>,
    bg_path_mapper: HashMap<String, Vec<u64>>,
    pipe_mapper: HashMap<u64, wgpu::RenderPipeline>,
    compute_pipe_mapper: HashMap<u64, wgpu::ComputePipeline>,
    pipe_path_mapper: HashMap<String, u64>,
    handle_counter: u64,
}

impl PipelineManager {
    pub async fn load_pipeline(
        &mut self,
        file_name: &str,
        shader_manager: &mut graphics::shader::ShaderManager,
        gpu_interfaces: &graphics::api::GPUInterfaces,
        default_depth_format: wgpu::TextureFormat,
        //layout: &wgpu::BindGroupLayout,
    ) -> handle::ResourceHandle {
        let loaded = self.pipe_path_mapper.contains_key(file_name);
        if loaded {
            let handle_data = self.pipe_path_mapper[file_name];
            return handle::ResourceHandle::from_data(handle_data);
        }

        let pipe_source = file_system::load_file_string(file_name).await.unwrap();
        let pipe_content_json: Value = serde_json::from_str(&pipe_source[..]).unwrap();

        let pipe_type = pipe_content_json["type"].as_str().unwrap();

        self.handle_counter += 1;
        let mut handle = self.handle_counter;

        match pipe_type {
            "raster" => {
                let pipe = self
                    .process_raster_pipeline(
                        pipe_content_json,
                        shader_manager,
                        gpu_interfaces,
                        default_depth_format,
                    )
                    .await;
                self.pipe_mapper.insert(handle, pipe);
            }
            "compute" => {
                let pipe = self
                    .process_compute_pipeline(pipe_content_json, shader_manager, gpu_interfaces)
                    .await;
                //tagging handle as compute
                handle |= (1 as u64) << (63 - handle::HANDLE_TYPE_BIT_COUNT);
                self.compute_pipe_mapper.insert(handle, pipe);
            }
            _ => panic!("Requested pipeline type not supported"),
        };

        self.pipe_path_mapper
            .insert(String::from(file_name), handle);

        handle::ResourceHandle::new(handle::ResourceHandleType::Pipeline, handle)
    }
    pub fn get_pipeline_from_handle(
        &self,
        handle: &handle::ResourceHandle,
    ) -> Result<&wgpu::RenderPipeline, &'static str> {
        let value = handle.get_value();
        let pipe = match self.pipe_mapper.get(&value) {
            Some(pipe) => pipe,
            None => return Err("could not find binding group layout"),
        };
        Ok(pipe)
    }
    pub fn get_compute_pipeline_from_handle(
        &self,
        handle: &handle::ResourceHandle,
    ) -> Result<&wgpu::ComputePipeline, &'static str> {
        let value = handle.get_value();
        assert!(!self.is_raster_pipeline(handle),"provided handle is not for a compute pipeline");
        let pipe = match self.compute_pipe_mapper.get(&value) {
            Some(pipe) => pipe,
            None => return Err("could not find binding group layout"),
        };
        Ok(pipe)
    }

    pub fn is_raster_pipeline(&self, handle: &handle::ResourceHandle) -> bool {
        let handle_type = handle.get_type();
        let handle_value = handle.get_value();
        assert!(handle_type == handle::ResourceHandleType::Pipeline);
        let mask = (1 as u64) << (63 - handle::HANDLE_TYPE_BIT_COUNT);
        (handle_value & mask) == 0
    }

    pub fn get_bind_group_from_handle(
        &self,
        handle: &handle::ResourceHandle,
    ) -> Result<&wgpu::BindGroupLayout, &'static str> {
        let value = handle.get_value();
        let group = match self.bg_mapper.get(&value) {
            Some(group) => group,
            None => return Err("could not find binding group layout"),
        };
        Ok(group)
    }

    pub async fn load_binding_group(
        &mut self,
        file_name: &str,
        gpu_interfaces: &graphics::api::GPUInterfaces,
    ) -> Vec<handle::ResourceHandle> {
        let loaded = self.bg_path_mapper.contains_key(file_name);
        if loaded {
            let source_data = self.bg_path_mapper.get(file_name);
            let mut to_return = Vec::new();
            for h in source_data.unwrap() {
                to_return.push(handle::ResourceHandle::from_data(*h));
            }
            return to_return;
        }

        let bg_source = file_system::load_file_string(file_name).await.unwrap();
        let bg_content_js: Value = serde_json::from_str(&bg_source[..]).unwrap();
        let set_values = bg_content_js["bindings"].as_array().unwrap();

        let mut set_bindings = Vec::new();
        for set in set_values {
            let mut bindings = Vec::new();
            let json_bindings = set.as_array().unwrap();
            for binding in json_bindings {
                let slot = binding["slot"].as_u64().unwrap() as u32;
                let visibility_array = binding["visibility"].as_array().unwrap();
                let visibility_bitfiled = get_bind_group_visibility(visibility_array);
                let type_value = binding["type"].as_str().unwrap();
                let binding_type = get_bind_group_type(type_value, binding, gpu_interfaces.sc_desc.format);

                bindings.push(wgpu::BindGroupLayoutEntry {
                    binding: slot,
                    visibility: visibility_bitfiled,
                    ty: binding_type,
                });
            }
            //oh wow... all this to get the string
            let file_name_no_ext = std::path::Path::new(file_name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            let bind_group_layout =
                gpu_interfaces
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        bindings: &bindings[..],
                        label: Some(&format!("{}_bg", file_name_no_ext)[..]),
                    });

            self.handle_counter += 1;
            self.bg_mapper
                .insert(self.handle_counter, bind_group_layout);

            let file_name_str = String::from(file_name);
            if self.bg_path_mapper.contains_key(&file_name_str) {
                self.bg_path_mapper
                    .get_mut(&file_name_str)
                    .unwrap()
                    .push(self.handle_counter)
            } else {
                let v = vec![self.handle_counter];
                self.bg_path_mapper.insert(file_name_str, v);
            }

            let h = handle::ResourceHandle::new(
                handle::ResourceHandleType::BindingGroup,
                self.handle_counter,
            );

            set_bindings.push(h);
        }

        set_bindings
    }

    async fn process_compute_pipeline(
        &mut self,
        pipe_content_json: Value,
        shader_manager: &mut graphics::shader::ShaderManager,
        gpu_interfaces: &graphics::api::GPUInterfaces,
    ) -> wgpu::ComputePipeline {
        let compute_name = pipe_content_json["compute"]["shader_name"]
            .as_str()
            .unwrap();
        let cmp_handle = shader_manager
            .load_shader_type(
                &gpu_interfaces.device,
                compute_name,
                graphics::shader::ShaderType::COMPUPTE,
            )
            .await;

        let cmp_module = shader_manager.get_shader_module(&cmp_handle).unwrap();

        let layout_name = pipe_content_json["layout"].as_str().unwrap();
        let bg_layout_handles = self.load_binding_group(layout_name, gpu_interfaces).await;

        let mut layouts_to_bind = Vec::new();
        for h in bg_layout_handles {
            let bg_layout = self.get_bind_group_from_handle(&h);
            layouts_to_bind.push(bg_layout.unwrap());
        }

        let render_pipeline_layout =
            gpu_interfaces
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &layouts_to_bind[..],
                });

        ///// The layout of bind groups for this pipeline.
        //pub layout: &'a PipelineLayout,

        ///// The compiled compute stage and its entry point.
        //pub compute_stage: ProgrammableStageDescriptor<'a>,

        gpu_interfaces
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                layout: &render_pipeline_layout,
                compute_stage: wgpu::ProgrammableStageDescriptor {
                    module: &cmp_module,
                    entry_point: "main",
                },
            })
    }

    async fn process_raster_pipeline(
        &mut self,
        pipe_content_json: Value,
        shader_manager: &mut graphics::shader::ShaderManager,
        gpu_interfaces: &graphics::api::GPUInterfaces,
        default_depth_format: wgpu::TextureFormat,
        //layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        //get the shaders
        let vertex_name = pipe_content_json["vertex"]["shader_name"].as_str().unwrap();

        let vs_handle = shader_manager
            .load_shader_type(
                &gpu_interfaces.device,
                vertex_name,
                graphics::shader::ShaderType::VERTEX,
            )
            .await;

        //get frag shader if any
        //let mut fs_stage: Option<wgpu::ProgrammableStageDescriptor> = None;
        let fragment_value = &pipe_content_json["fragment"];
        let fs_module: &wgpu::ShaderModule;
        let fs_stage = match fragment_value.as_null() {
            Some(()) => None,
            _ => {
                let fragment_name = fragment_value["shader_name"].as_str().unwrap();
                let fs_handle = shader_manager
                    .load_shader_type(
                        &gpu_interfaces.device,
                        fragment_name,
                        graphics::shader::ShaderType::FRAGMENT,
                    )
                    .await;

                fs_module = shader_manager.get_shader_module(&fs_handle).unwrap();
                Some(wgpu::ProgrammableStageDescriptor {
                    module: (fs_module),
                    entry_point: "main",
                })
            }
        };

        //this needs to happen afterwards, this is because we first compile the shaders,
        //which modfiies shader module. now, since shader module returned here, is an immutable
        //reference of data inside shader manager, we can't get another mutable referenace
        //after wards when compiling the fragment shader. so yeah, this lives here.
        let vs_module = shader_manager.get_shader_module(&vs_handle).unwrap();
        let vs_stage = wgpu::ProgrammableStageDescriptor {
            module: (&vs_module),
            entry_point: "main",
        };

        //next is raster state
        let raster_state = get_pipeline_raster_state(&pipe_content_json);

        //depth state
        let depth_stencil_state = get_depth_stencil_state(&pipe_content_json, default_depth_format);

        let primitive_value = pipe_content_json["primitive_topology"].as_str().unwrap();
        let primitive_topology = match primitive_value {
            "pointList" => wgpu::PrimitiveTopology::PointList,
            "lineList" => wgpu::PrimitiveTopology::LineList,
            "lineStrip" => wgpu::PrimitiveTopology::LineStrip,
            "triangleList" => wgpu::PrimitiveTopology::TriangleList,
            "triangleStrip" => wgpu::PrimitiveTopology::TriangleStrip,
            _ => panic!(
                "could not match requested primitive topology {}",
                primitive_value
            ),
        };

        let color_states =
            get_pipeline_color_states(&pipe_content_json, gpu_interfaces.sc_desc.format);

        let layout_name = pipe_content_json["layout"].as_str().unwrap();
        let bg_layout_handles = self.load_binding_group(layout_name, gpu_interfaces).await;

        let mut layouts_to_bind = Vec::new();
        for h in bg_layout_handles {
            let bg_layout = self.get_bind_group_from_handle(&h);
            layouts_to_bind.push(bg_layout.unwrap());
        }

        let render_pipeline_layout =
            gpu_interfaces
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &layouts_to_bind[..],
                });

        let vertex_state_type = pipe_content_json["vertex_state"]["type"].as_str().unwrap();
        let desc = get_vertex_attrbibute_descriptor(vertex_state_type);

        gpu_interfaces
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &render_pipeline_layout,
                vertex_stage: vs_stage,
                //frag is optional so we wrap it into an optioal
                fragment_stage: fs_stage,
                rasterization_state: Some(raster_state),
                color_states: &color_states[..],
                primitive_topology,
                depth_stencil_state,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint32,
                    vertex_buffers: &desc[..],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            })
    }
}

fn get_depth_stencil_state(
    pipe_content_json: &Value,
    swap_depth_format: wgpu::TextureFormat,
) -> Option<wgpu::DepthStencilStateDescriptor> {
    let state_value = &pipe_content_json["depth_state"];
    if state_value.is_null() {
        return None;
    }

    let format_str = state_value["format"].as_str().unwrap();

    let format = match format_str {
        "default" => swap_depth_format,
        _ => panic!(
            "unsupported swap chain dept format {:?}, if is a valid type add it to the function",
            swap_depth_format
        ),
    };

    let depth_write_enabled = state_value["depth_write_enabled"].as_bool().unwrap();
    let depth_compare = get_compare_function(&state_value["depth_compare"]);

    Some(wgpu::DepthStencilStateDescriptor {
        format,
        depth_write_enabled,
        depth_compare,
        stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
        stencil_read_mask: 0,
        stencil_write_mask: 0,
    })
}

fn get_compare_function(value: &serde_json::Value) -> wgpu::CompareFunction {
    let compare_str = value.as_str().unwrap();
    match compare_str {
        "Undefined" => wgpu::CompareFunction::Undefined,
        "Never" => wgpu::CompareFunction::Never,
        "Less" => wgpu::CompareFunction::Less,
        "Equal" => wgpu::CompareFunction::Equal,
        "LessEqual" => wgpu::CompareFunction::LessEqual,
        "Greater" => wgpu::CompareFunction::Greater,
        "NotEqual" => wgpu::CompareFunction::NotEqual,
        "GreaterEqual" => wgpu::CompareFunction::GreaterEqual,
        "Always" => wgpu::CompareFunction::Always,
        _ => panic!("Not supported compare fuction {}", compare_str),
    }
}

fn get_vertex_attrbibute_descriptor(name: &str) -> Vec<wgpu::VertexBufferDescriptor<'static>> {
    match name {
        "position_normal" => vec![
            wgpu::VertexBufferDescriptor {
                stride: 12 as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                }],
            },
            wgpu::VertexBufferDescriptor {
                stride: 12 as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                }],
            },
        ],
        "position" => vec![wgpu::VertexBufferDescriptor {
            stride: 12 as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float3,
            }],
        }],
        "none" => Vec::new(),
        _ => panic!("could not find {} vertex description", name),
    }
}

fn get_bind_group_visibility(visibilities: &[Value]) -> wgpu::ShaderStage {
    let mut out_vis = wgpu::ShaderStage::NONE;
    for visibility in visibilities {
        let visibility_str = visibility.as_str().unwrap();
        out_vis |= match visibility_str {
            "vertex" => wgpu::ShaderStage::VERTEX,
            "fragment" => wgpu::ShaderStage::FRAGMENT,
            "compute" => wgpu::ShaderStage::COMPUTE,
            _ => panic!("Unknown wgpu shader statage {}", visibility_str),
        };
    }

    //returning built visibility field
    out_vis
}

fn get_bind_group_type(
    type_str: &str,
    binding: &Value,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::BindingType {
    match type_str {
        //if is a uniform , we extract some extra data and return the built type
        "uniform" => {
            let uniform_config = &binding["uniform_config"];
            wgpu::BindingType::UniformBuffer {
                dynamic: uniform_config["dynamic"].as_bool().unwrap(),
            }
        }
        "texture" => {
            let texture_config = &binding["texture_config"];
            wgpu::BindingType::SampledTexture {
                multisampled: texture_config["multisampled"].as_bool().unwrap(),
                dimension: match texture_config["dimension"].as_str().unwrap() {
                    "2D" => wgpu::TextureViewDimension::D2,
                    _ => panic!("Texture dimension not yet supported please add it to the code"),
                },
                component_type: match texture_config["component_type"].as_str().unwrap() {
                    "uint" => wgpu::TextureComponentType::Uint,
                    "float" => wgpu::TextureComponentType::Float,
                    _ => {
                        panic!("Texture componet type not yet supported please add it to the code")
                    }
                },
            }
        }
        "sampler" => {
            let sampler_config = &binding["sampler_config"];
            wgpu::BindingType::Sampler {
                comparison: sampler_config["comparison"].as_bool().unwrap(),
            }
        }
        "storage_texture" => {
            let texture_config = &binding["texture_config"];
            wgpu::BindingType::StorageTexture {
                dimension: match texture_config["dimension"].as_str().unwrap() {
                    "2D" => wgpu::TextureViewDimension::D2,
                    _ => panic!("Texture dimension not yet supported please add it to the code"),
                },
                component_type: match texture_config["component_type"].as_str().unwrap() {
                    "uint" => wgpu::TextureComponentType::Uint,
                    "float" => wgpu::TextureComponentType::Float,
                    _ => {
                        panic!("Texture component type not yet supported please add it to the code")
                    }
                },
                format: get_color_format(
                    texture_config["format"].as_str().unwrap(),
                    swap_chain_format,
                ),
                readonly: texture_config["readonly"].as_bool().unwrap(),
            }
        }
        _ => panic!("Unexpected binding group type {}", type_str),
    }
}

fn get_pipeline_color_states(
    pipe_content_json: &Value,
    swap_chain_format: wgpu::TextureFormat,
) -> Vec<wgpu::ColorStateDescriptor> {
    let color_values = &pipe_content_json["color_states"];

    let color_exists = !color_values.is_null();
    if color_exists {
        let mut color_states = Vec::new();
        for color_value in color_values.as_array().unwrap() {
            let format_str = color_value["format"].as_str().unwrap();
            let format = get_color_format(format_str, swap_chain_format);
            let color_blend = get_pipeline_blend(color_value, "color_blend");
            let alpha_blend = get_pipeline_blend(color_value, "alpha_blend");

            color_states.push(wgpu::ColorStateDescriptor {
                format,
                color_blend,
                alpha_blend,
                write_mask: wgpu::ColorWrite::ALL,
            });
        }

        return color_states;
    } else {
        Vec::new()
    }
}

fn get_pipeline_blend(color_value: &Value, name: &str) -> wgpu::BlendDescriptor {
    let blend_value = color_value[name].as_str().unwrap();
    match blend_value {
        "replace" => wgpu::BlendDescriptor::REPLACE,
        _ => panic!("blend descriptor not supported yet {}", blend_value),
    }
}

fn get_color_format(
    color_str: &str,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::TextureFormat {
    match color_str {
        "swap_chain_native" => swap_chain_format,
        "Rgb10a2Unorm" => wgpu::TextureFormat::Rgb10a2Unorm,
        "Rgba32Float" => wgpu::TextureFormat::Rgba32Float,
        "Rg16Float" => wgpu::TextureFormat::Rg16Float,
        _ => panic!(
            "unsupported swap chain format {:?}, if is a valid type add it to the function",
            swap_chain_format
        ),
    }
}

fn get_pipeline_raster_state(pipe_content_json: &Value) -> wgpu::RasterizationStateDescriptor {
    let raster_value = &pipe_content_json["rasterization_state"];
    let raster_type = raster_value["type"].as_str().unwrap();
    match raster_type {
        "default" => wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        _ => {
            //we parse the raster definition
            wgpu::RasterizationStateDescriptor {
                front_face: get_raster_facing(raster_value),
                cull_mode: get_raster_cull(raster_value),
                depth_bias: raster_value["depth_bias"].as_i64().unwrap() as i32,
                depth_bias_slope_scale: raster_value["slope_scale"].as_f64().unwrap() as f32,
                depth_bias_clamp: raster_value["bias_clamp"].as_f64().unwrap() as f32,
            }
        }
    }
}

fn get_raster_facing(raster_value: &Value) -> wgpu::FrontFace {
    let front_face_str = raster_value["front_facing"].as_str().unwrap();
    match front_face_str {
        "ccw" => wgpu::FrontFace::Ccw,
        "cw" => wgpu::FrontFace::Cw,
        _ => panic!(
            "could not match requessted front facing value {}",
            front_face_str
        ),
    }
}

fn get_raster_cull(raster_value: &Value) -> wgpu::CullMode {
    let cull_str = raster_value["cull_mode"].as_str().unwrap();
    match cull_str {
        "none" => wgpu::CullMode::None,
        "front" => wgpu::CullMode::Front,
        "back" => wgpu::CullMode::Back,
        _ => panic!("could not match requested cull facing value {}", cull_str),
    }
}
