use egui::{CollapsingHeader, CtxRef, Ui};
use glam::{Quat, Vec3};

use crate::{
    gui_state::GuiState,
    model::{Model, Node},
};

pub fn gui(scene: &mut [Model], egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
    gui_model_hierarchy_window(scene, egui_ctx, gui_state);
    gui_scene_window(scene, egui_ctx, gui_state);
    gui_controls_window(egui_ctx, gui_state);
    gui_joints_window(&mut scene[gui_state.selected_model], egui_ctx, gui_state);
}

fn gui_controls_window(egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
    egui::Window::new("Controls").show(&egui_ctx, |ui| {
        if ui.button("Debug joints").clicked() {
            gui_state.debug_joints = !gui_state.debug_joints;
        }
    });
}

fn gui_model_hierarchy_window(scene: &[Model], egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
    let model = &scene[gui_state.selected_model];

    egui::Window::new("Model Hierarchy")
        .scroll2([false, true])
        .resizable(true)
        .show(&egui_ctx, |ui| {
            gui_node(&model.root, ui, gui_state);
        });
}

fn gui_node(node: &Node, ui: &mut Ui, gui_state: &mut GuiState) {
    let is_selected = Some(node.id) == gui_state.selected_node;
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
                        gui_node(child_node, ui, gui_state);
                    }
                });

            if response.header_response.clicked() {
                gui_state.selected_node = Some(node.id);
            }
        } else {
            if ui.add(egui::Button::new(node_name)).clicked() {
                gui_state.selected_node = Some(node.id);
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

fn gui_scene_window(scene: &[Model], egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
    egui::Window::new("Scene select").show(&egui_ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, model) in scene.iter().enumerate() {
                if ui.button(&model.name).clicked() {
                    gui_state.selected_model = i;
                }
            }
        });
    });
}

fn gui_joints_window(model: &mut Model, egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
    gui_joints_window_helper(&mut model.root, egui_ctx, gui_state);
}

fn gui_joints_window_helper(node: &mut Node, egui_ctx: &mut CtxRef, gui_state: &mut GuiState) {
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

                            let rot = joint.rotation;
                            let (axis, mut angle) = rot.to_axis_angle();

                            ui.label("Rotation");
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("angle");
                                    ui.add(egui::DragValue::new(&mut angle).speed(0.03));
                                });
                            });

                            joint.rotation = Quat::from_axis_angle(axis.normalize(), angle);
                        });
                }
            });
        });
    } else {
        // I assume there is only 1 skeleton in the models we are going to work with
        for child_node in &mut node.children {
            gui_joints_window_helper(child_node, egui_ctx, gui_state);
        }
    }
}
