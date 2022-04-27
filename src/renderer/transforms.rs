use std::{mem::size_of, ptr};

use glam::Mat4;

use crate::ogl::uniform_buffer::UniformBufferElement;

/// Uniform buffer element that stores the transformation matrices
pub struct Transforms {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

impl Transforms {
    pub fn new_indentity() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
        }
    }
}

impl UniformBufferElement for Transforms {
    fn update(&self) {
        let buf: Vec<f32> = [self.projection, self.view, self.model]
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
        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                3 * size_of::<[f32; 16]>() as isize,
                ptr::null() as _,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    const BINDING: u32 = 1;
}
