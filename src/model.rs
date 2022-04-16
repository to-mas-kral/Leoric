use std::path::Path;

use eyre::{eyre, Result};
use glam::{Mat4, Quat, Vec3};
use gltf::scene::Transform as GTransform;

mod animation;
mod joints;
mod mesh;
mod transform;

use self::mesh::Texture;
pub use self::{
    animation::{Animation, AnimationControl, AnimationTransform, AnimationTransforms, Animations},
    joints::{Joint, Joints},
    mesh::{Mesh, PrimTexInfo, Primitive},
    transform::Transform,
};

/// Image and vertex data of the asset.
pub struct DataBundle {
    /// Vertex data
    buffers: Vec<gltf::buffer::Data>,
    /// Texture data
    images: Vec<gltf::image::Data>,
    /// To keep track if which textures were already sent to the GPU
    pub gl_textures: Vec<Option<Texture>>,
}

impl DataBundle {
    fn new(buffers: Vec<gltf::buffer::Data>, images: Vec<gltf::image::Data>) -> Self {
        Self {
            buffers,
            gl_textures: vec![Option::None; images.len()],
            images,
        }
    }
}

/// This represents a gltf model and contains necessary data for rendering.
pub struct Model {
    /// Texture data points to vectors in this bundle
    #[allow(unused)]
    pub bundle: DataBundle,
    /// An artifical root node
    pub root: Node,
    pub name: String,
    pub animations: Animations,
}

impl Model {
    pub fn from_gltf(path: &str) -> Result<Model> {
        let (gltf, buffers, images) = gltf::import(path)?;
        let name = Path::new(path)
            .file_name()
            .map(|osstr| osstr.to_string_lossy().to_string())
            .unwrap_or("N/A".to_string());

        let mut bundle = DataBundle::new(buffers, images);

        if gltf.scenes().len() != 1 {
            return Err(eyre!("GLTF file contains more than 1 scene"));
        }
        let scene = gltf.scenes().next().unwrap();

        let mut id = 1;
        let mut nodes = Vec::new();
        for node in scene.nodes() {
            let node = Node::from_gltf(&node, &mut bundle, &mut id, &scene)?;
            id += 1;
            nodes.push(node);
        }

        let animations = Animation::from_gltf(&gltf, &bundle)?;

        let root = Node {
            index: usize::MAX,
            name: "Root".to_string(),
            children: nodes,
            mesh: None,
            transform: Mat4::IDENTITY,
            joints: None,
        };

        Ok(Model {
            bundle,
            root,
            name,
            animations,
        })
    }
}

/// A Node represents a subset of a gltf scene
/// Nodes form a tree hierarchy
pub struct Node {
    /// The same index as in the gltf file
    pub index: usize,
    pub name: String,

    pub children: Vec<Node>,
    pub mesh: Option<Mesh>,

    pub transform: Mat4,

    pub joints: Option<Joints>,
}

impl Node {
    fn from_gltf(
        node: &gltf::Node,
        bundle: &mut DataBundle,
        id: &mut u32,
        scene: &gltf::Scene,
    ) -> Result<Self> {
        let mut children = Vec::new();

        let name = node.name().unwrap_or(&format!("Node-{id}")).to_string();

        for child_node in node.children() {
            *id += 1;
            let node = Node::from_gltf(&child_node, bundle, id, scene)?;
            children.push(node);
        }

        let mesh = match node.mesh() {
            Some(m) => Some(Mesh::from_gltf(&m, bundle)?),
            None => None,
        };

        let transform = match node.transform() {
            GTransform::Matrix { matrix } => Mat4::from_cols_array_2d(&matrix),
            GTransform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                Mat4::from_translation(Vec3::from(translation))
                    * Mat4::from_quat(Quat::from_xyzw(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    ))
                    * Mat4::from_scale(Vec3::from(scale))
            }
        };

        let joints = if let Some(skin) = node.skin() {
            Some(Joints::from_gltf(bundle, &skin, scene)?)
        } else {
            None
        };

        Ok(Self {
            index: node.index(),
            children,
            mesh,
            transform,
            name,
            joints,
        })
    }
}
