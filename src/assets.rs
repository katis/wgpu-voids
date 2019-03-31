use shaderc::ShaderKind;

use crate::model_data::{ModelData, ModelLoadError};
use crate::shader::ShaderCompilationError;
use crate::shader::load_shader;
use std::marker::PhantomData;
use hashbrown::hash_map::HashMap;
use std::borrow::Cow;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error(display = "loading mesh failed")]
    MeshLoadFailed(#[error(cause)] ModelLoadError),
    #[error(display = "loading shader failed")]
    ShaderLoadFailed(#[error(cause)] ShaderCompilationError),
}

impl From<ModelLoadError> for AssetError {
    fn from(err: ModelLoadError) -> Self {
        AssetError::MeshLoadFailed(err)
    }
}

impl From<ShaderCompilationError> for AssetError {
    fn from(err: ShaderCompilationError) -> Self {
        AssetError::ShaderLoadFailed(err)
    }
}

pub struct Assets {
    pub models: AssetStore<ModelData>,
    pub shaders: AssetStore<Vec<u8>>,
}

impl Assets {
    pub fn load() -> Result<Assets, AssetError> {
        let cube = ModelData::load("assets/cube.glb")?;
        let mut models = AssetStore::new();
        models.insert("cube", cube);

        let vertex_shader = load_shader("assets/cube.vert.glsl", ShaderKind::Vertex)?;
        let fragment_shader = load_shader("assets/cube.frag.glsl", ShaderKind::Fragment)?;
        let mut shaders = AssetStore::new();
        shaders.insert("vertex", vertex_shader);
        shaders.insert("fragment", fragment_shader);

        Ok(Assets {
            models,
            shaders,
        })
    }
}

pub struct AssetStore<T> {
    assets: HashMap<String, T>,
}

impl<T> AssetStore<T> {
    pub fn new() -> AssetStore<T> {
        AssetStore {
            assets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: &str, asset: T) {
        self.assets.insert(name.to_string(), asset);
    }

    pub fn find(&self, asset_name: &str) -> Option<&T> {
        self.assets.get(asset_name)
    }
}
