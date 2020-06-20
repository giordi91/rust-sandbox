use super::super::platform;
use super::api;
use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

pub enum MeshBufferSemantic {
    None,
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

pub struct MeshBufferMapper {
    semantic: MeshBufferSemantic,
    offset: u32,
    length: u32,
    buffer_idx: u32,
}

pub struct MeshIndexBufferMapper {
    offset: u32,
    length: u32,
    is_uint16: bool,
}

#[derive(Default)]
pub struct Mesh {
    pub buffers: Vec<MeshBufferMapper>,
    pub index_buffer: Option<MeshIndexBufferMapper>,
}

pub struct Model {
    meshes: Vec<Mesh>,
}


pub struct GltfFile
{
    models : Vec<Model>,

} 

fn load_gltf_mesh_primitive(
    primitive: &gltf::Primitive,
    raw_buffers: &HashMap<u32, wgpu::Buffer>,
) -> Mesh {
    let attributes = primitive.attributes();

    //cannot get the size of the iterator because in doing so items are consumed
    //and ownership transfered
    let mut descriptors = Vec::new();
    let mut counter: u32 = 0;

    let mut mesh = Mesh::default();

    for attribute in attributes {
        println!("{:?} ", attribute.0);
        //mapping the semantic to a vertex format
        let semantic = attribute.0;
        let accessor = &attribute.1;

        let mut wgpu_semantic = MeshBufferSemantic::None;
        //converting to wgpu vertex format
        let format = match semantic {
            gltf::Semantic::Normals => {
                wgpu_semantic = MeshBufferSemantic::Normals;
                debug_assert!(accessor.data_type() == gltf::accessor::DataType::F32);
                wgpu::VertexFormat::Float3
            }
            gltf::Semantic::Positions => {
                wgpu_semantic = MeshBufferSemantic::Positions;
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
        //just making sure the buffer is in the raw list
        assert!(raw_buffers.contains_key(&(buffer_idx as u32)));

        //here we copy the data into corresponding gpu buffers
        //TODO have some smarter logic to keep a single allocation and only
        //using offsets
        let wgpu_buffer_usage = match wgpu_semantic {
            MeshBufferSemantic::Normals | MeshBufferSemantic::Positions => {
                wgpu::BufferUsage::VERTEX
            }
            _ => panic!("Requested wgpu_semantic not yet supported"),
        };

        //semantics.push(wgpu_semantic);

        //need to find the correct slice of the buffer
        let accessor_offset = accessor.offset();
        let view_offset = buffer_view.offset();
        let total_offset = accessor_offset + view_offset;
        //this is where the view end, but not necessarely the length of the buffer
        let view_len = buffer_view.length();

        let mesh_buffer = MeshBufferMapper {
            semantic: wgpu_semantic,
            offset: total_offset as u32,
            length: view_len as u32,
            buffer_idx: buffer_idx as u32,
        };

        mesh.buffers.push(mesh_buffer);

        counter += 1;
    }

    //next load index buffer if there is one
    let index_accessor = primitive.indices();
    match index_accessor {
        Some(idx_accessor) => {
            //read the index buffer
            let buffer_view = idx_accessor.view().unwrap();
            let buffer = buffer_view.buffer();
            let buffer_idx = buffer.index();
            //just making sure the buffer is in the raw list
            assert!(raw_buffers.contains_key(&(buffer_idx as u32)));

            let accessor_offset = idx_accessor.offset();
            let view_offset = buffer_view.offset();
            let total_offset = accessor_offset + view_offset;
            let view_len = buffer_view.length();

            let is_uint16 = match idx_accessor.data_type() {
                gltf::accessor::DataType::U16 => true,
                gltf::accessor::DataType::U32 => false,
                _ => panic!("unexpected datatype for index buffer"),
            };

            let mesh_idx_buffer = MeshIndexBufferMapper {
                offset: total_offset as u32,
                length: view_len as u32,
                is_uint16,
            };

            mesh.index_buffer = Some(mesh_idx_buffer);
        }
        None => {}
    }

    mesh
}

fn load_gltf_mesh(mesh: &gltf::Mesh, raw_buffers: &HashMap<u32, wgpu::Buffer>) -> Vec<Mesh> {
    let primitives = mesh.primitives();
    let mut meshes = Vec::new();
    for primitive in primitives {
        let mesh = load_gltf_mesh_primitive(&primitive, raw_buffers);
        meshes.push(mesh);
    }

    meshes
}

pub async fn load_gltf_file(file_name: &str, gpu_interfaces: &api::GPUInterfaces) -> GltfFile {
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

        let wgpu_buffer = gpu_interfaces.device.create_buffer_with_data(
            bytemuck::cast_slice(&buffer_content[..]),
            wgpu::BufferUsage::INDEX | wgpu::BufferUsage::VERTEX,
        );

        raw_buffers.insert(buffer_idx as u32, wgpu_buffer);
    }

    let mut models = Vec::new();
    for mesh in gltf.meshes() {
        let meshes = load_gltf_mesh(&mesh, &raw_buffers);

        let model = Model { meshes };
        models.push(model);
    }

    GltfFile{
        models,
    }
}
