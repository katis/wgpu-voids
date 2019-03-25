use std::io;

use shaderc::{Compiler, ShaderKind};
use shaderc::CompilationArtifact;
use wgpu::{Device, ShaderModule};

#[derive(Debug, Error)]
pub enum ShaderCompilationError {
    #[error(display = "could not create shader compiler")]
    NullCompiler,
    #[error(display = "could not create shader compile options")]
    NullOptions,
    #[error(display = "shader compilation error")]
    CompileFailed(#[error(cause)] shaderc::Error),
    #[error(display = "could not read shader source file")]
    FileError(#[error(cause)] std::io::Error),
}

impl From<shaderc::Error> for ShaderCompilationError {
    fn from(err: shaderc::Error) -> Self {
        ShaderCompilationError::CompileFailed(err)
    }
}

impl From<io::Error> for ShaderCompilationError {
    fn from(err: io::Error) -> Self {
        ShaderCompilationError::FileError(err)
    }
}

fn glsl_to_spirv(source: &str, shader_kind: ShaderKind) -> Result<CompilationArtifact, ShaderCompilationError> {
    use self::ShaderCompilationError::*;

    let mut compiler = Compiler::new().ok_or_else(|| NullCompiler)?;
    let mut options = shaderc::CompileOptions::new().ok_or_else(|| NullOptions)?;
    options.add_macro_definition("EP", Some("main"));
    let artifact = compiler.compile_into_spirv(source, shader_kind, "shader.glsl", "main", Some(&options))?;
    Ok(artifact)
}

pub fn load_shader(path: &str, shader_kind: ShaderKind) -> Result<Vec<u8>, ShaderCompilationError> {
    let shader_source = ::std::fs::read_to_string(path)?;
    let artifact = glsl_to_spirv(&shader_source, shader_kind)?;
    Ok(artifact.as_binary_u8().to_vec())
}
