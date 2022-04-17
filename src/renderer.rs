use std::{ptr, time::Instant};

use eyre::Result;
use glam::{Mat4, Vec4};

use crate::{
    camera::Camera,
    gui::Gui,
    model::{
        AnimationControl, AnimationTransform, DataBundle, Joint, Mesh, Model, Node, Primitive,
        PrimitiveTexture,
    },
    ogl::{shader::Shader, uniform_buffer::UniformBuffer},
    window::MyWindow,
};

mod joint_transforms;
mod material;
mod settings;
mod transforms;

use self::{
    joint_transforms::JointTransforms, material::Material, settings::Settings,
    transforms::Transforms,
};

pub struct Renderer {
    texture_shader: Shader,
    color_shader: Shader,

    transforms: UniformBuffer<Transforms>,
    joint_transforms: UniformBuffer<JointTransforms>,
    settings: UniformBuffer<Settings>,
    material: UniformBuffer<Material>,

    points_vao: u32,
    node_animation_transforms: Vec<NodeAnimationTransform>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let texture_shader =
            Shader::from_file("shaders/vs_combined.vert", "shaders/fs_texture.frag")?;
        let color_shader = Shader::from_file("shaders/vs_combined.vert", "shaders/fs_color.frag")?;

        // TODO: refactor debug joints drawing
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

        Ok(Self {
            texture_shader,
            color_shader,
            transforms: UniformBuffer::new(Transforms::new_indentity()),
            joint_transforms: UniformBuffer::new(JointTransforms::new()),
            settings: UniformBuffer::new(Settings::new()),
            material: UniformBuffer::new(Material::new()),
            points_vao,
            node_animation_transforms: Vec::new(),
        })
    }

    pub fn render(
        &mut self,
        models: &mut [Model],
        camera: &mut Camera,
        window: &MyWindow,
        gui_state: &Gui,
    ) {
        unsafe {
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.node_animation_transforms.clear();

        // Transformations
        // TODO: možná glu perspective
        let persp = Mat4::perspective_rh(
            f32::to_radians(60.),
            window.width as f32 / window.height as f32,
            0.1,
            3000.,
        );

        self.transforms.inner.projection = persp;
        self.transforms.inner.view = camera.get_view_mat();
        self.transforms.update();

        /* self.texture_shader
            .set_vec3(Vec3::new(-1., 2., 2.), "lightPos\0");
        self.texture_shader.set_vec3(camera.get_pos(), "viewPos\0"); */

        let model = &mut models[gui_state.selected_model];

        self.apply_animation(model);

        let transform = model.root.transform;
        self.render_node(&mut model.root, &model.bundle, transform, gui_state);
    }

    fn render_node(
        &mut self,
        node: &mut Node,
        bundle: &DataBundle,
        outer_transform: Mat4,
        gui_state: &Gui,
    ) {
        let next_level_transform = outer_transform * node.transform;

        if let Some(joints) = &mut node.joints {
            self.recalc_skin_matrices(&mut joints.joints, next_level_transform, gui_state);
        }

        if let Some(mesh) = &node.mesh {
            let do_skinning = node.joints.is_some();
            self.settings.inner.do_skinning = do_skinning;
            self.settings.update();

            self.render_mesh(mesh, next_level_transform);
        }

        for node in &mut node.children {
            self.render_node(node, bundle, next_level_transform, gui_state);
        }
    }

    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4) {
        self.transforms.inner.model = node_transform;
        self.transforms.update();

        let draw_mesh = |vao: u32, prim: &Primitive| unsafe {
            gl::BindVertexArray(vao);

            gl::DrawElements(
                gl::TRIANGLES,
                prim.indices.len() as i32,
                prim.indices.gl_type(),
                ptr::null(),
            );

            gl::BindVertexArray(0);
        };

        for prim in &mesh.primitives {
            match prim.texture_info {
                PrimitiveTexture::None { base_color_factor } => {
                    self.material.inner.base_color_factor = base_color_factor;
                    self.material.update();

                    self.color_shader.render(|| {
                        draw_mesh(prim.vao, prim);
                    });
                }
                PrimitiveTexture::Some {
                    gl_id,
                    base_color_factor,
                } => {
                    self.material.inner.base_color_factor = base_color_factor;
                    self.material.update();

                    unsafe {
                        gl::BindTexture(gl::TEXTURE_2D, gl_id);
                    }

                    self.texture_shader.render(|| {
                        draw_mesh(prim.vao, prim);
                    });
                }
            };
        }
    }

    pub fn recalc_skin_matrices(
        &mut self,
        joints: &mut [Joint],
        outer_transform: Mat4,
        gui_state: &Gui,
    ) {
        self.apply_joint_transforms(joints);

        let mut world_transforms = vec![Mat4::IDENTITY; joints.len()];

        for i in 0..joints.len() {
            let transform = match joints[i].parent {
                Some(parent_index) => world_transforms[parent_index] * joints[i].transform.matrix(),
                None => outer_transform * joints[i].transform.matrix(),
            };

            world_transforms[i] = transform;
        }

        if gui_state.debug_joints {
            self.debug_joints(&world_transforms);
        }

        let joint_matrices = &mut self.joint_transforms.inner.matrices;
        joint_matrices.clear();

        for (i, joint) in joints.iter().enumerate() {
            let mat = world_transforms[i] * joint.inverse_bind_matrix;
            joint_matrices.push(mat);
        }

        self.joint_transforms.update();
    }

    fn debug_joints(&mut self, world_transforms: &[Mat4]) {
        for trans in world_transforms {
            self.transforms.inner.model = *trans;
            self.transforms.update();

            self.material.inner.base_color_factor = Vec4::new(0.85, 0.08, 0.7, 1.0);
            self.material.update();

            self.settings.inner.do_skinning = false;
            self.settings.update();

            self.color_shader.render(|| unsafe {
                gl::BindVertexArray(self.points_vao);
                gl::PointSize(7.);
                gl::DrawArrays(gl::POINTS, 0, 1);
                gl::BindVertexArray(0);
            });
        }
    }

    fn apply_animation(&mut self, model: &mut Model) {
        let active_animation = match model.animations.animation_control {
            AnimationControl::Loop {
                active_animation,
                start_time,
            } => {
                let anim = &mut model.animations.animations[active_animation];

                let mut since_start = Instant::now().duration_since(start_time).as_secs_f32();
                if since_start > anim.end_time {
                    since_start %= anim.end_time;
                }

                anim.current_time = since_start;
                active_animation
            }
            AnimationControl::Controllable { active_animation } => active_animation,
            AnimationControl::Static => return,
        };

        self.node_animation_transforms.clear();
        let anim = &model.animations.animations[active_animation];
        let current_time = anim.current_time;

        for channel in &anim.channels {
            let keyframe_times = &channel.keyframe_times;

            'inner: for i in 0..keyframe_times.len() {
                let start_time = keyframe_times[i];

                if (i == keyframe_times.len() - 1) || (i == 0 && current_time < start_time) {
                    let transform = channel.get_fixed_transform(i);
                    self.node_animation_transforms
                        .push(NodeAnimationTransform::new(channel.node, transform));
                    break 'inner;
                }

                let end_time = keyframe_times[i + 1];

                if start_time <= current_time && end_time > current_time {
                    let coeff = (current_time - start_time) / (end_time - start_time);

                    let transform = channel.interpolate_transforms(i, coeff);

                    self.node_animation_transforms
                        .push(NodeAnimationTransform::new(channel.node, transform));
                    break;
                }
            }
        }

        // TODO: animate nodes aswell
        //self.apply_node_transforms(&mut model.root);
    }

    /* fn apply_node_transforms(&self, node: &mut Node) {
        // Apply animation transformation
        if let Some(node_animation_transform) = self
            .node_animation_transforms
            .iter()
            .find(|nat| nat.node == node.index)
        {
            match node_animation_transform.transform {
                AnimationTransform::Translation(trans) => node.transform.translation = trans,
                AnimationTransform::Rotation(rot) => node.transform.rotation = rot,
                AnimationTransform::Scale(scale) => node.transform.scale = scale,
            }
        }

        for child in &mut node.children {
            self.apply_node_transforms(child);
        }
    } */

    fn apply_joint_transforms(&self, joints: &mut [Joint]) {
        for joint in joints {
            for nat in &self.node_animation_transforms {
                if joint.node_index == nat.node {
                    match nat.transform {
                        AnimationTransform::Translation(trans) => {
                            joint.transform.translation = trans;
                        }
                        AnimationTransform::Rotation(rot) => {
                            joint.transform.rotation = rot;
                        }
                        AnimationTransform::Scale(scale) => joint.transform.scale = scale,
                    }
                }
            }
        }
    }
}

struct NodeAnimationTransform {
    /// Index of the node
    node: usize,
    /// Transform that should overwrite the node's current transform
    transform: AnimationTransform,
}

impl NodeAnimationTransform {
    fn new(node: usize, transform: AnimationTransform) -> Self {
        Self { node, transform }
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
