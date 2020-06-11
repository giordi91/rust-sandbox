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
    shader_counter: u64,
}
    #[cfg(target_arch = "wasm32")]
    macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit, RequestMode, Response};


#[cfg(target_arch = "wasm32")]
pub async fn load_file_wasm(url : &String) -> Result<Vec<u32>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    //TODO investigate, any way to have less jumps around?
    //we extract the array buffer from our response 
    let t : JsValue = JsFuture::from(resp.array_buffer()?).await?;
    //we initialize the array buffer out of the JsValue
    let bt :js_sys::ArrayBuffer = js_sys::ArrayBuffer::from(t);
    //next we use the array buffer to initialize a uin32Array
    let typebuf: js_sys::Uint32Array = js_sys::Uint32Array::new(&bt);
    //finally we copy this array into the final vector
    let mut body = vec![0 as u32; typebuf.length() as usize];
    typebuf.copy_to(&mut body);
    Ok(body)
}

impl ShaderManager {
    pub fn new() -> Self {
        let shader_mapper: HashMap<u64, Shader> = HashMap::new();
        Self {
            shader_mapper,
            shader_counter: 0,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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
        //let spv_exists = file_exists(&spv);
        let spv_exists = file_exists(&spv);
        let file_name = if spv_exists { spv } else { shader_file };
        let binary_data: Vec<u32>;

        if !spv_exists {
            let contents = fs::read_to_string(&file_name)
                .expect("Something went wrong reading the shader source file");
            //generating the spv, does not work on browser context
            let mut compiler = shaderc::Compiler::new().unwrap();
            let spv_code = 
                compiler
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

    #[cfg(target_arch = "wasm32")]
    pub async fn load_shader_type(
        &mut self,
        device: &wgpu::Device,
        shader_name: &str,
        shader_type: ShaderType,
    ) ->  handle::ResourceHandle {
       log!("from shader"); 
        //we need to get the extention
        let ext = match shader_type {
            ShaderType::VERTEX => ".vert",
            ShaderType::FRAGMENT => ".frag",
        };

        let shader_file = format!("{}{}", shader_name, ext);
        let spv = format!("{}{}", &shader_file[..], SPIRV_EXT);

       let content= load_file_wasm(&spv).await.unwrap();

        let module = device.create_shader_module(&content);

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

#[cfg(not(target_arch = "wasm32"))]
fn get_wgpu_shader_kind(shader_type: &ShaderType) -> shaderc::ShaderKind {
    match shader_type {
        ShaderType::VERTEX => shaderc::ShaderKind::Vertex,
        ShaderType::FRAGMENT => shaderc::ShaderKind::Fragment,
    }
}

//glslangValidator shader.frag -o shader.frag.spv -V100
