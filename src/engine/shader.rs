use std::collections::HashMap;

use super::platform;
use super::platform::file_system;

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
    shader_counter: u64,
}

//I tried to limit this to a minimum, unluckily here I can only compile shaders on native
//not browser (as of now ). Splitting between native and wasm was a bit overkill so for
//now it was split here. The wasm32 version should never be called and just return and empty
//value to keep the compiler happy. I will revisit in the future.
//One option is to expose the compile shader function as a trait and then pull this 
//correctly from the native module
//#[cfg(not(target_arch = "wasm32"))]

//#[cfg(target_arch = "wasm32")]

impl ShaderManager {
    pub fn new() -> Self {
        let shader_mapper: HashMap<u64, Shader> = HashMap::new();
        Self {
            shader_mapper,
            shader_counter: 0,
        }
    }

    pub async fn load_shader_type(
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
        let spv_exists = match platform::core::get_platform() {
            //if we are in the browser we can only load spv, so we force the file to
            //exists and we will try to download it, we could use the file_exists for wasm
            //but is an expensive download, so we just try to download it later on. The
            //function exists mostly for simmetry between native and wasm
            platform::Platform::BROWSER => true,
            platform::Platform::NATIVE => file_system::file_exists(&spv).await,
        };

        let file_name = if spv_exists { spv } else { shader_file };
        let binary_data: Vec<u32>;

        if !spv_exists {
            binary_data = platform::shader::compile_shader(&file_name, &shader_type).await;
        } else {
            let contents = file_system::load_file_u8(&file_name).await.unwrap();
            binary_data = wgpu::read_spirv(std::io::Cursor::new(&contents[..])).unwrap()
        }

        let module = device.create_shader_module(&binary_data);

        let shader = Shader {
            shader_type,
            module,
        };

        self.shader_counter += 1;
        self.shader_mapper.insert(self.shader_counter, shader);

        handle::ResourceHandle::new(handle::ResourceHandleType::SHADER, self.shader_counter)
    }

    //TODO investigate should pass the hande by value? will it get trivially copied?
    pub fn get_shader_module(
        &self,
        handle: &handle::ResourceHandle,
    ) -> Result<&wgpu::ShaderModule, &'static str> {
        //assert is the correct type

        let value = handle.get_value();

        let module = match self.shader_mapper.get(&value) {
            Some(shader) => &shader.module,
            None => return Err("Error finding shader"),
        };

        Ok(&module)
    }
}

//glslangValidator shader.frag -o shader.frag.spv -V100
