use std::fs;
use std::io::prelude::*;
use super::file_system;
use crate::engine::graphics::shader::ShaderType;

fn resolve_include(
    include_file: &str,
    include_type: shaderc::IncludeType,
    _file_using_include: &str,
    _include_depth: usize,
) -> Result<shaderc::ResolvedInclude, String> {
    match include_type {
        shaderc::IncludeType::Relative => {
            let content = fs::read_to_string(include_file).unwrap();
            return Ok(shaderc::ResolvedInclude {
                resolved_name: String::from(include_file),
                content,
            });
        }
        _ => panic!("unsupported include type for shader"),
    }
}

pub async fn compile_shader(file_name: &str, shader_type: &ShaderType) -> Vec<u32> {
    let compile_shader_type = match shader_type {
        ShaderType::VERTEX => shaderc::ShaderKind::Vertex,
        ShaderType::FRAGMENT => shaderc::ShaderKind::Fragment,
        ShaderType::COMPUPTE => shaderc::ShaderKind::Compute,
    };

    let contents = file_system::load_file_string(&file_name)
        .await
        .expect("Something went wrong reading the shader source file");
    //generating the spv, does not work on browser context
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_include_callback(resolve_include);
    let mut compiler = shaderc::Compiler::new().unwrap();
    let spv_code = compiler
        .compile_into_spirv(
            &contents[..],
            compile_shader_type,
            &file_name[..],
            "main",
            Some(&options),
        )
        .unwrap();

    let compiled_code = spv_code.as_binary_u8();
    let mut file = fs::File::create(format!("{}.spv",file_name)).unwrap();
    file.write_all(compiled_code).unwrap();
    wgpu::read_spirv(std::io::Cursor::new(compiled_code)).unwrap()
}
