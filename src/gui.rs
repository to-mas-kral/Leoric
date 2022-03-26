use std::time::Instant;

use egui::{CollapsingHeader, CtxRef, Slider, Ui};
use glam::Quat;

use crate::{
    camera::Camera,
    model::{AnimationControl, Model, Node},
};

pub struct Gui {
    /// Default 0 (assuming that there is at least 1 model in the scene)
    pub selected_model: usize,
    /// If joints should e visible inside of the mesh
    pub debug_joints: bool,
    /// If polygons should be drawn as wireframes
    pub wireframe: bool,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            selected_model: 0,
            debug_joints: true,
            wireframe: false,
        }
    }

    pub fn render(&mut self, scene: &mut [Model], camera: &mut Camera, egui_ctx: &mut CtxRef) {
        self.gui_model_hierarchy_window(scene, egui_ctx);
        self.gui_joints_window(&mut scene[self.selected_model], egui_ctx);
        self.gui_animations_window(&mut scene[self.selected_model], egui_ctx);
        self.gui_side_panel(scene, camera, egui_ctx);
    }

    fn gui_model_hierarchy_window(&mut self, scene: &[Model], egui_ctx: &mut CtxRef) {
        let model = &scene[self.selected_model];

        egui::Window::new("Model Hierarchy")
            .scroll2([false, true])
            .resizable(true)
            .show(&egui_ctx, |ui| {
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
        self.gui_joints_window_helper(&mut model.root, egui_ctx);
    }

    fn gui_joints_window_helper(&mut self, node: &mut Node, egui_ctx: &mut CtxRef) {
        if let Some(joints) = &mut node.joints {
            egui::Window::new("Joints").show(&egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for joint in joints.joints.iter_mut() {
                        let joint_name = &joint.name;

                        CollapsingHeader::new(joint_name)
                            .default_open(true)
                            .show(ui, |ui| {
                                Self::show_joint_transforms(joint, ui);
                            });
                    }
                });
            });
        } else {
            // I assume there is only 1 skeleton in the models we are going to work with
            for child_node in &mut node.children {
                self.gui_joints_window_helper(child_node, egui_ctx);
            }
        }
    }

    fn show_joint_transforms(joint: &mut crate::model::Joint, ui: &mut Ui) {
        let trans = &mut joint.transform.translation;

        ui.label("Translation");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::DragValue::new(&mut trans.x).speed(0.03));
                ui.label("y");
                ui.add(egui::DragValue::new(&mut trans.y).speed(0.03));
                ui.label("z");
                ui.add(egui::DragValue::new(&mut trans.z).speed(0.03));
            });
        });

        let scale = &mut joint.transform.scale;

        ui.label("Scale");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::DragValue::new(&mut scale.x).speed(0.01));
                ui.label("y");
                ui.add(egui::DragValue::new(&mut scale.y).speed(0.01));
                ui.label("z");
                ui.add(egui::DragValue::new(&mut scale.z).speed(0.01));
            });
        });

        let (axis, angle) = joint.transform.rotation.to_axis_angle();
        let mut angle = angle.to_degrees();

        ui.label("Rotation");
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("angle");
                ui.add(
                    egui::DragValue::new(&mut angle)
                        .speed(1.0)
                        .clamp_range((0.1)..=(359.9)),
                );
            });
        });

        joint.transform.rotation = Quat::from_axis_angle(axis.normalize(), angle.to_radians());
    }

    fn gui_side_panel(&mut self, scene: &[Model], camera: &mut Camera, egui_ctx: &mut CtxRef) {
        egui::SidePanel::right("Side Panel").show(egui_ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, model) in scene.iter().enumerate() {
                    if ui.button(&model.name).clicked() {
                        self.selected_model = i;
                    }
                }
            });

            ui.separator();

            if ui.button("Debug joints").clicked() {
                self.debug_joints = !self.debug_joints;
            }

            if ui.button("Wireframe").clicked() {
                self.wireframe = !self.wireframe;
            }

            ui.add(Slider::new(&mut camera.move_speed, 0.0..=100.).text("Camera move speed"));

            egui::global_dark_light_mode_switch(ui);
        });
    }

    fn gui_animations_window(&self, selected_model: &mut Model, egui_ctx: &mut CtxRef) {
        egui::Window::new("Animations")
            .scroll2([false, true])
            .resizable(true)
            .show(&egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let animations = &mut selected_model.animations;
                    for (i, animation) in animations.animations.iter_mut().enumerate() {
                        ui.group(|ui| {
                            let response = ui.add(
                                Slider::new(&mut animation.current_time, 0.0..=animation.end_time)
                                    .text("Animation time"),
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
                                egui_ctx.request_repaint();
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
                                        animations.animation_control =
                                            AnimationControl::Controllable {
                                                active_animation: i,
                                            }
                                    }
                                };
                            }
                        });
                    }
                });
            });
    }
}
