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
    name_to_id: HashMap<String, AssetId<T>>,
    assets: Vec<T>,
}

impl<T> AssetStore<T> {
    pub fn new() -> AssetStore<T> {
        AssetStore {
            name_to_id: HashMap::new(),
            assets: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: &str, asset: T) -> AssetId<T> {
        let id = AssetId(self.assets.len() as u32, PhantomData);
        self.name_to_id.insert(name.to_string(), id);
        self.assets.push(asset);
        id
    }

    pub fn get_id(&self, name: &str) -> Option<AssetId<T>> {
        self.name_to_id.get(name).map(|id| *id)
    }

    pub fn find(&self, asset_name: &str) -> Option<&T> {
        self.name_to_id.get(asset_name).map(|id| &self.assets[id.0 as usize])
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AssetId<T>(u32, PhantomData<T>);

impl<T> Copy for AssetId<T> {}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        AssetId(self.0, PhantomData)
    }
}
