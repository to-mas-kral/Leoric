use std::{mem::size_of, ptr};

use glam::Mat4;

use crate::ogl::uniform_buffer::UniformBufferElement;

const MAX_JOINT_TRANSFORMS: usize = 256;

/// Uniform buffer element that stores the vertex joint transforms
pub struct JointTransforms {
    pub matrices: Vec<Mat4>,
}

impl JointTransforms {
    pub fn new() -> Self {
        Self {
            matrices: Vec::new(),
        }
    }
}

impl UniformBufferElement for JointTransforms {
    fn update(&self) {
        if self.matrices.len() > MAX_JOINT_TRANSFORMS {
            todo!("Support models with more than 256 joints");
        }

        let buf: Vec<f32> = self
            .matrices
            .iter()
            .flat_map(|mat| mat.to_cols_array())
            .collect();

        unsafe {
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                (buf.len() * size_of::<f32>()) as isize,
                buf.as_ptr() as _,
            );
        }
    }

    fn init_buffer(&self) {
        let size = MAX_JOINT_TRANSFORMS * size_of::<[f32; 16]>();

        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size as isize,
                ptr::null() as _,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    const BINDING: u32 = 2;
}
