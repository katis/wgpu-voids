use cgmath::{Vector3, Point2};
use itertools::izip;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    tex_coord: Point2<f32>,
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
                wgpu::VertexAttributeDescriptor {
                    attribute_index: 2,
                    format: wgpu::VertexFormat::Float2,
                    offset: (size_of::<Vector3<f32>>() * 2) as u32,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
    pub texture: Texture,
}

#[derive(Debug)]
pub struct Texture {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }
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

        let base_color_texture = primitive.material()
            .pbr_metallic_roughness()
            .base_color_texture()
            .ok_or_else(|| NoBaseColorTexture {
                mesh: mesh_name(&mesh_doc),
            })?;

        let texture: Texture = match base_color_texture.texture().source().source() {
            gltf::image::Source::View { view, mime_type } => {
                if mime_type != "image/png" {
                    return Err(UnsupportedImageFormat { mime_type: mime_type.to_string() });
                }
                let bytes = access_bytes(&buffers, &view);
                load_png(bytes)?
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                unimplemented!("Can't load external files: {}", uri);
            }
        };

        let tex_coord_bytes = attribute_bytes(&buffers, &mesh_doc, &primitive, Semantic::TexCoords(base_color_texture.tex_coord()))?;
        let mut tex_coords: Vec<_> = bytes_to_point2(tex_coord_bytes).iter().map(|p| Point2::new(p.x, p.y)).collect();
        //let mut tex_coords = bytes_to_point2(tex_coord_bytes);

        let indices_doc = primitive.indices().ok_or_else(|| NoIndices { mesh: mesh_name(&mesh_doc) })?;
        let index_bytes = access_bytes(&buffers, &indices_doc.view());
        let indices = bytes_to_u16(index_bytes);

        let position_bytes = attribute_bytes(&buffers, &mesh_doc, &primitive, Semantic::Positions)?;
        let positions = bytes_to_vector3(position_bytes);

        println!("POSITIONS: {:?}", positions);

        let normal_bytes = attribute_bytes(&buffers, &mesh_doc, &primitive, Semantic::Normals)?;
        let normals = bytes_to_vector3(normal_bytes);

        let vertices: Vec<Vertex> = izip!(positions.into_iter(), normals.into_iter(), tex_coords.into_iter())
            .map(|(&position, &normal, tex_coord)| Vertex {
                position,
                normal,
                tex_coord,
            }).collect();

        Ok(Mesh {
            indices: indices.to_vec(),
            vertices,
            texture,
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
    #[error(display = "mesh {} has no base color texture", mesh)]
    NoBaseColorTexture {
        mesh: String,
    },
    #[error(display = "unknown image format {}", mime_type)]
    UnsupportedImageFormat {
        mime_type: String,
    },
    #[error(display = "decoding image failed")]
    ImageDecodeFailed(#[error(cause)] png::DecodingError),
}

impl From<gltf::Error> for MeshLoadError {
    fn from(err: gltf::Error) -> Self {
        MeshLoadError::InvalidImport(err)
    }
}

impl From<png::DecodingError> for MeshLoadError {
    fn from(err: png::DecodingError) -> Self {
        MeshLoadError::ImageDecodeFailed(err)
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
    let doc = primitive.get(&semantic)
        .ok_or_else(|| MeshLoadError::NoSemantic { mesh: mesh_name(mesh), semantic })?;
    Ok(access_bytes(buffers, &doc.view()))
}

fn access_bytes<'a>(buffers: &'a [gltf::buffer::Data], view: &gltf::buffer::View) -> &'a [u8] {
    let buffer_i = view.buffer().index();
    let buffer = &buffers[buffer_i];

    let start_i = view.offset();
    let end_i = view.offset() + view.length();

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

fn bytes_to_point2(bytes: &[u8]) -> &[Point2<f32>] {
    use ::std::mem;
    unsafe {
        ::std::slice::from_raw_parts(
            bytes.as_ptr() as *const Point2<f32>,
            bytes.len() / mem::size_of::<Point2<f32>>())
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

fn load_png(bytes: &[u8]) -> Result<Texture, png::DecodingError> {
    use png::Decoder;

    let decoder = Decoder::new(bytes);

    let (info, mut reader) = decoder.read_info()?;

    let mut pixels = vec![0; info.buffer_size()];
    reader.next_frame(&mut pixels)?;

    Ok(Texture {
        width: info.width,
        height: info.height,
        pixels,
    })
}