use eyre::Result;
use glam::{Mat4, Quat, Vec3};
use gltf::scene::Transform;

use super::DataBundle;

pub struct Joints {
    pub joints: Vec<Joint>,
}

impl Joints {
    pub fn from_gltf(
        bundle: &mut DataBundle,
        skin: &gltf::Skin,
        scene: &gltf::Scene,
    ) -> Result<Self> {
        let joint_indices: Vec<usize> = skin.joints().map(|j| j.index()).collect();

        let mut joints = Vec::new();

        let reader = skin.reader(|buf| Some(&bundle.buffers[buf.index()]));
        let inverse_bind_matrices = match reader.read_inverse_bind_matrices() {
            Some(matrices) => matrices.map(|m| Mat4::from_cols_array_2d(&m)).collect(),
            None => vec![Mat4::IDENTITY; joints.len()],
        };

        // TODO: not great performance-wise
        let children: Vec<gltf::Node> = scene.nodes().collect();

        Self::build_hierarchy(
            &children,
            &joint_indices,
            None,
            &mut joints,
            &inverse_bind_matrices,
        );

        Ok(Self { joints })
    }

    /// Traverse the scene and arrange the joint nodes into a correct hierarchy
    /// https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#joint-hierarchy
    /// "A node object does not specify whether it is a joint.
    /// Client implementations may need to traverse the skins array first, marking each joint node."
    // FIXME: the transformations will not be correct if there are "gap nodes" between the joints
    fn build_hierarchy(
        nodes: &[gltf::Node],
        joint_indices: &Vec<usize>,
        parent: Option<usize>,
        joints: &mut Vec<Joint>,
        inverse_bind_matrices: &[Mat4],
    ) {
        for node in nodes {
            let children: Vec<gltf::Node> = node.children().collect();
            let index = node.index();

            if joint_indices.contains(&index) {
                // Found a joint node, add it to the hierarchy
                let joints_index = joints.len();

                let matrix_index = joint_indices.iter().position(|i| *i == index).unwrap();
                let name = node.name().unwrap_or("N/A").to_string();

                let (translation, rotation, scale) = Self::get_joint_transform(node);

                joints.push(Joint::new(
                    parent,
                    inverse_bind_matrices[matrix_index],
                    translation,
                    rotation,
                    scale,
                    name,
                ));

                Self::build_hierarchy(
                    &children,
                    joint_indices,
                    Some(joints_index),
                    joints,
                    inverse_bind_matrices,
                );

                if parent.is_none() {
                    // This is the root node, break
                    return;
                }
            } else {
                if !joints.is_empty() {
                    // This is just a bad part of the spec...
                    //unimplemented!("ERROR: A non-joint node in the joint node hierarchy");
                }

                // Didn't find a joint node, recurse further
                Self::build_hierarchy(
                    &children,
                    joint_indices,
                    parent,
                    joints,
                    inverse_bind_matrices,
                );
            }
        }
    }

    fn get_joint_transform(node: &gltf::Node) -> (Vec3, Quat, Vec3) {
        let (translation, rotation, scale) = match node.transform() {
            Transform::Matrix { matrix: mat } => {
                // https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#transformations
                // "When matrix is defined, it MUST be decomposable to TRS properties."
                Mat4::from_cols_array_2d(&mat).to_scale_rotation_translation()
            }
            Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                let translation = Vec3::from(translation);
                // TODO: should normalize quaternion or not ?
                //let rotation = Quat::from_array(rotation);
                let rotation = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);
                let scale = Vec3::from(scale);

                (translation, rotation, scale)
            }
        };
        (translation, rotation, scale)
    }
}

pub struct Joint {
    /// An index to the parent node (None if this joint is the root)
    pub parent: Option<usize>,
    /// The matrix that transforms this node to the origin
    pub inverse_bind_matrix: Mat4,
    /// Local translation relative to the parent joint
    pub translation: Vec3,
    /// Local rotation relative to the parent joint
    pub rotation: Quat,
    /// Local scale relative to the parent joint
    pub scale: Vec3,
    /// All local transformation combined
    pub transform: Mat4,
    /// Name for debug purposes
    pub name: String,
}

impl Joint {
    pub fn new(
        parent: Option<usize>,
        inverse_bind_matrix: Mat4,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
        name: String,
    ) -> Self {
        let mut s = Self {
            parent,
            inverse_bind_matrix,
            translation,
            rotation,
            scale,
            transform: Mat4::IDENTITY,
            name,
        };

        s.recalc_transform();
        s
    }

    pub fn recalc_transform(&mut self) {
        self.transform = Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale);
    }
}
