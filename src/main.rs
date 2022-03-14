use std::{
    ffi::{c_void, CStr},
    ptr, thread,
    time::{Duration, Instant},
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

    let width = 1 * 1920u32;
    let height = 1 * 1080u32;

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
    glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));

    gl::load_with(|s| window.get_proc_address(s) as *const _);
    unsafe {
        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Enable(gl::DEPTH_TEST);
        //TODO: test culling
        //gl::Enable(gl::CULL_FACE);
        //gl::CullFace(gl::BACK);
        //gl::FrontFace(gl::CCW);
        gl::PolygonMode(gl::FRONT, gl::FILL);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

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
    window.set_framebuffer_size_polling(true);
    window.set_all_polling(true);
    window.set_raw_mouse_motion(true);
    window.make_current();

    // Shaders
    let shader = Shader::from_file("shaders/vs.vert", "shaders/fs.frag")?;

    // Scene setup
    // This work is based on "Lancia Fulvia rallye"
    // (https://sketchfab.com/3d-models/lancia-fulvia-rallye-5f02ef9e0daf481aba8c8b51216c0a6b)
    // by Floppy (https://sketchfab.com/fastolfe) licensed under CC-BY-NC-4.0
    // (http://creativecommons.org/licenses/by-nc/4.0/)
    let mut car = Solid::from_obj_file("resources/lancia_fulvia_rallye/Fulvia.obj")
        .wrap_err("Failed to load the object")?;

    // This work is based on "Street_Light"
    // (https://sketchfab.com/3d-models/street-light-16fc20d9d6564adb84ea27e35778da06)
    // by dodotcreatives (https://sketchfab.com/dodotcreatives)
    // licensed under CC-BY-4.0
    // (http://creativecommons.org/licenses/by/4.0/)
    let mut street_light = Solid::from_obj_file("resources/street_light/StreetLight.obj")
        .wrap_err("Failed to load the object")?;

    street_light.pos = Vec3::new(-1., 0., 2.);
    street_light.scale = Vec3::splat(0.3);

    let mut camera = Camera::new(Vec3::new(0., 0., 0.), 0.3, 0.05, width, height);
    let mut renderer = Renderer {};

    let start_time = Instant::now();

    while !window.should_close() {
        handle_input(&mut glfw, &mut window, &mut camera, &events);

        animate(&mut car, start_time);

        renderer.render(&[&street_light, &car], &shader, &mut camera, width, height);
        window.swap_buffers();

        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn animate(car: &mut Solid, start_time: Instant) {
    let time = Instant::now().duration_since(start_time);
    let angle = ((time.as_millis() / 50) % 360) as f32;
    car.rot.y = angle.to_radians();
}

fn handle_input(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::Window,
    camera: &mut Camera,
    events: &std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
) {
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
    for (_, event) in glfw::flush_messages(events) {
        handle_window_event(window, event, camera);
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, camera: &mut Camera) {
    let (mouse_x, mouse_y) = window.get_cursor_pos();
    camera.adjust_look(mouse_x as f32, mouse_y as f32);

    match event {
        glfw::WindowEvent::Key(key, _, Action::Press, _) => match key {
            Key::Escape => window.set_should_close(true),
            Key::Enter => window.set_cursor_mode(CursorMode::Disabled),
            Key::RightShift => window.set_cursor_mode(CursorMode::Normal),
            _ => (),
        },
        _ => {}
    }
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
