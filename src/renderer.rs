use std::ptr;

use glam::{Mat4, Vec3};

use crate::{camera::Camera, shader::Shader, solid::Solid};

pub struct Renderer {}

impl Renderer {
    pub fn render(
        &mut self,
        solids: &[&Solid],
        shader: &Shader,
        camera: &mut Camera,
        width: u32,
        height: u32,
    ) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader.id);
        }

        // Transformations
        // TODO: možná glu perspective
        let persp = Mat4::perspective_rh(
            f32::to_radians(60.),
            width as f32 / height as f32,
            0.1,
            300.,
        );
        shader.set_mat4(persp, "projection\0");
        shader.set_mat4(camera.get_view_mat(), "view\0");

        shader.set_vec3(Vec3::new(-1., 1.5, 2.), "lightPos\0");
        shader.set_vec3(camera.get_pos(), "viewPos\0");

        for solid in solids {
            shader.set_mat4(solid.transform, "model\0");

            for mesh in &solid.meshes {
                match (mesh.vao, mesh.texture) {
                    (Some(vao), Some(texture)) => unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, texture);
                        gl::BindVertexArray(vao);

                        shader.set_vec3(mesh.material.ambient_k, "material.ambient\0");
                        shader.set_vec3(mesh.material.diffuse_k, "material.diffuse\0");
                        shader.set_vec3(mesh.material.specular_k, "material.specular\0");
                        shader.set_f32(mesh.material.shininess, "material.shininess\0");

                        gl::DrawElements(
                            gl::TRIANGLES,
                            (mesh.vertex_count * 3) as i32,
                            gl::UNSIGNED_INT,
                            ptr::null(),
                        );
                    },
                    _ => (),
                }
            }
        }
    }
}
