use std::{
    ffi::{c_void, CStr},
    mem::size_of,
    ptr,
};

/// Abstraction for working with OpenGL Shaders.
pub mod shader;

/// Abstraction for working with OpenGL Uniform Buffers.
pub mod uniform_buffer;

// Indices of the vertex attributes
pub const POS_INDEX: u32 = 0;
pub const TEXCOORDS_INDEX: u32 = 1;
pub const NORMALS_INDEX: u32 = 2;
pub const JOINTS_INDEX: u32 = 3;
pub const WEIGHTS_INDEX: u32 = 4;

/// Create an opengl buffer with floating-point content.
///
/// 'buffer' is a reference to a slice of T.
///
/// 'components', 'attrib index' and 'typ' have the same meaning as the respective
/// arguments in glVertexAttribPointer.
pub fn create_float_buf<T: Copy>(
    buffer: &[T],
    components: i32,
    attrib_index: u32,
    typ: u32,
) -> u32 {
    let mut id: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut id as *mut _);
        gl::BindBuffer(gl::ARRAY_BUFFER, id);

        let buffer_size = buffer.len() * size_of::<T>();

        gl::BufferData(
            gl::ARRAY_BUFFER,
            buffer_size as isize,
            // The layout of Vec3 is #[repr(C)] (struct of 3 floats), so this should be correct
            buffer.as_ptr() as _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(attrib_index, components, typ, gl::FALSE, 0, 0 as _);
        gl::EnableVertexAttribArray(attrib_index);
    }

    id
}

/// Create an opengl buffer with integer content.
///
/// 'buffer' is a reference to a slice of T.
///
/// 'components', 'attrib index' and 'typ' have the same meaning as the respective
/// arguments in glVertexAttribPointer.
pub fn create_int_buf<T: Copy>(buffer: &[T], components: i32, attrib_index: u32, typ: u32) -> u32 {
    let mut id: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut id as *mut _);
        gl::BindBuffer(gl::ARRAY_BUFFER, id);

        let buffer_size = buffer.len() * size_of::<T>();

        gl::BufferData(
            gl::ARRAY_BUFFER,
            buffer_size as isize,
            // The layout of Vec3 is #[repr(C)] (struct of 3 floats), so it should be correct
            buffer.as_ptr() as _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribIPointer(attrib_index, components, typ, 0, 0 as _);
        gl::EnableVertexAttribArray(attrib_index);
    }

    id
}

pub fn init_debug() {
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_callback), ptr::null());
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0,
            ptr::null(),
            gl::TRUE,
        );
    };
}

extern "system" fn gl_debug_callback(
    _src: u32,
    _typ: u32,
    id: u32,
    severity: u32,
    _len: i32,
    msg: *const i8,
    _user_param: *mut c_void,
) {
    // Buffer creation on NVidia cards
    if id == 131185 {
        return;
    }

    match severity {
        gl::DEBUG_SEVERITY_NOTIFICATION => print!("OpenGL - notification: "),
        gl::DEBUG_SEVERITY_LOW => print!("OpenGL - low: "),
        gl::DEBUG_SEVERITY_MEDIUM => print!("OpenGL - medium: "),
        gl::DEBUG_SEVERITY_HIGH => print!("OpenGL - high: "),
        _ => unreachable!("Unknown severity in glDebugCallback: '{}'", severity),
    }

    // TODO: check if the message is guaranteed to be ASCII
    let msg = unsafe { CStr::from_ptr(msg) };
    println!("OpenGL debug message: '{}'", msg.to_string_lossy())
}
