use crate::engine::graphics::shader::ShaderType;
use super::file_system;

pub async fn compile_shader(file_name: &str, shader_type: &ShaderType) -> Vec<u32> {
    let compile_shader_type = match shader_type {
        ShaderType::VERTEX => shaderc::ShaderKind::Vertex,
        ShaderType::FRAGMENT => shaderc::ShaderKind::Fragment,
    };

    let contents = file_system::load_file_string(&file_name)
        .await
        .expect("Something went wrong reading the shader source file");
    //generating the spv, does not work on browser context
    let mut compiler = shaderc::Compiler::new().unwrap();
    let spv_code = compiler
        .compile_into_spirv(
            &contents[..],
            compile_shader_type,
            &file_name[..],
            "main",
            None,
        )
        .unwrap();

    wgpu::read_spirv(std::io::Cursor::new(spv_code.as_binary_u8())).unwrap()
}