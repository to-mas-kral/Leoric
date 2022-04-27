use glam::{Mat4, Quat, Vec3};
use gltf::scene::Transform as GTransform;

/// Describes the transformation of a Node or a Joint
pub struct Transform {
    /// Local translation relative to the parent joint
    pub translation: Vec3,
    /// Local rotation relative to the parent joint
    pub rotation: Quat,
    /// Local scale relative to the parent joint
    pub scale: Vec3,
}

impl Transform {
    /// Creates the transform from the gltf::Node struct
    pub fn from_gltf(node: &gltf::Node) -> Self {
        let (translation, rotation, scale) = match node.transform() {
            GTransform::Matrix { matrix: mat } => {
                // https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#transformations
                // "When matrix is defined, it MUST be decomposable to TRS properties."
                Mat4::from_cols_array_2d(&mat).to_scale_rotation_translation()
            }
            GTransform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                let translation = Vec3::from(translation);
                let scale = Vec3::from(scale);
                let rotation = Quat::from_array(rotation);

                (translation, rotation, scale)
            }
        };

        // https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html#transformations
        // When the scale is zero on all three axes (by node transform or by animated scale),
        // implementations are free to optimize away rendering of the node’s mesh, and all of
        // the node’s children’s meshes. This provides a mechanism to animate visibility.
        // Skinned meshes must not use this optimization unless all of the joints in the
        // skin are scaled to zero simultaneously.
        // ..... why is this a thing...
        // FIXME: scale(0, 0, 0)...

        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Constructs the transform combined from translation, rotation and scale
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}
