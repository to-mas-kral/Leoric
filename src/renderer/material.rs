use std::{mem::size_of, ptr};

use glam::Vec4;

use crate::ogl::uniform_buffer::UniformBufferElement;

pub struct Material {
    pub base_color_factor: Vec4,
}

impl Material {
    pub fn new() -> Self {
        Self {
            base_color_factor: Vec4::splat(1.),
        }
    }
}

impl UniformBufferElement for Material {
    fn update(&self) {
        let size = 4 * size_of::<f32>();
        let buf = self.base_color_factor.to_array();

        unsafe {
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, size as isize, buf.as_ptr() as _);
        }
    }

    fn init_buffer(&self) {
        let size = 4 * size_of::<f32>();

        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size as isize,
                ptr::null() as _,
                gl::STATIC_DRAW,
            );
        }
    }

    const BINDING: u32 = 4;
}
