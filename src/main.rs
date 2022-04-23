//! PGRF2 project - skeletal animation
//!
//! `main` function is the entry-point
use std::{thread, time::Duration};

use camera::Camera;
use eyre::Result;
use glam::Vec3;
use gui::Gui;
use model::Model;
use renderer::Renderer;
use sdl2::{keyboard::Scancode, EventPump};

use window::MyWindow;

/// A module for working with a basic free camera.
mod camera;

/// All of the code for drawing the GUI using egui.
mod gui;

/// Represents a single gltf 2.0 model (used models only have 1 scene).
mod model;

/// Handles rendering the whole scene.
mod renderer;

/// Abstractions for working with OpenGL.
mod ogl;

/// Handles window creation and egui boilerplate.
mod window;

/// Creates the window, configures OpenGL, sets up the scene and begins the render loop.
fn main() -> Result<()> {
    let mut window = MyWindow::new("PGRF2 Projekt - Skeletální Animace - Tomáš Král")?;

    ogl::init_debug();

    let mut scene = setup_scene()?;
    let mut gui = Gui::new();
    let mut renderer = Renderer::new()?;
    let mut camera = Camera::new(
        Vec3::new(0., 0., 0.),
        0.3,
        0.05,
        window.width,
        window.height,
    );

    'render_loop: loop {
        handle_inputs(&mut window.event_pump, &mut camera);

        window.begin_frame();

        renderer.render(&mut scene, &mut camera, &window, &gui);
        gui.prepare(&mut scene, &mut camera, &mut window.egui_ctx);

        let should_quit = window.end_frame();
        if should_quit {
            break 'render_loop;
        }

        thread::sleep(Duration::from_millis(3));
    }

    Ok(())
}

fn setup_scene() -> Result<Vec<Model>> {
    let mut scene = Vec::new();

    let mut add = |path: &str| -> Result<()> {
        let start = std::time::Instant::now();

        let model = Model::from_gltf(path)?;

        let time = std::time::Instant::now().duration_since(start);
        println!("Loading '{path}' took '{time:?}'");

        scene.push(model);
        Ok(())
    };

    add("resources/phoenix_bird/Bird.gltf")?;
    add("resources/animated_goblin_vs._vampire_spell_casting_loop/Duel.gltf")?;
    add("resources/dancing_stormtrooper/Stormtrooper.gltf")?;
    add("resources/animated_humanoid_robot/Droid.gltf")?;
    //add("resources/reap_the_whirlwind/Whirlwind.gltf")?;
    add("resources/toon_cat_free/Cat.gltf")?;
    add("resources/pakistan_girl_-_animated/Girl.gltf")?;
    add("resources/elephant_animation_idle/Elephant.gltf")?;

    add("resources/CesiumMan.glb")?;
    add("resources/RiggedFigure.gltf")?;
    add("resources/RiggedSimple.gltf")?;
    add("resources/Buggy.gltf")?;

    Ok(scene)
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

    if mouse_state.right() {
        camera.adjust_look(mouse_x, mouse_y);
    } else {
        camera.set_x_y(mouse_x, mouse_y)
    }
}
