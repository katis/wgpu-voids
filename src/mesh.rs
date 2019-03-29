use cgmath::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Vertex {
    pub fn buffer_descriptor() -> wgpu::VertexBufferDescriptor<'static> {
        use std::mem::size_of;

        wgpu::VertexBufferDescriptor {
            stride: size_of::<Vertex>() as u32,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 0,
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 1,
                    format: wgpu::VertexFormat::Float3,
                    offset: size_of::<Vector3<f32>>() as u32,
                },
            ]
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
}

impl Mesh {
    pub fn load(path: &str) -> Result<Mesh, MeshLoadError> {
        use self::MeshLoadError::*;
        use gltf::mesh::Semantic;

        let (document, buffers, _) = gltf::import(path)?;

        let mesh_doc = document.meshes().nth(0).ok_or_else(|| NoMesh { path: path.into() })?;
        let primitive = mesh_doc.primitives().nth(0).ok_or_else(|| NoPrimitives {
            mesh: mesh_name(&mesh_doc),
            path: path.into(),
        })?;
        let indices_doc = primitive.indices().ok_or_else(|| NoIndices { mesh: mesh_name(&mesh_doc) })?;
        let index_bytes = access_bytes(&buffers, &indices_doc);
        let indices = bytes_to_u16(index_bytes);

        let vertex_bytes = attribute_bytes(&buffers, &mesh_doc, &primitive, Semantic::Positions)?;
        let positions = bytes_to_vector3(vertex_bytes);

        let normal_bytes = attribute_bytes(&buffers, &mesh_doc, &primitive, Semantic::Normals)?;
        let normals = bytes_to_vector3(normal_bytes);

        let vertices: Vec<Vertex> = positions.into_iter().zip(normals.into_iter()).map(|(&position, &normal)| Vertex {
            position,
            normal
        }).collect();

        Ok(Mesh {
            indices: indices.to_vec(),
            vertices,
        })
    }
}

#[derive(Debug, Error)]
pub enum MeshLoadError {
    #[error(display = "could not import glTf file")]
    InvalidImport(#[error(cause)] gltf::Error),
    #[error(display = "file {} has no meshes", path)]
    NoMesh {
        path: String
    },
    #[error(display = "mesh {} of file {} has no meshes", mesh, path)]
    NoPrimitives {
        mesh: String,
        path: String,
    },
    #[error(display = "mesh {} has no indices", mesh)]
    NoIndices {
        mesh: String,
    },
    #[error(display = "mesh {} has no semantic {:?}", mesh, semantic)]
    NoSemantic {
        mesh: String,
        semantic: gltf::mesh::Semantic,
    },
}

impl From<gltf::Error> for MeshLoadError {
    fn from(err: gltf::Error) -> Self {
        MeshLoadError::InvalidImport(err)
    }
}

fn mesh_name(mesh: &gltf::mesh::Mesh) -> String {
    mesh.name().unwrap_or("<unknown>").to_string()
}

fn attribute_bytes<'a>(
    buffers: &'a [gltf::buffer::Data],
    mesh: &gltf::mesh::Mesh,
    primitive: &gltf::Primitive,
    semantic: gltf::mesh::Semantic,
) -> Result<&'a [u8], MeshLoadError> {
    let (_, doc) = primitive.attributes()
        .find(|(semantic, _)| semantic == semantic)
        .ok_or_else(|| MeshLoadError::NoSemantic { mesh: mesh_name(mesh), semantic })?;
    Ok(access_bytes(buffers, &doc))
}

fn access_bytes<'a>(buffers: &'a [gltf::buffer::Data], accessor: &gltf::Accessor) -> &'a [u8] {
    let buffer_i = accessor.view().buffer().index();
    let buffer = &buffers[buffer_i];

    let start_i = accessor.view().offset();
    let end_i = accessor.view().offset() + accessor.view().length();

    &buffer[start_i..end_i]
}

fn bytes_to_u16(bytes: &[u8]) -> &[u16] {
    use ::std::mem;
    unsafe {
        ::std::slice::from_raw_parts(
            bytes.as_ptr() as *const u16,
            bytes.len() / mem::size_of::<u16>())
    }
}

fn bytes_to_vector3(bytes: &[u8]) -> &[Vector3<f32>] {
    use ::std::mem;
    unsafe {
        ::std::slice::from_raw_parts(
            bytes.as_ptr() as *const Vector3<f32>,
            bytes.len() / mem::size_of::<Vector3<f32>>())
    }
}