use egui::{
    Button, Color32, Frame, Margin, RichText, SidePanel, Stroke, TextEdit, TextStyle,
    TopBottomPanel, Sense,
};

use super::app::App;
use super::constants::{
    COLOR_DIM, COLOR_PANEL_BG, COLOR_PORT_INPUT, COLOR_PORT_OUTPUT, COLOR_SIGNAL_HIGH,
    FONT_SIZE_LARGE, FONT_SIZE_BODY, FONT_SIZE_SUBHEAD,
    FONT_SIZE_ICON, FONT_SIZE_SMALL,
};

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Top bar
    // ─────────────────────────────────────────────────────────────────────────

    pub fn show_top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel")
            .exact_height(52.0)
            .frame(
                Frame::none()
                    .fill(Color32::from_rgb(18, 20, 30))
                    .inner_margin(Margin::symmetric(12.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("⚡").size(FONT_SIZE_LARGE));
                    ui.add(
                        TextEdit::singleline(&mut self.title)
                            .desired_width(200.0)
                            .font(TextStyle::Heading),
                    );

                    ui.separator();

                    if ui
                        .button(RichText::new("💾  Save Gate").color(Color32::from_rgb(150, 210, 255)))
                        .on_hover_text("Save current circuit as a reusable gate in the library")
                        .clicked()
                    {
                        self.save_current_graph_to_library();
                    }

                    if ui.button("🗑  Clear").on_hover_text("Reset canvas").clicked() {
                        self.clear_canvas();
                    }

                    ui.separator();

                    let run_button_text = if self.simulation_running {
                        RichText::new("⏸  Pause").color(Color32::YELLOW)
                    } else {
                        RichText::new("▶  Run").color(COLOR_SIGNAL_HIGH)
                    };
                    if ui.button(run_button_text).clicked() {
                        if self.simulation_running {
                            self.simulation_running = false;
                        } else {
                            self.build_simulation_from_graph();
                            self.simulation_running = true;
                        }
                    }

                    if ui.button("⏭  Step").on_hover_text("Rebuild & advance one tick").clicked() {
                        self.simulation_running = false;
                        if self.simulation.is_none() {
                            self.build_simulation_from_graph();
                        }
                        self.step_simulation();
                    }

                    ui.separator();

                    if let Some(error_message) = &self.simulation_error.clone() {
                        ui.label(
                            RichText::new(format!("⚠  {error_message}"))
                                .color(Color32::RED)
                                .size(FONT_SIZE_BODY),
                        );
                    } else if self.simulation.is_some() {
                        ui.label(
                            RichText::new(if self.simulation_running { "● Running" } else { "● Built" })
                                .color(COLOR_SIGNAL_HIGH)
                                .size(FONT_SIZE_SUBHEAD),
                        );
                    }
                });
            });
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Left panel — inputs
    // ─────────────────────────────────────────────────────────────────────────

    pub fn show_left_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("left_panel")
            .resizable(false)
            .exact_width(158.0)
            .frame(
                Frame::none()
                    .fill(COLOR_PANEL_BG)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(55, 62, 90)))
                    .inner_margin(Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("INPUTS").color(COLOR_PORT_INPUT).strong());
                ui.separator();

                let mut input_to_remove: Option<usize> = None;
                let mut reorder_from_to: Option<(usize, usize)> = None;

                let dragging_index: Option<usize> = self.input_drag_reorder.map(|(d, _)| d);

                let mut row_centre_ys: Vec<f32> = Vec::with_capacity(self.graph.inputs.len());

                for input_index in 0..self.graph.inputs.len() {
                    let is_drop_target = self
                        .input_drag_reorder
                        .map(|(_, target_index)| target_index == input_index)
                        .unwrap_or(false)
                        && dragging_index != Some(input_index);

                    if is_drop_target {
                        let available = ui.available_rect_before_wrap();
                        ui.painter().hline(
                            available.x_range(),
                            available.top(),
                            Stroke::new(2.0, COLOR_PORT_INPUT),
                        );
                    }

                    let row_response = ui.horizontal(|ui| {
                        let handle_response = ui.add(
                            Button::new(RichText::new("⠿").color(COLOR_DIM).size(FONT_SIZE_ICON))
                                .frame(false)
                                .sense(Sense::drag()),
                        );

                        let is_on = self.input_states.get(input_index).copied().unwrap_or(false);
                        if ui.small_button(if is_on { "🟢" } else { "⚫" }).clicked() {
                            if let Some(state) = self.input_states.get_mut(input_index) {
                                *state = !*state;
                            }
                        }
                        ui.add(
                            TextEdit::singleline(&mut self.graph.inputs[input_index])
                                .desired_width(55.0)
                                .font(TextStyle::Small),
                        );
                        if ui.small_button("✖").clicked() {
                            input_to_remove = Some(input_index);
                        }

                        handle_response
                    });

                    let row_rect = row_response.response.rect;
                    row_centre_ys.push(row_rect.center().y);

                    let handle_response = row_response.inner;

                    if handle_response.drag_started() {
                        self.input_drag_reorder = Some((input_index, input_index));
                    }

                    if handle_response.dragged() {
                        if let Some(pointer_pos) = ui.input(|input_state| input_state.pointer.interact_pos()) {
                            let target_index = closest_index_to_y(pointer_pos.y, &row_centre_ys);
                            if let Some((dragged_index, _)) = self.input_drag_reorder {
                                self.input_drag_reorder = Some((dragged_index, target_index));
                            }
                        }
                    }

                    if handle_response.drag_stopped() {
                        if let Some((dragged_index, target_index)) = self.input_drag_reorder.take() {
                            if dragged_index != target_index {
                                reorder_from_to = Some((dragged_index, target_index));
                            }
                        }
                    }
                }

                if let Some((old_index, new_index)) = reorder_from_to {
                    if old_index < self.input_states.len() && new_index < self.input_states.len() {
                        if old_index < new_index {
                            self.input_states[old_index..=new_index].rotate_left(1);
                        } else {
                            self.input_states[new_index..=old_index].rotate_right(1);
                        }
                    }
                    self.graph.reorder_input(old_index, new_index);
                }

                if let Some(index) = input_to_remove {
                    self.graph.remove_input(index);
                    if index < self.input_states.len() {
                        self.input_states.remove(index);
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.new_input_name)
                            .desired_width(80.0)
                            .hint_text("name…"),
                    );
                    if ui.small_button("＋").clicked() {
                        let name = if self.new_input_name.is_empty() {
                            format!("I{}", self.graph.inputs.len())
                        } else {
                            std::mem::take(&mut self.new_input_name)
                        };
                        self.graph.add_input(name); // Changed from .push()
                        self.input_states.push(false);
                    }
                });

                ui.add_space(16.0);
                ui.label(RichText::new("TIPS").color(COLOR_DIM).strong().size(FONT_SIZE_SMALL));
                for hint_text in &[
                    "Right-click canvas\nto spawn gates",
                    "Click output (yellow)\nthen input (blue)\nto connect",
                    "Shift+drag on canvas\nto bulk-wire ports",
                    "Right-click a wire\nto delete it",
                    "Middle-drag/scroll\nto pan & zoom",
                    "Hover + Del\nto delete a gate",
                    "Drag ⠿ to reorder\nports",
                ] {
                    ui.label(RichText::new(*hint_text).color(COLOR_DIM).size(FONT_SIZE_SMALL));
                    ui.add_space(3.0);
                }
            });
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Right panel — outputs & library
    // ─────────────────────────────────────────────────────────────────────────

    pub fn show_right_panel(&mut self, ctx: &egui::Context) {
        SidePanel::right("right_panel")
            .resizable(false)
            .exact_width(168.0)
            .frame(
                Frame::none()
                    .fill(COLOR_PANEL_BG)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(55, 62, 90)))
                    .inner_margin(Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("OUTPUTS").color(COLOR_PORT_OUTPUT).strong());
                ui.separator();

                let mut output_to_remove: Option<usize> = None;
                let mut reorder_from_to: Option<(usize, usize)> = None;

                let dragging_index: Option<usize> = self.output_drag_reorder.map(|(d, _)| d);

                let mut row_centre_ys: Vec<f32> = Vec::with_capacity(self.graph.outputs.len());

                for output_index in 0..self.graph.outputs.len() {
                    let is_drop_target = self
                        .output_drag_reorder
                        .map(|(_, target_index)| target_index == output_index)
                        .unwrap_or(false)
                        && dragging_index != Some(output_index);

                    if is_drop_target {
                        let available = ui.available_rect_before_wrap();
                        ui.painter().hline(
                            available.x_range(),
                            available.top(),
                            Stroke::new(2.0, COLOR_PORT_OUTPUT),
                        );
                    }

                    let row_response = ui.horizontal(|ui| {
                        let handle_response = ui.add(
                            Button::new(RichText::new("⠿").color(COLOR_DIM).size(FONT_SIZE_ICON))
                                .frame(false)
                                .sense(Sense::drag()),
                        );

                        let is_on = self.output_states.get(output_index).copied().unwrap_or(false);
                        ui.label(if is_on { "🟡" } else { "⚫" });
                        ui.add(
                            TextEdit::singleline(&mut self.graph.outputs[output_index])
                                .desired_width(60.0)
                                .font(TextStyle::Small),
                        );
                        if ui.small_button("✖").clicked() {
                            output_to_remove = Some(output_index);
                        }

                        handle_response
                    });

                    let row_rect = row_response.response.rect;
                    row_centre_ys.push(row_rect.center().y);

                    let handle_response = row_response.inner;

                    if handle_response.drag_started() {
                        self.output_drag_reorder = Some((output_index, output_index));
                    }

                    if handle_response.dragged() {
                        if let Some(pointer_pos) = ui.input(|input_state| input_state.pointer.interact_pos()) {
                            let target_index = closest_index_to_y(pointer_pos.y, &row_centre_ys);
                            if let Some((dragged_index, _)) = self.output_drag_reorder {
                                self.output_drag_reorder = Some((dragged_index, target_index));
                            }
                        }
                    }

                    if handle_response.drag_stopped() {
                        if let Some((dragged_index, target_index)) = self.output_drag_reorder.take() {
                            if dragged_index != target_index {
                                reorder_from_to = Some((dragged_index, target_index));
                            }
                        }
                    }
                }

                if let Some((old_index, new_index)) = reorder_from_to {
                    if old_index < self.output_states.len() && new_index < self.output_states.len() {
                        if old_index < new_index {
                            self.output_states[old_index..=new_index].rotate_left(1);
                        } else {
                            self.output_states[new_index..=old_index].rotate_right(1);
                        }
                    }
                    self.graph.reorder_output(old_index, new_index);
                }

                if let Some(index) = output_to_remove {
                    self.graph.remove_output(index);
                    if index < self.output_states.len() {
                        self.output_states.remove(index);
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.new_output_name)
                            .desired_width(80.0)
                            .hint_text("name…"),
                    );
                    if ui.small_button("＋").clicked() {
                        let name = if self.new_output_name.is_empty() {
                            format!("O{}", self.graph.outputs.len())
                        } else {
                            std::mem::take(&mut self.new_output_name)
                        };
                        self.graph.add_output(name); // Changed from .push()
                        self.output_states.push(false);
                    }
                });

                // ── Library ───────────────────────────────────────────────────

                ui.add_space(12.0);
                ui.label(RichText::new("LIBRARY").color(COLOR_DIM).strong().size(FONT_SIZE_BODY));
                ui.separator();

                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("Load").color(Color32::from_rgb(150, 210, 255)))
                        .on_hover_text("Load library from file")
                        .clicked()
                    {
                        self.load_library_from_file();
                    }
                    if !self.library.is_empty() {
                        if ui
                            .button(RichText::new("Save").color(Color32::from_rgb(150, 210, 255)))
                            .on_hover_text("Save library to file")
                            .clicked()
                        {
                            self.save_library_to_file();
                        }
                    }
                });

                if !self.library.is_empty() {
                    ui.label(
                        RichText::new("Right-click to manage")
                            .color(COLOR_DIM)
                            .italics()
                            .size(FONT_SIZE_SMALL),
                    );
                    ui.add_space(4.0);

                    let mut gate_to_open:   Option<usize> = None;
                    let mut gate_to_delete: Option<usize> = None;
                    let mut gate_to_rename: Option<usize> = None;
                    let mut commit_rename = false;

                    for (library_index, saved_gate) in self.library.iter().enumerate() {
                        let button_label = format!(
                            "▣ {}  ({} → {})",
                            saved_gate.name, saved_gate.input_count, saved_gate.output_count
                        );

                        let is_renaming = self.library_rename_index == Some(library_index);

                        if is_renaming {
                            ui.horizontal(|ui| {
                                ui.add(
                                    TextEdit::singleline(&mut self.library_rename_text)
                                        .desired_width(100.0)
                                        .font(TextStyle::Small),
                                );
                                if ui.small_button("✔").clicked() {
                                    commit_rename = true;
                                }
                            });
                            continue;
                        }

                        let button_response = ui.add(
                            Button::new(
                                RichText::new(&button_label)
                                    .size(FONT_SIZE_BODY)
                                    .color(Color32::from_rgb(180, 200, 255)),
                            )
                            .frame(false),
                        );

                        button_response.context_menu(|ui| {
                            ui.label(RichText::new(&saved_gate.name).strong().size(FONT_SIZE_SUBHEAD));
                            ui.separator();
                            if ui.button("📂  Open for editing").clicked() {
                                gate_to_open = Some(library_index);
                                ui.close_menu();
                            }
                            if ui.button("✏  Rename").clicked() {
                                gate_to_rename = Some(library_index);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(RichText::new("🗑  Delete").color(Color32::RED)).clicked() {
                                gate_to_delete = Some(library_index);
                                ui.close_menu();
                            }
                        });
                    }

                    if commit_rename {
                        if let Some(rename_index) = self.library_rename_index {
                            let new_name = self.library_rename_text.trim().to_string();
                            if !new_name.is_empty() {
                                self.library[rename_index].name        = new_name.clone();
                                let updated_gate = self.library[rename_index].clone();
                                Self::update_saved_gate_instances_in_graph(
                                    &mut self.graph,
                                    rename_index,
                                    &updated_gate,
                                );
                                let mut library = std::mem::take(&mut self.library);
                                for library_gate in &mut library {
                                    Self::update_saved_gate_instances_in_graph(
                                        &mut library_gate.graph,
                                        rename_index,
                                        &updated_gate,
                                    );
                                }
                                self.library = library;
                            }
                        }
                        self.library_rename_index = None;
                        self.library_rename_text.clear();
                    }
                    if let Some(index) = gate_to_rename {
                        self.library_rename_text  = self.library[index].name.clone();
                        self.library_rename_index = Some(index);
                    }
                    if let Some(index) = gate_to_open {
                        self.open_library_gate_for_editing(index);
                    }
                    if let Some(index) = gate_to_delete {
                        self.delete_library_gate(index);
                    }
                }
            });
    }
}

fn closest_index_to_y(pointer_y: f32, row_centre_ys: &[f32]) -> usize {
    if row_centre_ys.is_empty() {
        return 0;
    }
    let mut closest_index = 0;
    let mut closest_distance = f32::MAX;
    for (row_index, &centre_y) in row_centre_ys.iter().enumerate() {
        let distance = (pointer_y - centre_y).abs();
        if distance < closest_distance {
            closest_distance = distance;
            closest_index = row_index;
        }
    }
    closest_index
}