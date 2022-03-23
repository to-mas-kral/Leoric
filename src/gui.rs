use egui::{CollapsingHeader, CtxRef, Ui};
use glam::Quat;

use crate::model::{Model, Node};

pub struct Gui {
    pub selected_node: Option<u32>,
    /// Default 0 (assuming that there is at least 1 model in the scene)
    pub selected_model: usize,
    /// If joints should e visible inside of the mesh
    pub debug_joints: bool,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            selected_node: None,
            selected_model: 0,
            debug_joints: true,
        }
    }

    pub fn render(&mut self, scene: &mut [Model], egui_ctx: &mut CtxRef) {
        self.gui_model_hierarchy_window(scene, egui_ctx);
        self.gui_joints_window(&mut scene[self.selected_model], egui_ctx);
        self.gui_side_panel(scene, egui_ctx);
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
        let is_selected = Some(node.id) == self.selected_node;
        let default_open = node.children.len() == 1;

        ui.horizontal(|ui| {
            let node_name = node.name.as_deref().unwrap_or("N/A");

            if !&node.children.is_empty() {
                let response = CollapsingHeader::new(node_name)
                    .id_source(node.id)
                    .default_open(default_open)
                    .selectable(true)
                    .selected(is_selected)
                    .show(ui, |ui| {
                        for child_node in &node.children {
                            self.gui_node(child_node, ui);
                        }
                    });

                if response.header_response.clicked() {
                    self.selected_node = Some(node.id);
                }
            } else {
                if ui.add(egui::Button::new(node_name)).clicked() {
                    self.selected_node = Some(node.id);
                }
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

                        // FIXME: if there are 2 or more unnamed nodes then we will have to do something about the IDs
                        CollapsingHeader::new(joint_name)
                            .default_open(false)
                            .show(ui, |ui| {
                                let trans = &mut joint.translation;
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

                                let (axis, angle) = joint.rotation.to_axis_angle();
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

                                joint.rotation =
                                    Quat::from_axis_angle(axis.normalize(), angle.to_radians());
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

    fn gui_side_panel(&mut self, scene: &[Model], egui_ctx: &mut CtxRef) {
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
        });
    }
}
