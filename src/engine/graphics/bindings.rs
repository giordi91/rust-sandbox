use crate::engine::graphics;
use crate::engine::platform;
use crate::engine::platform::file_system;
use serde_json::Value;

pub fn get_bind_group_visibility(visiblities: &Vec<Value>) -> wgpu::ShaderStage {
    let mut out_vis = wgpu::ShaderStage::NONE;
    for visibility in visiblities {
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

pub fn get_bind_group_type(type_str: &str, binding: &Value) -> wgpu::BindingType {
    match type_str {
        //if is a uniform , we extract some extra data and return the built type
        "uniform" => wgpu::BindingType::UniformBuffer {
            dynamic: binding["dynamic"].as_bool().unwrap(),
        },
        _ => panic!("Unexpected binding group type {}", type_str),
    }
}

pub async fn load_binding_group(
    file_name: &str,
    gpu_interfaces: &graphics::api::GPUInterfaces,
) -> wgpu::BindGroupLayout {
    let bg_source = file_system::load_file_string(file_name).await.unwrap();
    let bg_content_js: Value = serde_json::from_str(&bg_source[..]).unwrap();
    let bindings_values = &bg_content_js["bindings"].as_array().unwrap();

    let mut bindings = Vec::new();
    for binding in *bindings_values {
        let slot = binding["slot"].as_u64().unwrap() as u32;
        let visibility_array = binding["visibility"].as_array().unwrap();
        let visibility_bitfiled = get_bind_group_visibility(visibility_array);
        let type_value = binding["type"].as_str().unwrap();
        let binding_type = get_bind_group_type(type_value, binding);

        bindings.push(wgpu::BindGroupLayoutEntry {
            binding: slot,
            visibility: visibility_bitfiled,
            ty: binding_type,
        })
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
    bind_group_layout
}

pub async fn load_pipeline(
    file_name: &str,
    shader_manager: &mut graphics::shader::ShaderManager,
    gpu_interfaces: &graphics::api::GPUInterfaces,
) {
    let pipe_source = file_system::load_file_string(file_name).await.unwrap();
    let pipe_content_json: Value = serde_json::from_str(&pipe_source[..]).unwrap();

    let pipe_type = pipe_content_json["type"].as_str().unwrap();
    match pipe_type {
        "raster" => {
            process_raster_pipeline(pipe_content_json, shader_manager, gpu_interfaces).await
        }
        _ => panic!(),
    }
}

async fn process_raster_pipeline(
    pipe_content_json: Value,
    shader_manager: &mut graphics::shader::ShaderManager,
    gpu_interfaces: &graphics::api::GPUInterfaces,
) {
    //get the shaders
    let vertex_name = pipe_content_json["vertex"]["shader_name"].as_str().unwrap();

    let vs_handle = shader_manager
        .load_shader_type(
            &gpu_interfaces.device,
            vertex_name,
            graphics::shader::ShaderType::VERTEX,
        )
        .await;

    let vs_module = shader_manager.get_shader_module(&vs_handle).unwrap();
    let vs_stage = wgpu::ProgrammableStageDescriptor {
        module: (vs_module),
        entry_point: "main",
    };

    //get frag shader if any
    let mut fs_stage: Option<wgpu::ProgrammableStageDescriptor> = None;
    let fragment_value = &pipe_content_json["fragment"];
    if !fragment_value.is_null() {
        let fragment_name = fragment_value["shader_name"].as_str().unwrap();
        let fs_handle = shader_manager
            .load_shader_type(
                &gpu_interfaces.device,
                fragment_name,
                graphics::shader::ShaderType::VERTEX,
            )
            .await;

        let fs_module = shader_manager.get_shader_module(&fs_handle).unwrap();
        fs_stage = Some(wgpu::ProgrammableStageDescriptor {
            module: (fs_module),
            entry_point: "main",
        });
    }

    //next is raster state
    let raster_state = get_pipeline_raster_state(&pipe_content_json);

    let primitive_value = pipe_content_json["primitive_topology"].as_str().unwrap(); 
    let primitive_topology  = match primitive_value{
        "pointList" => wgpu::PrimitiveTopology::PointList,
        "lineList" => wgpu::PrimitiveTopology::LineList,
        "lineStrip" => wgpu::PrimitiveTopology::LineStrip,
        "triangleList" => wgpu::PrimitiveTopology::TriangleList,
        "triangleStrip" => wgpu::PrimitiveTopology::TriangleStrip,
        _ => panic!("could not match requested primitive topology {}",primitive_value)
    }; 

    //let color_states  = get_pipeline_color
}

fn get_pipeline_raster_state(pipe_content_json: &Value) -> wgpu::RasterizationStateDescriptor {
    let raster_value= &pipe_content_json["rasterization_state"];
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

/*
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
*/
