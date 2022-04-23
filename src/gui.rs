use std::time::Instant;

use egui::{CollapsingHeader, CtxRef, RichText, Slider, Ui};
use glam::{Quat, Vec3};

use crate::{
    camera::Camera,
    model::{AnimationControl, Animations, Joint, Model, Node},
};

/// Contains the current state of the GUI.
/// Implements methods for displaying the widgets.
pub struct Gui {
    /// Default 0 (assuming that there is at least 1 model in the scene)
    pub selected_model: usize,
    /// If joints should be visible inside of the mesh
    pub draw_skeleton: bool,
    /// If the mesh should be visible
    pub mesh_visible: bool,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            selected_model: 0,
            draw_skeleton: false,
            mesh_visible: true,
        }
    }

    pub fn prepare(&mut self, scene: &mut [Model], camera: &mut Camera, egui_ctx: &mut CtxRef) {
        self.gui_model_hierarchy_window(scene, egui_ctx);
        self.gui_joints_window(&mut scene[self.selected_model], egui_ctx);
        self.gui_side_panel(scene, camera, egui_ctx);
    }

    fn gui_model_hierarchy_window(&mut self, scene: &[Model], egui_ctx: &mut CtxRef) {
        let model = &scene[self.selected_model];

        egui::Window::new("Model Hierarchy")
            .scroll2([false, true])
            .resizable(true)
            .show(egui_ctx, |ui| {
                self.gui_node(&model.root, ui);
            });
    }

    fn gui_node(&mut self, node: &Node, ui: &mut Ui) {
        let default_open = node.children.len() == 1;

        ui.horizontal(|ui| {
            if !&node.children.is_empty() {
                CollapsingHeader::new(&node.name)
                    .id_source(node.index)
                    .default_open(default_open)
                    .selectable(true)
                    .show(ui, |ui| {
                        for child_node in &node.children {
                            self.gui_node(child_node, ui);
                        }
                    });
            } else {
                ui.label(&node.name);
            }

            if let Some(mesh) = &node.mesh {
                ui.separator();

                let mesh_name = mesh.name.as_deref().unwrap_or("N/A");
                ui.add(egui::Label::new(mesh_name));

                ui.end_row()
            }
        });
    }

    fn gui_joints_window(&mut self, model: &mut Model, egui_ctx: &mut CtxRef) {
        self.gui_joints_window_helper(&mut model.root, &mut model.animations, egui_ctx);
    }

    fn gui_joints_window_helper(
        &mut self,
        node: &mut Node,
        animations: &mut Animations,
        egui_ctx: &mut CtxRef,
    ) {
        if let Some(joints) = &mut node.joints {
            egui::Window::new("Joints").show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for joint in joints.joints.iter_mut() {
                        let joint_name = &joint.name;

                        CollapsingHeader::new(joint_name).show(ui, |ui| {
                            Self::show_joint_transforms(joint, animations, ui);
                        });
                    }
                });
            });
        } else {
            // I assume there is only 1 skeleton in the models we are going to work with
            for child_node in &mut node.children {
                self.gui_joints_window_helper(child_node, animations, egui_ctx);
            }
        }
    }

    fn show_joint_transforms(joint: &mut Joint, animations: &mut Animations, ui: &mut Ui) {
        let trans = &mut joint.transform.translation;
        let (axis, angle) = joint.transform.rotation.to_axis_angle();
        let mut angle = angle.to_degrees();

        let response = ui.group(|ui| {
            ui.label("Translation");
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::DragValue::new(&mut trans.x).speed(0.03));
                ui.label("y");
                ui.add(egui::DragValue::new(&mut trans.y).speed(0.03));
                ui.label("z");
                ui.add(egui::DragValue::new(&mut trans.z).speed(0.03));
            });

            ui.label("Rotation");
            ui.horizontal(|ui| {
                ui.label("angle");
                ui.add(
                    egui::DragValue::new(&mut angle)
                        .speed(1.0)
                        .clamp_range((0.1)..=(359.9)),
                );
            });
        });

        if response.response.hovered() {
            animations.animation_control = AnimationControl::Static;
        }

        joint.transform.rotation = Quat::from_axis_angle(axis.normalize(), angle.to_radians());
    }

    fn gui_side_panel(&mut self, scene: &mut [Model], camera: &mut Camera, egui_ctx: &mut CtxRef) {
        egui::SidePanel::right("Side Panel").show(egui_ctx, |ui| {
            ui.group(|ui| {
                ui.add(egui::Label::new(RichText::new("Scenes").heading().strong()));
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, model) in scene.iter().enumerate() {
                        if ui.button(&model.name).clicked() {
                            self.selected_model = i;
                        }
                    }
                });
            });

            ui.group(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("Settings").heading().strong(),
                ));

                ui.separator();

                if ui.button("Debug joints").clicked() {
                    self.draw_skeleton = !self.draw_skeleton;
                }

                if ui.button("Draw mesh").clicked() {
                    self.mesh_visible = !self.mesh_visible;
                }

                ui.add(
                    Slider::new(&mut camera.move_speed, 0.0..=15.)
                        .text("Camera move speed")
                        .smart_aim(false),
                );

                if ui.button("Reset Camera").clicked() {
                    camera.set_pos(Vec3::new(0.0, 0.0, 3.0));
                }

                egui::global_dark_light_mode_switch(ui);
            });

            ui.group(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("Animations").heading().strong(),
                ));

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_animation_view(scene, ui);
                });
            });
        });
    }

    fn show_animation_view(&mut self, scene: &mut [Model], ui: &mut Ui) {
        let selected_model = &mut scene[self.selected_model];
        let animations = &mut selected_model.animations;
        for (i, animation) in animations.animations.iter_mut().enumerate() {
            ui.group(|ui| {
                let response = ui.add(
                    Slider::new(&mut animation.current_time, 0.0..=animation.end_time)
                        .text("Animation time")
                        .smart_aim(false),
                );

                if response.clicked() || response.dragged() || response.changed() {
                    animations.animation_control = AnimationControl::Controllable {
                        active_animation: i,
                    };
                }

                if let AnimationControl::Loop {
                    active_animation: _,
                    start_time: _,
                } = animations.animation_control
                {
                    ui.ctx().request_repaint();
                }

                if ui.button("Play").clicked() {
                    match animations.animation_control {
                        AnimationControl::Static
                        | AnimationControl::Controllable {
                            active_animation: _,
                        } => {
                            animations.animation_control = AnimationControl::Loop {
                                active_animation: i,
                                start_time: Instant::now(),
                            }
                        }
                        AnimationControl::Loop {
                            active_animation: _,
                            start_time: _,
                        } => {
                            animations.animation_control = AnimationControl::Controllable {
                                active_animation: i,
                            }
                        }
                    };
                }
            });
        }
    }
}
