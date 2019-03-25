use shaderc::ShaderKind;

use crate::mesh::{Mesh, MeshLoadError};
use crate::shader::ShaderCompilationError;
use crate::shader::load_shader;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error(display = "loading mesh failed")]
    MeshLoadFailed(#[error(cause)] MeshLoadError),
    #[error(display = "loading shader failed")]
    ShaderLoadFailed(#[error(cause)] ShaderCompilationError),
}

impl From<MeshLoadError> for AssetError {
    fn from(err: MeshLoadError) -> Self {
        AssetError::MeshLoadFailed(err)
    }
}

impl From<ShaderCompilationError> for AssetError {
    fn from(err: ShaderCompilationError) -> Self {
        AssetError::ShaderLoadFailed(err)
    }
}

pub struct Assets {
    pub cube: Mesh,
    pub vertex_shader: Vec<u8>,
    pub fragment_shader: Vec<u8>,
}

impl Assets {
    pub fn load() -> Result<Assets, AssetError> {
        let cube = Mesh::load("assets/cube.glb")?;
        let vertex_shader = load_shader("assets/cube.vert.glsl", ShaderKind::Vertex)?;
        let fragment_shader = load_shader("assets/cube.frag.glsl", ShaderKind::Fragment)?;

        Ok(Assets {
            cube,
            vertex_shader,
            fragment_shader
        })
    }
}