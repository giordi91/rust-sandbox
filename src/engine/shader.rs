use std::collections::HashMap;
use std::fs;

const SPIRV_EXT: &'static str = ".spv";
pub enum ShaderType {
    VERTEX,
    FRAGMENT,
}

use super::handle;

pub struct Shader {
    pub shader_type: ShaderType,
    pub module: wgpu::ShaderModule,
}

pub struct ShaderManager {
    shader_mapper: HashMap<u64, Shader>,
    compiler: shaderc::Compiler,
    shader_counter: u64,
}

impl ShaderManager {
    pub fn new() -> Self {
        let shader_mapper: HashMap<u64, Shader> = HashMap::new();
        let compiler = shaderc::Compiler::new().unwrap();
        Self {
            shader_mapper,
            compiler,
            shader_counter: 0,
        }
    }
    pub fn load_shader_type(
        &mut self,
        device: &wgpu::Device,
        shader_name: &str,
        shader_type: ShaderType,
    ) -> handle::ResourceHandle {
        //first we want to check of an spir-v variant exists, that will save us
        //time at runtime (also compiling won't work in browser anyway)

        //we need to get the extention
        let ext = match shader_type {
            ShaderType::VERTEX => ".vert",
            ShaderType::FRAGMENT => ".frag",
        };

        let shader_file = format!("{}{}", shader_name, ext);
        let spv = format!("{}{}", &shader_file[..], SPIRV_EXT);
        let spv_exists = file_exists(&spv);
        let file_name = if spv_exists { spv } else { shader_file };
        let binary_data: Vec<u32>;

        if !spv_exists {
            let contents = fs::read_to_string(&file_name)
                .expect("Something went wrong reading the shader source file");
            //generating the spv, does not work on browser context
            let spv_code = self
                .compiler
                .compile_into_spirv(
                    &contents[..],
                    get_wgpu_shader_kind(&shader_type),
                    &file_name[..],
                    "main",
                    None,
                )
                .unwrap();
            binary_data = wgpu::read_spirv(std::io::Cursor::new(spv_code.as_binary_u8())).unwrap();
        } else {
            let contents = fs::read(&file_name).expect("Something went wrong reading the spv file");
            binary_data = wgpu::read_spirv(std::io::Cursor::new(&contents[..])).unwrap()
        }

        let module = device.create_shader_module(&binary_data);

        let shader = Shader {
            shader_type,
            module,
        };

        self.shader_counter += 1;
        self.shader_mapper.insert(self.shader_counter, shader);

        handle::ResourceHandle::new(handle::ResourceHandleType::SHADER,self.shader_counter)
    }

    //TODO investigate should pass the hande by value? will it get trivially copied?
    pub fn get_shader_module(&self, handle: &handle::ResourceHandle) ->Result<&wgpu::ShaderModule, &'static str>
    {
        //assert is the correct type 

        let value = handle.get_value();
        
        let module = match self.shader_mapper.get(&value) {
            Some(shader) => &shader.module,
            None => return Err("Error finding shader") ,
        };

        Ok(&module)

    }
}

fn file_exists(file_name: &str) -> bool {
    return std::path::Path::new(file_name).exists();
}

fn get_wgpu_shader_kind(shader_type: &ShaderType) -> shaderc::ShaderKind {
    match shader_type {
        ShaderType::VERTEX => shaderc::ShaderKind::Vertex,
        ShaderType::FRAGMENT => shaderc::ShaderKind::Fragment,
    }
}

//glslangValidator shader.frag -o shader.frag.spv -V100
