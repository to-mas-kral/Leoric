use std::ptr;

use glam::{Mat4, Vec3};

use crate::{
    camera::Camera,
    model::{Mesh, Model, Node},
    shader::Shader,
};

pub struct Renderer {
    shader: Shader,
}

impl Renderer {
    pub fn new(shader: Shader) -> Self {
        Self { shader }
    }

    pub fn render(&mut self, models: &[&Model], camera: &mut Camera, width: u32, height: u32) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(self.shader.id);
        }

        // Transformations
        // TODO: možná glu perspective
        let persp = Mat4::perspective_rh(
            f32::to_radians(60.),
            width as f32 / height as f32,
            0.1,
            300.,
        );
        self.shader.set_mat4(persp, "projection\0");
        self.shader.set_mat4(camera.get_view_mat(), "view\0");

        self.shader.set_vec3(Vec3::new(-1., 2., 2.), "lightPos\0");
        self.shader.set_vec3(camera.get_pos(), "viewPos\0");

        for model in models {
            self.render_node(&model.root, Mat4::IDENTITY);
        }
    }

    fn render_node(&mut self, node: &Node, mut node_transform: Mat4) {
        node_transform *= node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, node_transform);
        }

        for model in &node.children {
            self.render_node(model, node_transform);
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4) {
        self.shader.set_mat4(node_transform, "model\0");

        for prim in &mesh.primitives {
            match (prim.vao, prim.gl_texture_id) {
                (Some(vao), Some(texture_id)) => unsafe {
                    gl::BindTexture(gl::TEXTURE_2D, texture_id);
                    gl::BindVertexArray(vao);

                    gl::DrawElements(
                        gl::TRIANGLES,
                        prim.indices.len() as i32,
                        prim.indices.gl_type(),
                        ptr::null(),
                    );

                    gl::BindVertexArray(0);
                },
                _ => (),
            }

            /* match (mesh.vao, mesh.texture) {
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
            } */
        }
    }
}
