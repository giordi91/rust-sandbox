use super::super::handle;
use super::super::platform;
use super::api;
use super::buffer;

use std::collections::HashMap;

#[derive(PartialEq)]
pub enum MeshBufferSemantic {
    None,
    Positions,
    Normals,
    TexCoords,
}

pub struct MeshBufferMapper {
    pub semantic: MeshBufferSemantic,
    pub offset: u32,
    pub length: u32,
    pub buffer_idx: handle::ResourceHandle,
}

pub struct MeshIndexBufferMapper {
    pub offset: u32,
    pub length: u32,
    pub is_uint16: bool,
    pub buffer_idx: handle::ResourceHandle,
    pub count: u32,
}

#[derive(Default)]
pub struct Mesh {
    pub buffers: Vec<MeshBufferMapper>,
    pub index_buffer: Option<MeshIndexBufferMapper>,
}

impl Mesh {
    pub fn get_buffer_from_semantic(&self, semantic: MeshBufferSemantic) -> &MeshBufferMapper {
        for buffer in self.buffers.iter() {
            if buffer.semantic == semantic {
                return buffer;
            }
        }

        panic!("Could not find buffer with requested semantic");
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub matrix: cgmath::Matrix4<f32>,
}

pub struct GltfFile {
    pub models: Vec<Model>,
}

fn load_gltf_mesh_primitive(
    primitive: &gltf::Primitive,
    gpu_raw_buffers: &Vec<handle::ResourceHandle>,
    raw_buffers: &HashMap<u32, Vec<u8>>,
    gpu_interfaces: &api::GPUInterfaces,
    buffer_manager: &mut buffer::BufferManager,
) -> Mesh {
    let attributes = primitive.attributes();

    //cannot get the size of the iterator because in doing so items are consumed
    //and ownership transfered
    let mut descriptors = Vec::new();
    let mut counter: u32 = 0;

    let mut mesh = Mesh::default();

    for attribute in attributes {
        //mapping the semantic to a vertex format
        let semantic = attribute.0;
        let accessor = &attribute.1;

        //converting to wgpu vertex format
        let (format, wgpu_semantic) = match semantic {
            gltf::Semantic::Normals => {
                debug_assert!(accessor.data_type() == gltf::accessor::DataType::F32);
                (wgpu::VertexFormat::Float3, MeshBufferSemantic::Normals)
            }
            gltf::Semantic::Positions => {
                debug_assert!(accessor.data_type() == gltf::accessor::DataType::F32);
                (wgpu::VertexFormat::Float3, MeshBufferSemantic::Positions)
            }
            //_ => panic!("GLTF atttribute semantic not yet supported {:?}", semantic),
            _ => {
                platform::core::to_console(
                    &format!(
                        "GLTF atttribute semantic not yet supported {:?} and will be ignored...",
                        semantic
                    )[..],
                );
                continue;
            }
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

        let buff_handle = gpu_raw_buffers.get(buffer_idx).unwrap();

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
            buffer_idx: buff_handle.clone(),
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
            let buffer_idx_gltf = buffer.index();

            let ori_handle = gpu_raw_buffers.get(buffer_idx_gltf).unwrap();
            let mut buffer_idx = ori_handle.clone();

            let accessor_offset = idx_accessor.offset();
            let view_offset = buffer_view.offset();
            let mut total_offset = accessor_offset + view_offset;
            let mut view_len = buffer_view.length();
            let count = idx_accessor.count() as u32;

            let is_uint16 = match idx_accessor.data_type() {
                gltf::accessor::DataType::U16 => true,
                gltf::accessor::DataType::U32 => false,
                _ => panic!("unexpected datatype for index buffer"),
            };

            //let us convert the buffer to u32
            if is_uint16 {
                //lets allocate a buffer big enough
                let mut idx_buffer_32 = Vec::new();
                idx_buffer_32.reserve(count as usize);

                let raw_buffer = raw_buffers.get(&(buffer_idx_gltf as u32)).unwrap();
                let indices: &[u16] = bytemuck::cast_slice(&raw_buffer[total_offset..]);
                for i in 0..count {
                    idx_buffer_32.push(indices[i as usize] as u32);
                }

                //create a wgpu buffer
                let buff_handle = buffer_manager.create_buffer_with_data(
                    bytemuck::cast_slice(&idx_buffer_32[..]),
                    wgpu::BufferUsage::INDEX,
                    gpu_interfaces,
                );

                buffer_idx = buff_handle;
                total_offset = 0;
                view_len = (count * 4) as usize;
            }

            let mesh_idx_buffer = MeshIndexBufferMapper {
                offset: total_offset as u32,
                length: view_len as u32,
                is_uint16: false,
                buffer_idx,
                count,
            };

            mesh.index_buffer = Some(mesh_idx_buffer);
        }
        None => {}
    }

    mesh
}

fn load_gltf_mesh(
    mesh: &gltf::Mesh,
    gpu_raw_buffers: &Vec<handle::ResourceHandle>,
    raw_buffers: &HashMap<u32, Vec<u8>>,
    gpu_interfaces: &api::GPUInterfaces,
    buffer_manager: &mut buffer::BufferManager,
) -> Vec<Mesh> {
    let primitives = mesh.primitives();
    let mut meshes = Vec::new();
    for primitive in primitives {
        let mesh = load_gltf_mesh_primitive(
            &primitive,
            gpu_raw_buffers,
            raw_buffers,
            gpu_interfaces,
            buffer_manager,
        );
        meshes.push(mesh);
    }

    meshes
}

pub async fn load_gltf_file(
    file_name: &str,
    gpu_interfaces: &api::GPUInterfaces,
    buffer_manager: &mut buffer::BufferManager,
) -> GltfFile {
    let gltf_content = platform::file_system::load_file_u8(file_name)
        .await
        .unwrap();
    let gltf = gltf::Gltf::from_slice(&gltf_content[..]).unwrap();

    //assert!(gltf.scenes().len() == 0, "only one scene is supported for now on in the gltf loader");
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
    let mut gpu_raw_buffers = Vec::new();

    //let us first load all the buffers
    for buffer in gltf.buffers() {
        let buffer_idx = buffer.index();

        let buffer_src = buffer.source();

        let buffer_uri = match buffer_src {
            gltf::buffer::Source::Uri(uri) => uri,
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

        let gpu_buff_handle = buffer_manager.create_buffer_with_data(
            bytemuck::cast_slice(&buffer_content[..]),
            wgpu::BufferUsage::INDEX | wgpu::BufferUsage::VERTEX,
            gpu_interfaces,
        );

        raw_buffers.insert(buffer_idx as u32, buffer_content);
        gpu_raw_buffers.push(gpu_buff_handle);
    }

    let mut models = Vec::new();

    for scene in gltf.scenes() {
        //scene
        for node in scene.nodes() {
            if let Some(curr_mesh) = node.mesh() {
                //we have the correct node let us extract the mesh
                let transform = node.transform().matrix();
                let c0 = cgmath::vec4(transform[0][0],transform[0][1],transform[0][2],transform[0][3]);
                let c1 = cgmath::vec4(transform[1][0],transform[1][1],transform[1][2],transform[1][3]);
                let c2 = cgmath::vec4(transform[2][0],transform[2][1],transform[2][2],transform[2][3]);
                let c3 = cgmath::vec4(transform[3][0],transform[3][1],transform[3][2],transform[3][3]);

                let matrix = cgmath::Matrix4::from_cols(c0, c1, c2, c3);

                let meshes = load_gltf_mesh(
                    &curr_mesh,
                    &gpu_raw_buffers,
                    &raw_buffers,
                    gpu_interfaces,
                    buffer_manager,
                );

                let model = Model { meshes, matrix };
                models.push(model);
            }
        }
    }

    GltfFile { models }
}

/*
fn load_transformation(
    mesh: &gltf::Mesh,
    mesh_index: u32,
    gltf_scene: &gltf::Scene,
) -> cgmath::Matrix4<f32> {

    let mesh_idx = mesh.index();
    for node in gltf_scene.nodes(){

        if let  Some(curr_mesh)= node.mesh()
        {
            if curr_mesh.index() != mesh_idx
            {
                continue;
            }
            //we have the correct node let us extract the mesh
            node.transform()
        }
    }
    cgmath::prelude::SquareMatrix::identity()
}
*/
