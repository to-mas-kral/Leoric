use std::mem::size_of;

use eyre::Result;
use image::DynamicImage;

const POS_ATTRIB_INDEX: u32 = 0;
const TEXCOORDS_ATTRIB_INDEX: u32 = 1;

pub struct Solid {
    pub meshes: Vec<Mesh>,
}

impl Solid {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Solid { meshes }
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

            let material = Material::new(diffuse_texture);

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
            let mut vao = 0;

            // TODO: normals

            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);

                gl::GenBuffers(1, &mut positions);
                gl::BindBuffer(gl::ARRAY_BUFFER, positions);

                let positions_size = mesh.positions.len() * size_of::<f32>();
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    positions_size as isize,
                    mesh.positions.as_ptr() as _,
                    gl::STATIC_DRAW,
                );

                gl::GenBuffers(1, &mut texcoords);
                gl::BindBuffer(gl::ARRAY_BUFFER, texcoords);

                let texcoords_size = mesh.texcoords.len() * size_of::<f32>();
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    texcoords_size as isize,
                    mesh.texcoords.as_ptr() as _,
                    gl::STATIC_DRAW,
                );

                gl::BindBuffer(gl::ARRAY_BUFFER, positions);
                gl::VertexAttribPointer(POS_ATTRIB_INDEX, 3, gl::FLOAT, gl::FALSE, 0, 0 as _);
                gl::EnableVertexAttribArray(POS_ATTRIB_INDEX);

                gl::BindBuffer(gl::ARRAY_BUFFER, texcoords);
                gl::VertexAttribPointer(TEXCOORDS_ATTRIB_INDEX, 2, gl::FLOAT, gl::FALSE, 0, 0 as _);
                gl::EnableVertexAttribArray(TEXCOORDS_ATTRIB_INDEX);

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

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    mesh.material.diffuse_texture.width as i32,
                    mesh.material.diffuse_texture.height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    mesh.material.diffuse_texture.pixels.as_slice().as_ptr() as _,
                );
                gl::GenerateMipmap(gl::TEXTURE_2D);

                mesh.texture = Some(texture);
                mesh.vao = Some(vao);
            }
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
    // ambient
    // diffuse
    // specular
    // shininess
    pub diffuse_texture: Texture,
}

impl Material {
    pub fn new(diffuse_texture: DynamicImage) -> Self {
        let diffuse_texture = diffuse_texture.flipv().into_rgba8();
        let width = diffuse_texture.width();
        let height = diffuse_texture.height();

        let flat: Vec<u8> = diffuse_texture.into_raw();
        Self {
            diffuse_texture: Texture::new(flat, width, height),
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
