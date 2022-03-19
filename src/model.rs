use std::mem::size_of;

use eyre::{eyre, Result};
use gl::types::GLenum;
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use gltf::{
    image::Format,
    mesh::util::ReadIndices,
    scene::Transform,
    texture::{MagFilter, MinFilter, WrappingMode},
};

/// Image and vertex data of the asset
pub struct DataBundle {
    /// Vertex data
    buffers: Vec<gltf::buffer::Data>,
    /// Texture data
    images: Vec<gltf::image::Data>,
    /// To keep track if which textures were already sent to the GPU
    pub gl_textures: Vec<Option<Texture>>,
}

impl DataBundle {
    fn new(buffers: Vec<gltf::buffer::Data>, images: Vec<gltf::image::Data>) -> Self {
        Self {
            buffers,
            gl_textures: vec![Option::None; images.len()],
            images,
        }
    }
}

/// This represents a top-level Node in a gltf hierarchy
pub struct Model {
    /// Texture data points to vectors in this bundle
    #[allow(unused)]
    pub bundle: DataBundle,
    /// An artifical root node
    pub root: Node,
}

impl Model {
    pub fn from_gltf(path: &str) -> Result<Model> {
        let (gltf, buffers, images) = gltf::import(path)?;

        let mut bundle = DataBundle::new(buffers, images);

        if gltf.scenes().len() != 1 {
            return Err(eyre!("GLTF file contains more than 1 scene"));
        }
        let scene = gltf.scenes().next().unwrap();

        let mut id = 1;
        let mut nodes = Vec::new();
        for node in scene.nodes() {
            let node = Node::from_gltf(&node, &mut bundle, &mut id)?;
            id += 1;
            nodes.push(node);
        }

        let root = Node {
            id: 0,
            children: nodes,
            mesh: None,
            transform: Mat4::IDENTITY,
            name: Some("Root".to_string()),
        };

        Ok(Model { bundle, root })
    }
}

pub struct Node {
    pub id: u32,
    pub children: Vec<Node>,
    pub mesh: Option<Mesh>,
    pub transform: Mat4,
    pub name: Option<String>,
}

impl Node {
    fn from_gltf(node: &gltf::Node, bundle: &mut DataBundle, id: &mut u32) -> Result<Self> {
        let mut children = Vec::new();

        let my_id = *id;

        for child_node in node.children() {
            *id += 1;
            let node = Node::from_gltf(&child_node, bundle, id)?;
            children.push(node);
        }

        let mesh = match node.mesh() {
            Some(m) => Some(Mesh::from_gltf(&m, bundle)?),
            None => None,
        };

        let transform = match node.transform() {
            Transform::Matrix { matrix } => Mat4::from_cols_array_2d(&matrix),
            Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Mat4::from_scale_rotation_translation(
                Vec3::from(scale),
                Quat::from_array(rotation),
                Vec3::from(translation),
            ),
        };

        Ok(Self {
            id: my_id,
            children,
            mesh,
            transform,
            name: node.name().map(|n| n.to_owned()),
        })
    }
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    name: Option<String>,
}

impl Mesh {
    fn from_gltf(mesh: &gltf::Mesh, bundle: &mut DataBundle) -> Result<Self> {
        let name = mesh.name().map(|n| n.to_owned());

        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let primitive = Primitive::from_gltf(&primitive, bundle)?;
            primitives.push(primitive);
        }

        Ok(Mesh { primitives, name })
    }
}

/// Better than using generics here
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

pub struct Primitive {
    /// Texture index into the DataBundle textures vector
    pub texture_index: Option<usize>,

    pub vao: Option<u32>,
    pub indices: Indices,
    pub positions: Vec<Vec3>,
    pub texcoords: Vec<Vec2>,
    pub normals: Vec<Vec3>,
}

impl Primitive {
    fn from_gltf(primitive: &gltf::Primitive, bundle: &mut DataBundle) -> Result<Self> {
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

        let material = primitive.material();

        let mut primitive = Self {
            vao: None,
            texture_index: None,
            indices,
            positions,
            texcoords,
            normals,
        };

        primitive.create_buffers(&material, bundle);

        Ok(primitive)
    }

    const POS_ATTRIB_INDEX: u32 = 0;
    const TEXCOORDS_ATTRIB_INDEX: u32 = 1;
    const NORMALS_ATTRIB_INDEX: u32 = 2;

    fn create_buffers(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let mut positions = 0;
        let mut texcoords = 0;
        let mut indices = 0;
        let mut normals = 0;
        let mut vao = 0;

        assert!(
            self.positions.len() == self.texcoords.len()
                && self.normals.len() == self.texcoords.len()
        );

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Positions
            Self::create_buf(
                &mut positions,
                &self.positions,
                3,
                Self::POS_ATTRIB_INDEX,
                gl::FLOAT,
            );

            // Texcoords
            Self::create_buf(
                &mut texcoords,
                &self.texcoords,
                2,
                Self::TEXCOORDS_ATTRIB_INDEX,
                gl::FLOAT,
            );

            // Normals
            Self::create_buf(
                &mut normals,
                &self.normals,
                3,
                Self::NORMALS_ATTRIB_INDEX,
                gl::FLOAT,
            );

            // Indices
            gl::GenBuffers(1, &mut indices);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.indices.size() as isize,
                self.indices.ptr(),
                gl::STATIC_DRAW,
            );

            // TODO: primitives without textures
            let pbr = material.pbr_metallic_roughness();
            let gl_texture_id = match pbr.base_color_texture() {
                Some(tex_info) => {
                    Some(self.create_texture(&tex_info.texture(), pbr.base_color_factor(), bundle))
                }
                None => None,
            };

            // Unbind buffers
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            self.vao = Some(vao);
            self.texture_index = gl_texture_id;
        }
    }

    fn create_buf<T: Copy>(id: &mut u32, buffer: &[T], stride: i32, attrib_index: u32, typ: u32) {
        unsafe {
            gl::GenBuffers(1, id);
            gl::BindBuffer(gl::ARRAY_BUFFER, *id);

            let buffer_size = buffer.len() * size_of::<T>();

            gl::BufferData(
                gl::ARRAY_BUFFER,
                buffer_size as isize,
                // TODO: check for safety - the layout of Vec3 is #[repr(C)] (struct of 3 floats),
                // so it should be correct
                buffer.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(attrib_index, stride, typ, gl::FALSE, 0, 0 as _);
            gl::EnableVertexAttribArray(attrib_index);
        }
    }

    fn create_texture(
        &mut self,
        tex: &gltf::Texture,
        base_color_factor: [f32; 4],
        bundle: &mut DataBundle,
    ) -> usize {
        let tex_index = tex.source().index();
        if bundle.gl_textures[tex_index].is_some() {
            return tex_index;
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

        bundle.gl_textures[tex_index] =
            Some(Texture::new(gl_tex_id, Vec4::from(base_color_factor)));
        tex_index
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

#[derive(Clone)]
pub struct Texture {
    pub gl_id: u32,
    pub base_color_factor: Vec4,
}

impl Texture {
    pub fn new(gl_id: u32, base_color_factor: Vec4) -> Self {
        Self {
            gl_id,
            base_color_factor,
        }
    }
}
