use std::{
    ffi::{c_void, CStr},
    ptr, thread,
    time::{Duration, Instant},
};

use camera::Camera;
use egui::{panel::Side, Color32, Slider};
use egui_backend::{DpiScaling, ShaderVersion};
use eyre::Result;
use glam::Vec3;
use model::Model;
use renderer::Renderer;
use sdl2::{
    event::Event,
    keyboard::Scancode,
    video::{GLProfile, SwapInterval},
    EventPump,
};
use shader::Shader;

use egui_sdl2_gl as egui_backend;

mod camera;
mod model;
mod renderer;
mod shader;

fn main() -> Result<()> {
    // TODO: error handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let width = 1 * 1920u32;
    let height = 1 * 1080u32;

    let window = video_subsystem
        .window(
            "PGRF2 Projekt - Skeletální Animace - Tomáš Král",
            width,
            height,
        )
        .opengl()
        .resizable()
        .position_centered()
        .allow_highdpi()
        .build()?;

    // Init OpenGL
    let _gl_ctx = window.gl_create_context().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_major_version(4);
    gl_attr.set_context_minor_version(6);
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_flags().debug().set();
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);

    window
        .subsystem()
        .gl_set_swap_interval(SwapInterval::VSync)
        .unwrap();

    let shader_ver = ShaderVersion::Default;
    // On linux use GLES SL 100+, like so:
    //let shader_ver = ShaderVersion::Adaptive;
    let (mut painter, mut egui_state) =
        egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Custom(2.0));
    let mut egui_ctx = egui::CtxRef::default();
    let mut event_pump = sdl_context.event_pump().unwrap();

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

    // Winodw init
    //window.set_cursor_pos((width as f64) / 2., (height as f64) / 2.);

    // Shaders
    let shader = Shader::from_file("shaders/vs.vert", "shaders/fs.frag")?;

    // Scene setup
    let mut scene: Vec<&Model> = Vec::new();

    // This work is based on "Lancia Fulvia rallye"
    // (https://sketchfab.com/3d-models/lancia-fulvia-rallye-5f02ef9e0daf481aba8c8b51216c0a6b)
    // by Floppy (https://sketchfab.com/fastolfe) licensed under CC-BY-NC-4.0
    // (http://creativecommons.org/licenses/by-nc/4.0/)
    //let car_gltf = Model::from_gltf("resources/lancia_fulvia_rallye/scene.gltf")?;
    //scene.push(&car_gltf);

    let boiler = Model::from_gltf("resources/donkey_boiler/scene.gltf")?;
    scene.push(&boiler);

    //let figure = Model::from_gltf("resources/RiggedSimple.gltf")?;
    //scene.push(&figure);

    let mut camera = Camera::new(Vec3::new(0., 0., 0.), 0.3, 0.05, width, height);
    let mut renderer = Renderer::new(shader);

    let start_time = Instant::now();

    let mut ambient_light = 1f32;

    'running: loop {
        handle_inputs(&mut event_pump, &mut camera);

        //
        // EGUI
        //
        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::SidePanel::new(Side::Right, "side_panel").show(&egui_ctx, |ui| {
            ui.add(
                Slider::new(&mut ambient_light, 0.0..=1.0)
                    .text("Ambientní osvětlení")
                    .text_color(Color32::WHITE),
            );
            ui.separator();
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        // Process ouput
        egui_state.process_output(&window, &egui_output);

        // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
        // if egui_ctx.used_size() != painter.screen_rect.size() {
        //     println!("resized.");
        //     let _size = egui_ctx.used_size();
        //     let (w, h) = (_size.x as u32, _size.y as u32);
        //     window.set_size(w, h).unwrap();
        // }

        let paint_jobs = egui_ctx.tessellate(paint_cmds);
        //
        // EGUI
        //

        unsafe {
            //gl::Viewport(0, 0, width as i32, height as i32);
            gl::Enable(gl::DEPTH_TEST);
            //TODO: test culling
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::PolygonMode(gl::FRONT, gl::FILL);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        renderer.render(&scene, &mut camera, width, height, ambient_light);

        unsafe {
            // Disable backface culling, otherwise egui doesn't render correctly
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
        }

        if !egui_output.needs_repaint {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            painter.paint_jobs(None, paint_jobs, &egui_ctx.font_image());
            window.gl_swap_window();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn handle_inputs(event_pump: &mut EventPump, camera: &mut Camera) {
    let k = event_pump.keyboard_state();

    if k.is_scancode_pressed(Scancode::W) {
        camera.move_forward(1.0);
    }

    if k.is_scancode_pressed(Scancode::S) {
        camera.move_backward(1.0);
    }

    if k.is_scancode_pressed(Scancode::A) {
        camera.strafe_left(1.0);
    }

    if k.is_scancode_pressed(Scancode::D) {
        camera.strafe_right(1.0);
    }

    let mouse_state = event_pump.mouse_state();
    let mouse_x = mouse_state.x() as f32;
    let mouse_y = mouse_state.y() as f32;

    if mouse_state.left() {
        camera.adjust_look(mouse_x, mouse_y);
    } else {
        camera.set_x_y(mouse_x, mouse_y)
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
