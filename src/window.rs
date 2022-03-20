use std::time::Instant;

use egui::{Color32, CtxRef};
use egui_backend::{painter::Painter, DpiScaling, EguiStateHandler};
use egui_sdl2_gl::ShaderVersion;
use eyre::{eyre, Result};
use sdl2::{
    event::Event,
    video::Window,
    video::{GLContext, GLProfile, SwapInterval},
    EventPump, Sdl, VideoSubsystem,
};

use egui_sdl2_gl as egui_backend;

pub struct MyWindow {
    _sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    window: Window,
    _gl_ctx: GLContext,
    pub event_pump: EventPump,

    pub egui_ctx: CtxRef,
    egui_state: EguiStateHandler,
    painter: Painter,
    start_time: Instant,
}

impl MyWindow {
    pub fn new(title: &str, dim: (u32, u32)) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(|e| eyre!("{e}"))?;
        let video_subsystem = sdl_context.video().map_err(|e| eyre!("{e}"))?;

        let window = video_subsystem
            .window(title, dim.0, dim.1)
            .opengl()
            .resizable()
            .position_centered()
            .allow_highdpi()
            .build()?;

        // Init OpenGL
        let gl_ctx = window.gl_create_context().map_err(|e| eyre!("{e}"))?;
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_major_version(4);
        gl_attr.set_context_minor_version(6);
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_double_buffer(true);

        window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::VSync)
            .map_err(|e| eyre!("{e}"))?;

        let shader_ver = ShaderVersion::Default;
        // On linux use GLES SL 100+, like so:
        //let shader_ver = ShaderVersion::Adaptive;
        let (painter, egui_state) =
            egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Custom(2.5));
        let egui_ctx = egui::CtxRef::default();
        let event_pump = sdl_context.event_pump().map_err(|e| eyre!("{e}"))?;

        Ok(Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            window,
            _gl_ctx: gl_ctx,
            event_pump,
            egui_ctx,
            egui_state,
            painter,
            start_time: Instant::now(),
        })
    }

    pub fn begin_frame(&mut self) {
        self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_ctx.begin_frame(self.egui_state.input.take());

        let mut visuals = egui::Visuals::default();
        visuals.override_text_color = Some(Color32::WHITE);
        self.egui_ctx.set_visuals(visuals);

        /* egui::SidePanel::new(Side::Right, "side_panel")
        .frame(Frame::group(&self.egui_ctx.style()).margin((10., 10.)))
        .show(&self.egui_ctx, |ui| {
            ui.add(Slider::new(&mut ambient_light, 0.0..=1.0).text("Ambientní osvětlení"));
            ui.separator();
        }); */
    }

    /// Finalizes the frame and returns if the render loop should terminate
    pub fn end_frame(&mut self) -> bool {
        let (egui_output, paint_cmds) = self.egui_ctx.end_frame();
        // Process ouput
        self.egui_state.process_output(&self.window, &egui_output);

        let paint_jobs = self.egui_ctx.tessellate(paint_cmds);

        if !egui_output.needs_repaint {
            // TODO: check egui_backend needs_repaint
            /* if let Some(event) = self.event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => return true,
                    _ => {
                        self.egui_state
                            .process_input(&self.window, event, &mut self.painter);
                    }
                }
            } */
        } else {
            self.painter
                .paint_jobs(None, paint_jobs, &self.egui_ctx.font_image());
            self.window.gl_swap_window();
        }
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return true,
                _ => {
                    self.egui_state
                        .process_input(&self.window, event, &mut self.painter);
                }
            }
        }

        false
    }
}
