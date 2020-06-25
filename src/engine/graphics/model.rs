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
    pub buffer_idx: u64,
}

pub struct MeshIndexBufferMapper {
    pub offset: u32,
    pub length: u32,
    pub is_uint16: bool,
    pub buffer_idx: u64,
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
}

pub struct GltfFile {
    pub models: Vec<Model>,
}

fn load_gltf_mesh_primitive(
    primitive: &gltf::Primitive,
    gpu_raw_buffers: &mut HashMap<handle::ResourceHandle, handle::ResourceHandle>,
    raw_buffers: &HashMap<handle::ResourceHandle, Vec<u8>>,
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
        //just making sure the buffer is in the raw list

        let tmp_buff_handle =
            handle::ResourceHandle::new(handle::ResourceHandleType::Buffer, buffer_idx as u64);
        assert!(gpu_raw_buffers.contains_key(&(tmp_buff_handle)));
        let buff_handle = gpu_raw_buffers.get(&tmp_buff_handle).unwrap();

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
            buffer_idx: buff_handle.raw(),
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
            //just making sure the buffer is in the raw list
            let idx_buff_handle = handle::ResourceHandle::new(
                handle::ResourceHandleType::Buffer,
               buffer_idx_gltf as u64,
            );
            let ori_handle = gpu_raw_buffers.get(&idx_buff_handle).unwrap();
            assert!(gpu_raw_buffers.contains_key(&(idx_buff_handle)));
            let mut buffer_idx = ori_handle.raw();

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
                let idx_buff_handle = handle::ResourceHandle::new(
                    handle::ResourceHandleType::Buffer,
                    buffer_idx_gltf as u64,
                );
                let raw_buffer = raw_buffers.get(&(idx_buff_handle)).unwrap();
                let indices: &[u16] = bytemuck::cast_slice(&raw_buffer[total_offset..]);
                for i in 0..count {
                    idx_buffer_32.push(indices[i as usize] as u32);
                }

                //create a wgpu buffer
                let buff_handle= buffer_manager.create_buffer_with_data(
                    bytemuck::cast_slice(&idx_buffer_32[..]),
                    wgpu::BufferUsage::INDEX,
                    gpu_interfaces
                );

                buffer_idx = buff_handle.raw();
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
    gpu_raw_buffers: &mut HashMap<handle::ResourceHandle, handle::ResourceHandle>,
    raw_buffers: &HashMap<handle::ResourceHandle, Vec<u8>>,
    gpu_interfaces: &api::GPUInterfaces,
    buffer_manager: &mut buffer::BufferManager,
) -> Vec<Mesh> {
    let primitives = mesh.primitives();
    let mut meshes = Vec::new();
    for primitive in primitives {
        let mesh =
            load_gltf_mesh_primitive(&primitive, gpu_raw_buffers, raw_buffers, gpu_interfaces, buffer_manager);
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
    let mut gpu_raw_buffers = HashMap::new();

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

        //let wgpu_buffer = gpu_interfaces.device.create_buffer_with_data(
        //    bytemuck::cast_slice(&buffer_content[..]),
        //    wgpu::BufferUsage::INDEX | wgpu::BufferUsage::VERTEX,
        //);
        let gpu_buff_handle= buffer_manager.create_buffer_with_data(
            bytemuck::cast_slice(&buffer_content[..]),
            wgpu::BufferUsage::INDEX | wgpu::BufferUsage::VERTEX,
            gpu_interfaces,
        );

        let buff_handle =
            handle::ResourceHandle::new(handle::ResourceHandleType::Buffer, buffer_idx as u64);
        raw_buffers.insert(buff_handle, buffer_content);
        gpu_raw_buffers.insert(buff_handle, gpu_buff_handle);
    }

    let mut models = Vec::new();
    for mesh in gltf.meshes() {
        let meshes = load_gltf_mesh(&mesh, &mut gpu_raw_buffers, &raw_buffers, gpu_interfaces, buffer_manager);

        let model = Model { meshes };
        models.push(model);
    }

    GltfFile {
        models,
    }
}
