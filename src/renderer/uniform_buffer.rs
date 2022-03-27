use std::{mem::size_of, ptr};

use glam::Mat4;

/// Abstraction for working with UniformBuffers
pub struct UniformBuffer<T: UniformBufferElement>
where
    T: UniformBufferElement,
{
    pub id: u32,
    pub inner: T,
}

impl<T: UniformBufferElement> UniformBuffer<T>
where
    T: UniformBufferElement,
{
    pub fn new(inner: T) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::UNIFORM_BUFFER, id);

            let binding = <T as UniformBufferElement>::binding();
            gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

            inner.init_buffer();
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        Self { id, inner }
    }

    pub fn update(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);

            self.inner.update();

            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}

/// Every element that we want to store in the UniformBuffer has to implement the 'udpate' method
pub trait UniformBufferElement {
    /// Update buffer data using gl::BufferSubData
    fn update(&self);
    /// Return the binding port
    fn binding() -> u32;
    /// Allocate data for the element with gl::BufferData
    fn init_buffer(&self);
}

pub struct Transforms {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

impl UniformBufferElement for Transforms {
    fn update(&self) {
        let buf: Vec<f32> = [self.projection, self.view, self.model]
            .iter()
            .map(|mat| mat.to_cols_array())
            .flatten()
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

    fn binding() -> u32 {
        1
    }

    fn init_buffer(&self) {
        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                3 * size_of::<[f32; 16]>() as isize,
                ptr::null() as _,
                gl::STATIC_DRAW,
            );
        }
    }
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

const MAX_JOINT_TRANSFORMS: usize = 256;

pub struct JointTransforms {
    pub matrices: Vec<Mat4>,
}

impl UniformBufferElement for JointTransforms {
    fn update(&self) {
        if self.matrices.len() > MAX_JOINT_TRANSFORMS {
            todo!("Support models with more than 256 joints");
        }

        let buf: Vec<f32> = self
            .matrices
            .iter()
            .map(|mat| mat.to_cols_array())
            .flatten()
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

    fn binding() -> u32 {
        2
    }

    fn init_buffer(&self) {
        let size = MAX_JOINT_TRANSFORMS * size_of::<[f32; 16]>();

        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size as isize,
                ptr::null() as _,
                gl::STATIC_DRAW,
            );
        }
    }
}

impl JointTransforms {
    pub fn new() -> Self {
        Self {
            matrices: Vec::new(),
        }
    }
}
