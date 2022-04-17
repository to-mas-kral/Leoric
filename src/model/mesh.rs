use std::mem::size_of;

use eyre::{eyre, Result};
use gl::types::GLenum;
use glam::{Vec2, Vec3, Vec4};
use gltf::{
    image::Format,
    mesh::util::ReadIndices,
    texture::{MagFilter, MinFilter, WrappingMode},
};

use crate::ogl;

use super::DataBundle;

/// A gltf 'Mesh' contains multiple real sub-meshes (called Primitives in the gltf parlance)
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub name: Option<String>,
}

impl Mesh {
    pub fn from_gltf(mesh: &gltf::Mesh, bundle: &mut DataBundle) -> Result<Self> {
        let name = mesh.name().map(|n| n.to_owned());

        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let primitive = Primitive::from_gltf(&primitive, bundle)?;
            primitives.push(primitive);
        }

        Ok(Mesh { primitives, name })
    }
}

/// A Primitive represents a single 'mesh' in the normal meaning of that word
/// (a collection of vertices with a specific topology like Triangles or Lines).
pub struct Primitive {
    pub texture_info: PrimitiveTexture,
    pub vao: u32,

    pub indices: Indices,
    pub positions: Vec<Vec3>,
    pub texcoords: Vec<Vec2>,
    pub normals: Vec<Vec3>,
    pub skin: Option<PrimSkin>,
}

impl Primitive {
    pub fn from_gltf(primitive: &gltf::Primitive, bundle: &mut DataBundle) -> Result<Self> {
        let mode = primitive.mode();

        if mode != gltf::mesh::Mode::Triangles {
            return Err(eyre!("primitive mode: '{mode:?}' is not impelemnted"));
        }

        let reader = primitive.reader(|buffer| Some(&bundle.buffers[buffer.index()]));
        let positions = reader
            .read_positions()
            .ok_or(eyre!("primitive doesn't containt positions"))?
            .map(Vec3::from)
            .collect();

        let indices = match reader
            .read_indices()
            .ok_or(eyre!("primitive doesn't containt indices"))?
        {
            ReadIndices::U32(b) => Indices::U32(b.collect()),
            ReadIndices::U16(b) => Indices::U16(b.collect()),
            ReadIndices::U8(b) => Indices::U8(b.collect()),
        };

        let mut texcoords = Vec::new();
        let mut texture_set = 0;
        while let Some(texcoords_reader) = reader.read_tex_coords(texture_set) {
            if texture_set >= 1 {
                //eprintln!("WARN: primitive has more than 1 texture coordinate set");
                break;
            }

            texcoords = texcoords_reader.into_f32().map(Vec2::from).collect();

            texture_set += 1;
        }

        let normals = reader
            .read_normals()
            .ok_or(eyre!("primitive doesn't containt normals"))?
            .map(Vec3::from)
            .collect();

        let skin = match (reader.read_joints(0), reader.read_weights(0)) {
            (Some(joints), Some(weights)) => {
                let joints = joints.into_u16().map(|j| j.map(|ji| ji as u32)).collect();
                // TODO: u8 / u16 joint weights normalization
                match weights {
                    gltf::mesh::util::ReadWeights::U8(_) => todo!("U8 weights"),
                    gltf::mesh::util::ReadWeights::U16(_) => todo!("U16 weights"),
                    _ => {}
                }
                let weights = weights.into_f32().collect();

                Some(PrimSkin::new(joints, weights))
            }
            _ => None,
        };

        let material = primitive.material();

        let mut primitive = Self {
            vao: 0,
            texture_info: PrimitiveTexture::None {
                base_color_factor: Vec4::splat(1.),
            },
            indices,
            positions,
            texcoords,
            normals,
            skin,
        };

        primitive.create_buffers(&material, bundle);

        if primitive.vao == 0 {
            return Err(eyre!("primitive VAO wasn't correctly initialized"));
        }

        Ok(primitive)
    }

    fn create_buffers(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let mut indices = 0;
        let mut vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let _positions = ogl::create_float_buf(&self.positions, 3, ogl::POS_INDEX, gl::FLOAT);
            let _texcoords =
                ogl::create_float_buf(&self.texcoords, 2, ogl::TEXCOORDS_INDEX, gl::FLOAT);
            let _normals = ogl::create_float_buf(&self.normals, 3, ogl::NORMALS_INDEX, gl::FLOAT);

            if let Some(skin) = &self.skin {
                let _joints =
                    ogl::create_int_buf(&skin.joints, 4, ogl::JOINTS_INDEX, gl::UNSIGNED_INT);
                let _weights =
                    ogl::create_float_buf(&skin.weights, 4, ogl::WEIGHTS_INDEX, gl::FLOAT);
            }

            // Indices
            gl::GenBuffers(1, &mut indices);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.indices.size() as isize,
                self.indices.ptr(),
                gl::STATIC_DRAW,
            );

            let pbr = material.pbr_metallic_roughness();
            let texture_index = match pbr.base_color_texture() {
                Some(tex_info) => {
                    self.create_texture(&tex_info.texture(), pbr.base_color_factor(), bundle)
                }
                None => {
                    let base_color_factor = Vec4::from(pbr.base_color_factor());
                    PrimitiveTexture::None { base_color_factor }
                }
            };

            // Unbind buffers
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            self.vao = vao;
            self.texture_info = texture_index;
        }
    }

    /// Creates a new OpenGL texture.
    ///
    /// If the texture already exists (bundle.gl_textures\[texture_index\] == Some(...)),
    /// no new texture is created, only the Texture struct is cloned.
    fn create_texture(
        &mut self,
        tex: &gltf::Texture,
        base_color_factor: [f32; 4],
        bundle: &mut DataBundle,
    ) -> PrimitiveTexture {
        let tex_index = tex.source().index();
        if let Some(texture) = bundle.gl_textures[tex_index].clone() {
            return texture;
        }

        let gl_tex_id = unsafe {
            let mut texture = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            self.set_texture_sampler(&tex.sampler());

            let image = &bundle.images[tex_index];

            assert!(image.width.is_power_of_two());
            assert!(image.height.is_power_of_two());

            let (internal_format, format) = match image.format {
                Format::R8G8 => (gl::RG8, gl::RG),
                Format::R8G8B8 => (gl::RGB8, gl::RGB),
                Format::R8G8B8A8 => (gl::RGBA8, gl::RGBA),
                f => unimplemented!("Unimplemented image format: '{f:?}'"),
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                image.pixels.as_ptr() as _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            texture
        };

        let texture = PrimitiveTexture::Some {
            gl_id: gl_tex_id,
            base_color_factor: Vec4::from(base_color_factor),
        };
        bundle.gl_textures[tex_index] = Some(texture.clone());
        texture
    }

    /// Sets the appropriate sampler functions for the currently created texture.
    fn set_texture_sampler(&self, sampler: &gltf::texture::Sampler) {
        let min_filter = match sampler.min_filter() {
            Some(min_filter) => match min_filter {
                MinFilter::Nearest => gl::NEAREST,
                MinFilter::Linear => gl::LINEAR,
                MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
                MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
                MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
                MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
            },
            None => gl::LINEAR_MIPMAP_LINEAR,
        };

        let mag_filter = match sampler.mag_filter() {
            Some(mag_filter) => match mag_filter {
                MagFilter::Nearest => gl::NEAREST,
                MagFilter::Linear => gl::LINEAR,
            },
            None => gl::LINEAR,
        };

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
        }

        let wrap_s = match sampler.wrap_s() {
            WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
            WrappingMode::Repeat => gl::REPEAT,
        };

        let wrap_t = match sampler.wrap_t() {
            WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
            WrappingMode::Repeat => gl::REPEAT,
        };

        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);
        }
    }
}

/// Texture info for a primitive.
///
/// If the primitive has a texture, copy the texture information from the Model's gl_textures.
///
/// If not, the base_color_factor serves as the object color.
#[derive(Clone)]
pub enum PrimitiveTexture {
    None { base_color_factor: Vec4 },
    Some { gl_id: u32, base_color_factor: Vec4 },
}

/// Optional skin data for a primitive.
pub struct PrimSkin {
    pub joints: Vec<[u32; 4]>,
    pub weights: Vec<[f32; 4]>,
}

impl PrimSkin {
    pub fn new(joints: Vec<[u32; 4]>, weights: Vec<[f32; 4]>) -> Self {
        Self { joints, weights }
    }
}

/// Vertex indices for a primitive.
///
/// Better than using generics here.
pub enum Indices {
    U32(Vec<u32>),
    U16(Vec<u16>),
    U8(Vec<u8>),
}

impl Indices {
    pub fn size(&self) -> usize {
        match self {
            Indices::U32(buf) => buf.len() * size_of::<u32>(),
            Indices::U16(buf) => buf.len() * size_of::<u16>(),
            Indices::U8(buf) => buf.len() * size_of::<u8>(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Indices::U32(buf) => buf.len(),
            Indices::U16(buf) => buf.len(),
            Indices::U8(buf) => buf.len(),
        }
    }

    pub fn ptr(&self) -> *const std::ffi::c_void {
        match self {
            Indices::U32(buf) => buf.as_ptr() as _,
            Indices::U16(buf) => buf.as_ptr() as _,
            Indices::U8(buf) => buf.as_ptr() as _,
        }
    }

    pub fn gl_type(&self) -> GLenum {
        match self {
            Indices::U32(_) => gl::UNSIGNED_INT,
            Indices::U16(_) => gl::UNSIGNED_SHORT,
            Indices::U8(_) => gl::UNSIGNED_BYTE,
        }
    }
}
