use std::{mem::size_of, ptr};

use glam::Vec3;

use crate::ogl::uniform_buffer::UniformBufferElement;

pub struct Lighting {
    pub light_pos: Vec3,
}

impl Lighting {
    pub fn new(light_pos: Vec3) -> Self {
        Self { light_pos }
    }
}

impl UniformBufferElement for Lighting {
    fn update(&self) {
        // GLSL vec3 has an alignment of 16 bytes (4 floats)
        let size = 4 * size_of::<f32>();
        let buf = self.light_pos.extend(0.).to_array();

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

    const BINDING: u32 = 5;
}
