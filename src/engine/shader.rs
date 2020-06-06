use std::fs;

const SPIRV_EXT: &'static str  = ".spv";
pub enum ShaderType
{
    VERTEX,
    FRAGMENT
}

pub fn load_shader(filename: &str) -> String 
{
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    contents
}

fn file_exists(file_name: &str) -> bool
{
    return std::path::Path::new(file_name).exists()
}

//glslangValidator shader.frag -o shader.frag.spv -V100 

pub fn load_shader_type(shader_name: &str, shader_type: ShaderType) -> String
{
    //first we want to check of an spir-v variant exists, that will save us 
    //time at runtime (also compiling won't work in browser anyway)
    
    //we need to get the extention
    let ext = match shader_type{
        ShaderType::VERTEX => ".vert",
        ShaderType::FRAGMENT => ".frag"
    };

    let shader_file = format!("{}{}",shader_name, ext);
    let spv =   format!("{}{}",&shader_file[..], SPIRV_EXT);
    //let spv_exists = file_exists(shader_name);
    let spv_exists = false;
    let file_name = if spv_exists {  spv } else{ shader_file};
     println!("{}",file_name);

    let contents = fs::read_to_string(file_name)
        .expect("Something went wrong reading the file");

    contents

}
