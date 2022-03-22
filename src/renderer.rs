use std::ptr;

use glam::{Mat4, Vec3, Vec4};

use crate::{
    camera::Camera,
    gui_state::GuiState,
    model::{DataBundle, Joint, Joints, Mesh, Model, Node, PrimTexInfo},
    shader::Shader,
};

pub struct Renderer {
    shader: Shader,
    points_vao: u32,
}

impl Renderer {
    pub fn new(shader: Shader) -> Self {
        let points_vao = {
            let mut positions = 0;
            let mut texcoords = 0;
            let mut normals = 0;
            let mut vao = 0;

            unsafe {
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);

                // Positions
                create_buf(&mut positions, &[0., 0., 0.], 3, 0, gl::FLOAT);

                // Texcoords
                create_buf(&mut texcoords, &[0., 0., 0.], 2, 1, gl::FLOAT);

                // Normals
                create_buf(&mut normals, &[0., 0., 0.], 3, 2, gl::FLOAT);

                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            vao
        };

        Self { shader, points_vao }
    }

    pub fn render(
        &mut self,
        models: &mut [Model],
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

        self.shader.set_u32(0, "useTexture\0");

        let model = &mut models[gui_state.selected_model];

        let is_selected = Some(model.root.id) == gui_state.selected_node;
        let transform = model.root.transform;
        self.render_node(
            &mut model.root,
            &model.bundle,
            transform,
            is_selected,
            gui_state,
        );
    }

    fn render_node(
        &mut self,
        node: &mut Node,
        bundle: &DataBundle,
        outer_transform: Mat4,
        is_selected: bool,
        gui_state: &GuiState,
    ) {
        //println!("Node: {:?}, {}", node.name, node.transform);
        let next_level_transform = outer_transform * node.transform;

        if let Some(joints) = &mut node.joints {
            //println!("Node with a skin: {:?}", &node.name);
            for joint in &mut joints.joints {
                joint.recalc_transform();
            }

            if gui_state.debug_joints {
                self.render_joints(&mut joints.joints, next_level_transform);
            }

            self.recalc_skin_matrices(&mut joints.joints, next_level_transform);
        }

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, bundle, next_level_transform, is_selected, gui_state);
        }

        for node in &mut node.children {
            // is_selected must be true for children
            let is_selected = is_selected || Some(node.id) == gui_state.selected_node;
            self.render_node(node, bundle, next_level_transform, is_selected, gui_state);
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
            match (prim.vao, &prim.texture_info) {
                (Some(vao), prim_tex_info) => unsafe {
                    match prim_tex_info {
                        #[rustfmt::skip]
                        // TODO: draw both plain-color objects and textured ones
                        PrimTexInfo::None { base_color_factor } => {
                            self
                            .shader
                            .set_vec4(*base_color_factor, "texBaseColorFactor\0");

                            self.shader.set_u32(0, "useTexture\0");
                        }
                        PrimTexInfo::Some { texture_index } => {
                            let texture = &bundle.gl_textures[*texture_index].as_ref().unwrap();
                            self.shader
                                .set_vec4(texture.base_color_factor, "texBaseColorFactor\0");
                            self.shader.set_u32(1, "useTexture\0");

                            gl::BindTexture(gl::TEXTURE_2D, texture.gl_id);
                        }
                    };

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
        }
    }

    fn render_joints(&self, joints: &mut [Joint], outer_transform: Mat4) {
        let mut world_transforms = vec![Mat4::IDENTITY; joints.len()];

        //println!("Outer: transform: {}", outer_transform);
        //println!(
        //    "Joint 0 inverse-inverse-bind-matrix: {}",
        //    joints[0].inverse_bind_matrix.inverse()
        //);
        //println!(
        //    "outer_transform * ^ matrix = {}",
        //    outer_transform * joints[0].inverse_bind_matrix.inverse()
        //);

        //println!("Joint 0 transform-mnatrix: {}", joints[0].transform);
        //println!(
        //    "outer_transform  * ^ matrix= {}",
        //    outer_transform * joints[0].transform
        //);

        // https://github.com/KhronosGroup/glTF/issues/1665#issuecomment-529272521
        for i in 0..joints.len() {
            let transform = match joints[i].parent {
                Some(parent_index) => world_transforms[parent_index] * joints[i].transform,
                None => outer_transform * joints[i].transform,
            };

            world_transforms[i] = transform;
        }

        //println!("Joint[0] transform: {}", world_transforms[0]);

        for (i, joint) in joints.iter().enumerate() {
            let bind_transform = outer_transform * joint.inverse_bind_matrix.inverse();
            //println!("Bind transform: {}", bind_transform);

            self.shader.set_mat4(world_transforms[i], "model\0");
            self.shader.set_u32(1, "drawingPoints\0");
            self.shader
                .set_vec4(Vec4::new(0.7, 0.2, 0.2, 1.0), "texBaseColorFactor\0");

            if i == 0 {
                self.shader
                    .set_vec4(Vec4::new(0.2, 0.2, 0.7, 1.0), "texBaseColorFactor\0");
            }

            unsafe {
                gl::BindVertexArray(self.points_vao);
                gl::PointSize(10.);
                gl::DrawArrays(gl::POINTS, 0, 1);
                gl::BindVertexArray(0);
            }

            // Bind transform for debug
            self.shader.set_mat4(bind_transform, "model\0");

            self.shader
                .set_vec4(Vec4::new(0.1, 0.9, 0.3, 1.0), "texBaseColorFactor\0");

            unsafe {
                gl::BindVertexArray(self.points_vao);
                gl::PointSize(10.);
                gl::DrawArrays(gl::POINTS, 0, 1);
                gl::BindVertexArray(0);
            }

            self.shader.set_u32(0, "drawingPoints\0");
        }
    }

    pub fn recalc_skin_matrices(&self, joints: &[Joint], outer_transform: Mat4) {
        let mut world_transforms = vec![Mat4::IDENTITY; joints.len()];

        // https://github.com/KhronosGroup/glTF/issues/1665#issuecomment-529272521
        for i in 0..joints.len() {
            let transform = match joints[i].parent {
                Some(parent_index) => world_transforms[parent_index] * joints[i].transform,
                None => outer_transform * joints[i].transform,
            };

            world_transforms[i] = transform;
        }

        let mut skinning_matrices = Vec::new();
        skinning_matrices.reserve(joints.len());

        for (i, joint) in joints.iter().enumerate() {
            let parent_transform = match joint.parent {
                Some(j) => joints[j].transform,
                None => outer_transform,
            };

            let skinning_matrix = world_transforms[i] * joint.inverse_bind_matrix;

            /* println!("MAT: {}", world_transforms[i]);
            println!("MAT: {}", parent_transform); */

            if joint.name == "arm_joint_L_2" {
                println!("MAT: {}", skinning_matrix);
            }
            //skinning_matrices.push(skinning_matrix);
            skinning_matrices.push(Mat4::IDENTITY);
        }

        self.shader
            .set_mat4_arr(&skinning_matrices, "jointMatrices\0");
    }
}

fn create_buf<T: Copy>(id: &mut u32, buffer: &[T], stride: i32, attrib_index: u32, typ: u32) {
    unsafe {
        gl::GenBuffers(1, id);
        gl::BindBuffer(gl::ARRAY_BUFFER, *id);

        let buffer_size = buffer.len() * std::mem::size_of::<T>();

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
