use std::ptr;

use glam::{Mat4, Vec3};

use crate::{
    camera::Camera,
    gui_state::GuiState,
    model::{DataBundle, Mesh, Model, Node},
    shader::Shader,
};

pub struct Renderer {
    shader: Shader,
}

impl Renderer {
    pub fn new(shader: Shader) -> Self {
        Self { shader }
    }

    pub fn render(
        &mut self,
        models: &[Model],
        camera: &mut Camera,
        width: u32,
        height: u32,
        gui_state: &GuiState,
    ) {
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
            let is_selected = Some(model.root.id) == gui_state.selected_node;
            self.render_node(
                &model.root,
                &model.bundle,
                Mat4::IDENTITY,
                is_selected,
                gui_state,
            );
        }
    }

    fn render_node(
        &mut self,
        node: &Node,
        bundle: &DataBundle,
        mut node_transform: Mat4,
        is_selected: bool,
        gui_state: &GuiState,
    ) {
        node_transform *= node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, bundle, node_transform, is_selected, gui_state);
        }

        for node in &node.children {
            // is_selected msut be true for children
            let is_selected = is_selected || Some(node.id) == gui_state.selected_node;
            self.render_node(node, bundle, node_transform, is_selected, gui_state);
        }
    }

    fn render_mesh(
        &mut self,
        mesh: &Mesh,
        bundle: &DataBundle,
        node_transform: Mat4,
        is_selected: bool,
        gui_state: &GuiState,
    ) {
        self.shader.set_mat4(node_transform, "model\0");

        if is_selected || gui_state.selected_node.is_none() {
            self.shader.set_f32(1.0, "globalAlpha\0")
        } else {
            self.shader.set_f32(0.075, "globalAlpha\0")
        }

        for prim in &mesh.primitives {
            match (prim.vao, prim.texture_index) {
                (Some(vao), Some(texture_index)) => unsafe {
                    let texture = &bundle.gl_textures[texture_index].as_ref().unwrap();
                    self.shader
                        .set_vec4(texture.base_color_factor, "texBaseColorFactor\0");

                    gl::BindTexture(gl::TEXTURE_2D, texture.gl_id);
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
