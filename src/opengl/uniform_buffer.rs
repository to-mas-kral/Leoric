/// Abstraction for working with UniformBuffers.
/// The syntax might be a bit weird.
/// UniformBuffer is generic over T, and T must implement the UniformBufferElement trait.
pub struct UniformBuffer<T: UniformBufferElement> {
    pub id: u32,
    pub inner: T,
}

impl<T: UniformBufferElement> UniformBuffer<T>
where
    T: UniformBufferElement,
{
    /// Generate a new UniformBuffer and allocate memory for it
    pub fn new(inner: T) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::UNIFORM_BUFFER, id);

            let binding = <T as UniformBufferElement>::BINDING;
            gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

            inner.init_buffer();
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        Self { id, inner }
    }

    /// Update the UniformBuffer with the current state of `inner`
    pub fn update(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);

            self.inner.update();

            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}

pub trait UniformBufferElement {
    /// The binding port
    const BINDING: u32;
    /// Update buffer data using gl::BufferSubData
    fn update(&self);
    /// Allocate data for the element with gl::BufferData
    fn init_buffer(&self);
}
