use eyre::{eyre, Result};
use glam::Mat4;
use gltf::mesh::util::joints;

use super::DataBundle;

pub struct Joints {
    joints: Vec<Joint>,
}

impl Joints {
    pub fn from_gltf(
        node: &gltf::Node,
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

                joints.push(Joint::new(
                    parent,
                    inverse_bind_matrices[matrix_index],
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
}

pub struct Joint {
    /// An index to the parent node (None if this joint is the root)
    parent: Option<usize>,
    /// The matrix that transforms this node to the root of the model
    inverse_bind_matrix: Mat4,
    /// Name for debug purposes
    name: String,
}

impl Joint {
    pub fn new(parent: Option<usize>, inverse_bind_matrix: Mat4, name: String) -> Self {
        Self {
            parent,
            inverse_bind_matrix,
            name,
        }
    }
}
