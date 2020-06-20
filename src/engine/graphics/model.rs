use super::super::platform;
use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

pub enum MeshBufferSemantic {
    Positions,
    Normals,
    TexCoords,
}

// main.rs
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    },
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Mesh {}

pub struct Model {}

/*
fn load_gltf_primitive_attribute_definition(
    primitive: &gltf::Primitive,
) -> (
    Vec<MeshBufferSemantic>,
    Vec<wgpu::VertexAttributeDescriptor>,
) {

    (semantics, descriptors)
}
*/

fn load_gltf_mesh_primitive(primitive: &gltf::Primitive, raw_buffers : &HashMap<u32,Vec<u8>>  ) {
    let attributes = primitive.attributes();

    //cannot get the size of the iterator because in doing so items are consumed
    //and ownership transfered
    let mut descriptors = Vec::new();
    let mut semantics = Vec::new();
    let mut counter: u32 = 0;

    //let buffers = HashMap::new();

    for attribute in attributes {
        println!("{:?} ", attribute.0);
        //mapping the semantic to a vertex format
        let semantic = attribute.0;
        let accessor = &attribute.1;

        //converting to wgpu vertex format
        let format = match semantic {
            gltf::Semantic::Normals => {
                semantics.push(MeshBufferSemantic::Normals);
                debug_assert!(accessor.data_type() == gltf::accessor::DataType::F32);
                wgpu::VertexFormat::Float3
            }
            gltf::Semantic::Positions => {
                semantics.push(MeshBufferSemantic::Positions);
                debug_assert!(accessor.data_type() == gltf::accessor::DataType::F32);
                wgpu::VertexFormat::Float3
            }
            _ => panic!("GLTF atttribute semantic not yet supported {:?}", semantic),
        };

        //building the complete vertex mapper
        let offset = accessor.offset() as u64;
        let shader_location = counter;
        descriptors.push(wgpu::VertexAttributeDescriptor {
            offset,
            shader_location,
            format,
        });

        //extracting the buffer data
        let buffer_view = accessor.view().unwrap();
        let buffer = buffer_view.buffer();
        let buffer_idx = buffer.index();

        let buffer_src = buffer.source();

        //let buffer_uri = match buffer_src{
        //    gltf::buffer::Source::Uri(uri) => {println!("{}",uri); uri},
        //    gltf::buffer::Source::Bin => panic!("gltf bin field of buffer is not supported yet");
        //};

        counter += 1;
    }
}

fn load_gltf_mesh(mesh: &gltf::Mesh, raw_buffers : &HashMap<u32,Vec<u8>> ) {
    let primitives = mesh.primitives();
    for primitive in primitives {
        load_gltf_mesh_primitive(&primitive, raw_buffers);
    }
}

pub async fn load_gltf_model(file_name: &str) -> Model {
    let gltf_content = platform::file_system::load_file_u8(file_name)
        .await
        .unwrap();
    let gltf = gltf::Gltf::from_slice(&gltf_content[..]).unwrap();
    /*
    for scene in gltf.scenes() {
        //scene
        for root in scene.nodes() {
            let fmt_str = format!(
                "Node #{} has {} children",
                root.index(),
                root.children().count(),
            );
            platform::core::to_console(&fmt_str[..]);

            for child in root.children() {
                println!("{}", child.index());
                let mesh = child.mesh();
                match mesh {
                    Some(mesh_node) => println!("found mesh"),
                    _ => println!("no mesh"),
                }
            }
        }
    }
    */

    let mut raw_buffers = HashMap::new();

    //let us first load all the buffers
    for buffer in gltf.buffers() {
        let buffer_idx = buffer.index();

        let buffer_src = buffer.source();

        let buffer_uri = match buffer_src {
            gltf::buffer::Source::Uri(uri) => {
                println!("{}", uri);
                uri
            }
            gltf::buffer::Source::Bin => panic!("gltf bin field of buffer is not supported yet"),
        };

        let parent_folder = std::path::Path::new(file_name)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        let buffer_path = String::from(parent_folder) + "/" + buffer_uri;

        let buffer_content = platform::file_system::load_file_u8(&buffer_path[..])
            .await
            .unwrap();

        raw_buffers.insert(buffer_idx as u32,buffer_content);
    }

    for mesh in gltf.meshes() {
        println!("{}", mesh.name().unwrap());
        load_gltf_mesh(&mesh, &raw_buffers);
    }

    Model {}
}
