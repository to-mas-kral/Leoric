use std::mem::size_of;

use eyre::{eyre, Result};
use glam::{Vec2, Vec3, Vec4};
use gltf::{
    image::Format,
    mesh::util::ReadIndices,
    texture::{MagFilter, MinFilter, WrappingMode},
};

use super::{DataBundle, Indices, Texture};

/// A Primitive represents a single 'mesh' in the normal meaning of that word
/// (a collection of vertices with a specific topology like Trianglesd or Lines)
pub struct Primitive {
    pub texture_info: PrimTexInfo,
    pub vao: Option<u32>,

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
            .map(|p| Vec3::from(p))
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

            texcoords = texcoords_reader
                .into_f32()
                .map(|tc| Vec2::from(tc))
                .collect();

            texture_set += 1;
        }

        let normals = reader
            .read_normals()
            .ok_or(eyre!("primitive doesn't containt normals"))?
            .map(|n| Vec3::from(n))
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
            vao: None,
            texture_info: PrimTexInfo::None {
                base_color_factor: Vec4::splat(1.),
            },
            indices,
            positions,
            texcoords,
            normals,
            skin,
        };

        primitive.create_buffers(&material, bundle);

        Ok(primitive)
    }

    // Indices of the vertex attributes
    const POS_INDEX: u32 = 0;
    const TEXCOORDS_INDEX: u32 = 1;
    const NORMALS_INDEX: u32 = 2;
    const JOINTS_INDEX: u32 = 3;
    const WEIGHTS_INDEX: u32 = 4;

    fn create_buffers(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let mut indices = 0;
        let mut vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let _positions = Self::create_float_buf(&self.positions, 3, Self::POS_INDEX, gl::FLOAT);
            let _texcoords =
                Self::create_float_buf(&self.texcoords, 2, Self::TEXCOORDS_INDEX, gl::FLOAT);
            let _normals = Self::create_float_buf(&self.normals, 3, Self::NORMALS_INDEX, gl::FLOAT);

            if let Some(skin) = &self.skin {
                let _joints =
                    Self::create_int_buf(&skin.joints, 4, Self::JOINTS_INDEX, gl::UNSIGNED_INT);
                let _weights =
                    Self::create_float_buf(&skin.weights, 4, Self::WEIGHTS_INDEX, gl::FLOAT);
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
                    let texture =
                        self.create_texture(&tex_info.texture(), pbr.base_color_factor(), bundle);
                    PrimTexInfo::Some(texture)
                }
                None => {
                    let base_color_factor = Vec4::from(pbr.base_color_factor());
                    PrimTexInfo::None { base_color_factor }
                }
            };

            // Unbind buffers
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            self.vao = Some(vao);
            self.texture_info = texture_index;
        }
    }

    /// Create an opengl buffer with floating-point content.
    ///
    /// 'buffer' is a reference to a slice of T.
    ///
    /// 'components', 'attrib index' and 'typ' have the same meaning as the respective
    /// arguments in glVertexAttribPointer.
    fn create_float_buf<T: Copy>(
        buffer: &[T],
        components: i32,
        attrib_index: u32,
        typ: u32,
    ) -> u32 {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id as *mut _);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);

            let buffer_size = buffer.len() * size_of::<T>();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                buffer_size as isize,
                // TODO: check for safety - the layout of Vec3 is #[repr(C)] (struct of 3 floats),
                // so it should be correct
                buffer.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(attrib_index, components, typ, gl::FALSE, 0, 0 as _);
            gl::EnableVertexAttribArray(attrib_index);
        }

        id
    }

    /// Create an opengl buffer with integer content.
    ///
    /// 'buffer' is a reference to a slice of T.
    ///
    /// 'components', 'attrib index' and 'typ' have the same meaning as the respective
    /// arguments in glVertexAttribPointer.
    fn create_int_buf<T: Copy>(buffer: &[T], components: i32, attrib_index: u32, typ: u32) -> u32 {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id as *mut _);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);

            let buffer_size = buffer.len() * size_of::<T>();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                buffer_size as isize,
                // TODO: check for safety - the layout of Vec3 is #[repr(C)] (struct of 3 floats),
                // so it should be correct
                buffer.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribIPointer(attrib_index, components, typ, 0, 0 as _);
            gl::EnableVertexAttribArray(attrib_index);
        }

        id
    }

    fn create_texture(
        &mut self,
        tex: &gltf::Texture,
        base_color_factor: [f32; 4],
        bundle: &mut DataBundle,
    ) -> Texture {
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

        let texture = Texture::new(gl_tex_id, Vec4::from(base_color_factor));
        bundle.gl_textures[tex_index] = Some(texture.clone());
        texture
    }

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

/// Texture info for a primitive
/// If the primitive has a texture, copy the texture information from the Model's gl_textures
/// If not, the base_color_factor serves as the object color
pub enum PrimTexInfo {
    None { base_color_factor: Vec4 },
    Some(Texture),
}

pub struct PrimSkin {
    pub joints: Vec<[u32; 4]>,
    pub weights: Vec<[f32; 4]>,
}

impl PrimSkin {
    pub fn new(joints: Vec<[u32; 4]>, weights: Vec<[f32; 4]>) -> Self {
        Self { joints, weights }
    }
}
