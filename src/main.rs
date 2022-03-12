#![feature(array_chunks)]

use std::{
    ffi::{c_void, CStr},
    ptr, thread,
    time::Duration,
};

use camera::Camera;
use eyre::{Context, ContextCompat, Result};
use glam::Vec3;
use glfw::{Action, Context as GlfwContext, CursorMode, Key, OpenGlProfileHint, WindowHint};
use renderer::Renderer;
use shader::Shader;
use solid::Solid;

mod camera;
mod renderer;
mod shader;
mod solid;

fn main() -> Result<()> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).wrap_err("Failed to create GLFW window.")?;

    let width = 3840;
    let height = 2160;

    let (mut window, events) = glfw
        .create_window(
            width,
            height,
            "PGRFII projekt - Tomáš Král",
            glfw::WindowMode::Windowed,
        )
        .context("Failed to create GLFW window.")?;

    // Init OpenGL
    glfw.window_hint(WindowHint::ContextVersionMajor(4));
    glfw.window_hint(WindowHint::ContextVersionMinor(6));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlDebugContext(true));

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Enable(gl::DEPTH_TEST);
        // TODO: test culling
        //gl::Enable(gl::CULL_FACE);
        //gl::CullFace(gl::BACK);
        //gl::FrontFace(gl::CW);
        gl::PolygonMode(gl::FRONT, gl::FILL);

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

    // Winodw init
    window.set_cursor_pos((width as f64) / 2., (height as f64) / 2.);
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_framebuffer_size_polling(true);
    window.set_all_polling(true);
    window.set_raw_mouse_motion(true);
    window.make_current();

    // Shaders
    // https://sketchfab.com/3d-models/the-great-drawing-room-feb9ad17e042418c8e759b81e3b2e5d7
    let shader = Shader::from_file("shaders/vs.vert", "shaders/fs.frag")?;

    // Scene setup
    let room = Solid::from_obj_file("resources/the-great-drawing-room/model.obj")
        .wrap_err("Failed to load the object")?;

    let mut camera = Camera::new(Vec3::new(0., 0., 0.), 0.5, 0.05, width, height);
    let mut renderer = Renderer {};

    while !window.should_close() {
        glfw.poll_events();

        // pressing a key only generates one event, use get_key() instead
        if window.get_key(Key::W) == Action::Press {
            camera.move_forward(1.0);
        }
        if window.get_key(Key::S) == Action::Press {
            camera.move_backward(1.0);
        }
        if window.get_key(Key::A) == Action::Press {
            camera.strafe_left(1.0);
        }
        if window.get_key(Key::D) == Action::Press {
            camera.strafe_right(1.0);
        }

        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut camera);
        }

        renderer.render(&[&room], &shader, &mut camera, width, height);
        window.swap_buffers();

        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, camera: &mut Camera) {
    let (mouse_x, mouse_y) = window.get_cursor_pos();
    camera.adjust_look(mouse_x as f32, mouse_y as f32);

    match event {
        glfw::WindowEvent::Key(key, _, Action::Press, _) => match key {
            Key::Escape => window.set_should_close(true),
            _ => (),
        },
        _ => {}
    }
}

extern "system" fn gl_debug_callback(
    _src: u32,
    _typ: u32,
    _id: u32,
    _severity: u32,
    _len: i32,
    msg: *const i8,
    _user_param: *mut c_void,
) {
    // TODO: check if the message is guaranteed to be ASCII
    let msg = unsafe { CStr::from_ptr(msg) };
    println!("OpenGL debug message: '{}'", msg.to_string_lossy())
}
