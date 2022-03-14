use std::mem::size_of;

use eyre::Result;
use glam::{EulerRot, Mat4, Vec3};
use image::DynamicImage;

const POS_ATTRIB_INDEX: u32 = 0;
const TEXCOORDS_ATTRIB_INDEX: u32 = 1;
const NORMALS_ATTRIB_INDEX: u32 = 2;

pub struct Solid {
    pub meshes: Vec<Mesh>,
    pub pos: Vec3,
    pub scale: Vec3,
    pub rot: Vec3,
}

impl Solid {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Solid {
            meshes,
            pos: Vec3::splat(0.),
            scale: Vec3::splat(1.),
            rot: Vec3::splat(0.),
        }
    }

    pub fn get_transform(&self) -> Mat4 {
        let r = self.rot;
        Mat4::from_translation(self.pos)
            * Mat4::from_euler(EulerRot::XYZ, r.x, r.y, r.z)
            * Mat4::from_scale(self.scale)
    }

    pub fn from_obj_file(obj_path: &str) -> Result<Self> {
        let mut load_options = tobj::LoadOptions::default();
        load_options.triangulate = true;
        load_options.single_index = true;

        let (models, materials) = tobj::load_obj(&obj_path, &load_options)?;
        let materials = materials?;

        let mut meshes = Vec::new();
        for model in models {
            let mat_index = model.mesh.material_id.unwrap();
            let material = &materials[mat_index];
            let diffuse_texture_path = format!(
                "{}/textures/{}",
                obj_path.rsplit_once("/").unwrap().0,
                material.diffuse_texture
            );

            let diffuse_texture = if material.diffuse_texture == "" {
                image::DynamicImage::new_rgb8(1, 1)
            } else {
                image::open(diffuse_texture_path)?
            };

            let material = Material::new(
                diffuse_texture,
                Vec3::from(material.diffuse),
                Vec3::from(material.ambient),
                Vec3::from(material.specular),
                material.shininess,
                material.illumination_model,
            );

            let mesh = Mesh::new(
                model.mesh.positions,
                model.mesh.indices,
                model.mesh.normals,
                model.mesh.texcoords,
                material,
            );

            meshes.push(mesh);
        }

        let mut solid = Solid::new(meshes);
        solid.create_bufs();

        Ok(solid)
    }

    pub fn create_bufs(&mut self) {
        for mesh in &mut self.meshes {
            let mut positions = 0;
            let mut texcoords = 0;
            let mut indices = 0;
            let mut normals = 0;
            let mut vao = 0;

            // TODO: normals

            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);

                // Positions
                Self::create_buf(
                    &mut positions,
                    &mesh.positions,
                    3,
                    POS_ATTRIB_INDEX,
                    gl::FLOAT,
                );

                // Texcoords
                Self::create_buf(
                    &mut texcoords,
                    &mesh.texcoords,
                    2,
                    TEXCOORDS_ATTRIB_INDEX,
                    gl::FLOAT,
                );

                // Normals
                Self::create_buf(
                    &mut normals,
                    &mesh.normals,
                    3,
                    NORMALS_ATTRIB_INDEX,
                    gl::FLOAT,
                );

                // Indices
                gl::GenBuffers(1, &mut indices);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);

                let indices_size = mesh.indices.len() * size_of::<u32>();
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    indices_size as isize,
                    mesh.indices.as_ptr() as _,
                    gl::STATIC_DRAW,
                );

                // Texture
                let texture = Self::create_tex(mesh);

                mesh.texture = Some(texture);
                mesh.vao = Some(vao);
            }
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
                buffer.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(attrib_index, stride, typ, gl::FALSE, 0, 0 as _);
            gl::EnableVertexAttribArray(attrib_index);
        }
    }

    fn create_tex(mesh: &mut Mesh) -> u32 {
        unsafe {
            let mut texture = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::MIRRORED_REPEAT as i32,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::MIRRORED_REPEAT as i32,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            let width = mesh.material.diffuse_texture.width as i32;
            let height = mesh.material.diffuse_texture.height as i32;
            let pixels = mesh.material.diffuse_texture.pixels.as_slice();

            // TODO: account for different stride
            assert!(width * height * 4 == pixels.len() as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            texture
        }
    }
}

pub struct Mesh {
    pub vao: Option<u32>,
    pub vertex_count: u32,
    pub texture: Option<u32>,

    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material: Material,
}

impl Mesh {
    pub fn new(
        positions: Vec<f32>,
        indices: Vec<u32>,
        normals: Vec<f32>,
        texcoords: Vec<f32>,
        material: Material,
    ) -> Self {
        /* let positions: Vec<Vec3> = positions.array_chunks().map(|c| (*c).into()).collect();
        let normals: Vec<Vec3> = normals.array_chunks().map(|c| (*c).into()).collect();
        let texcoords: Vec<Vec2> = texcoords.array_chunks().map(|c| (*c).into()).collect();

        let indices: Vec<[u32; 3]> = indices.array_chunks().map(|c| *c).collect(); */

        Self {
            vao: None,
            texture: None,
            vertex_count: (indices.len() / 3) as u32,
            material,
            positions,
            normals,
            texcoords,
            indices,
        }
    }
}

pub struct Material {
    pub ambient_k: Vec3,
    pub diffuse_k: Vec3,
    pub specular_k: Vec3,
    pub shininess: f32,
    pub illumination_model: Option<u8>,

    pub diffuse_texture: Texture,
}

impl Material {
    pub fn new(
        diffuse_texture: DynamicImage,
        diffuse_k: Vec3,
        ambient_k: Vec3,
        specular_k: Vec3,
        shininess: f32,
        illumination_model: Option<u8>,
    ) -> Self {
        let diffuse_texture = diffuse_texture.flipv().into_rgba8();
        let width = diffuse_texture.width();
        let height = diffuse_texture.height();

        let flat: Vec<u8> = diffuse_texture.into_raw();
        Self {
            diffuse_texture: Texture::new(flat, width, height),
            diffuse_k,
            ambient_k,
            specular_k,
            shininess,
            illumination_model,
        }
    }
}

pub struct Texture {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }
}
