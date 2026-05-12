use egui::{
    Align2, CentralPanel, Color32, FontId, Frame, Key, Painter, PointerButton, Pos2, Rect,
    Response, RichText, Rounding, Sense, Stroke, pos2, vec2,
};

use crate::sim_builder::PortRef;

use super::app::App;
use super::constants::{
    COLOR_BACKGROUND, COLOR_BOX_SELECT, COLOR_BOX_SELECT_BORDER, COLOR_DIM,
    COLOR_PORT_INPUT, COLOR_PORT_OUTPUT, COLOR_SIGNAL_HIGH, COLOR_WIRE_PENDING,
    GRID_CELL_SIZE, IO_RAIL_STEP, PORT_RADIUS,
};
use super::geometry::{
    canvas_rect_from_two_points, draw_bezier_wire, input_port_canvas_pos,
    make_nand_node, make_saved_gate_node, output_port_canvas_pos, snap_to_grid,
};
use super::graph::{BulkWireState, EditorNode, EditorNodeKind, Wire};

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Canvas panel
    // ─────────────────────────────────────────────────────────────────────────

    pub fn show_canvas(&mut self, ctx: &egui::Context) {
        CentralPanel::default()
            .frame(Frame::none().fill(COLOR_BACKGROUND))
            .show(ctx, |ui| {
                let (canvas_response, painter) =
                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                let canvas_origin = canvas_response.rect.min;
                let canvas_rect   = canvas_response.rect;

                // ── Zoom (scroll wheel) ────────────────────────────────────
                let scroll_delta = ui.input(|input| input.smooth_scroll_delta.y);
                if scroll_delta != 0.0 && canvas_response.hovered() {
                    self.canvas_zoom =
                        (self.canvas_zoom * (1.0 + scroll_delta * 0.0012)).clamp(0.25, 4.0);
                }

                // ── Pan (middle-mouse drag) ────────────────────────────────
                if canvas_response.dragged_by(PointerButton::Middle) {
                    self.canvas_pan += canvas_response.drag_delta() / self.canvas_zoom;
                }

                let pointer_screen_pos = ui.input(|input| input.pointer.interact_pos());
                let shift_held         = ui.input(|input| input.modifiers.shift);

                let hovered_port       = pointer_screen_pos
                    .and_then(|pos| self.hit_test_port(pos, canvas_origin, canvas_rect));
                let hovered_gate_index = pointer_screen_pos
                    .and_then(|pos| self.hit_test_gate(pos, canvas_origin));
                let hovered_wire       = pointer_screen_pos
                    .and_then(|pos| self.hit_test_wire(pos, canvas_origin, canvas_rect));

                // ── End gate drag ──────────────────────────────────────────
                if canvas_response.drag_stopped() {
                    self.dragging_gate = None;
                }

                // ── Bulk-wire box-select (Shift+drag) ──────────────────────
                self.update_bulk_wire(
                    &canvas_response,
                    pointer_screen_pos,
                    shift_held,
                    canvas_origin,
                    canvas_rect,
                );

                // ── Gate dragging ──────────────────────────────────────────
                if matches!(self.bulk_wire_state, BulkWireState::Idle) {
                    if let Some((dragged_gate_index, drag_offset)) = self.dragging_gate {
                        if canvas_response.dragged_by(PointerButton::Primary) {
                            if let Some(pointer_pos) = pointer_screen_pos {
                                let raw_canvas_pos =
                                    self.screen_to_canvas(pointer_pos, canvas_origin) + drag_offset;
                                self.graph.nodes[dragged_gate_index].pos = Pos2::new(
                                    (raw_canvas_pos.x / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
                                    (raw_canvas_pos.y / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
                                );
                            }
                        }
                    } else if canvas_response.drag_started_by(PointerButton::Primary)
                        && hovered_port.is_none()
                        && !shift_held
                    {
                        if let (Some(gate_index), Some(pointer_pos)) =
                            (hovered_gate_index, pointer_screen_pos)
                        {
                            let drag_offset = self.graph.nodes[gate_index].pos
                                - self.screen_to_canvas(pointer_pos, canvas_origin);
                            self.dragging_gate = Some((gate_index, drag_offset));
                        }
                    }
                }

                // ── Port click → single wire ───────────────────────────────
                if canvas_response.clicked() && !shift_held {
                    if let Some((clicked_port, is_output_port)) = hovered_port.clone() {
                        match self.pending_wire_start.take() {
                            None => {
                                if is_output_port {
                                    self.pending_wire_start = Some(clicked_port);
                                }
                            }
                            Some(wire_start) => {
                                if !is_output_port {
                                    // Remove any existing wire driving this input port.
                                    self.graph.wires.retain(|wire| {
                                        !(wire.to.node == clicked_port.node
                                            && wire.to.port == clicked_port.port)
                                    });
                                    self.graph.wires.push(Wire { from: wire_start, to: clicked_port });
                                } else {
                                    self.pending_wire_start = Some(clicked_port);
                                }
                            }
                        }
                    } else {
                        self.pending_wire_start = None;
                    }
                }

                // ── Right-click wire → delete ──────────────────────────────
                if canvas_response.secondary_clicked() {
                    if let Some(wire) = hovered_wire.clone() {
                        if hovered_port.is_none() && hovered_gate_index.is_none() {
                            self.graph.remove_wire(&wire);
                        }
                    }
                }

                // ── Delete/Backspace → remove hovered gate ─────────────────
                if ui.input(|input| {
                    input.key_pressed(Key::Delete) || input.key_pressed(Key::Backspace)
                }) {
                    if let Some(gate_index) = hovered_gate_index {
                        self.graph.remove_gate(gate_index);
                    }
                }

                // ── Right-click canvas → gate spawn menu ───────────────────
                let spawn_canvas_pos = pointer_screen_pos
                    .map(|pos| snap_to_grid(self.screen_to_canvas(pos, canvas_origin)))
                    .unwrap_or(Pos2::ZERO);

                // Only show the spawn menu when not hovering a wire or gate.
                if hovered_wire.is_none() && hovered_gate_index.is_none() {
                    canvas_response.context_menu(|ui| {
                        ui.label(RichText::new("Add Gate").strong().size(13.0));
                        ui.separator();

                        if ui.button("⊼  NAND  (2→1)").clicked() {
                            self.graph.nodes.push(make_nand_node(spawn_canvas_pos));
                            ui.close_menu();
                        }

                        if !self.library.is_empty() {
                            ui.separator();
                            ui.label(
                                RichText::new("Library")
                                    .color(COLOR_DIM)
                                    .italics()
                                    .size(11.0),
                            );
                            let mut gate_to_spawn: Option<usize> = None;
                            for (library_index, saved_gate) in self.library.iter().enumerate() {
                                let button_label = format!(
                                    "▣  {}  ({} → {})",
                                    saved_gate.name, saved_gate.input_count, saved_gate.output_count
                                );
                                if ui.button(button_label).clicked() {
                                    gate_to_spawn = Some(library_index);
                                    ui.close_menu();
                                }
                            }
                            if let Some(library_index) = gate_to_spawn {
                                let saved_gate = &self.library[library_index];
                                let input_labels: Vec<String>  = saved_gate.graph.inputs.clone();
                                let output_labels: Vec<String> = saved_gate.graph.outputs.clone();
                                self.graph.nodes.push(make_saved_gate_node(
                                    spawn_canvas_pos,
                                    library_index,
                                    saved_gate.name.clone(),
                                    input_labels,
                                    output_labels,
                                ));
                            }
                        }
                    });
                }

                // ── Draw ──────────────────────────────────────────────────
                self.draw_grid(&painter, canvas_rect);
                self.draw_io_rails(&painter, canvas_rect);
                self.draw_wires(&painter, canvas_origin, canvas_rect, &hovered_wire);

                // In-progress wire preview following the cursor.
                if let Some(wire_start_port) = &self.pending_wire_start.clone() {
                    let start_screen =
                        self.port_to_screen_pos(wire_start_port, true, canvas_origin, canvas_rect);
                    if let (Some(start_pos), Some(mouse_pos)) = (start_screen, pointer_screen_pos) {
                        draw_bezier_wire(&painter, start_pos, mouse_pos, COLOR_WIRE_PENDING, 2.0);
                    }
                }

                // Gate nodes.
                for gate_index in 0..self.graph.nodes.len() {
                    let is_hovered = hovered_gate_index == Some(gate_index)
                        && self.dragging_gate.map(|(dragged, _)| dragged) != Some(gate_index);
                    let is_dragging =
                        self.dragging_gate.map(|(dragged, _)| dragged) == Some(gate_index);
                    self.draw_gate_node(
                        gate_index,
                        &painter,
                        canvas_origin,
                        canvas_rect,
                        is_hovered,
                        is_dragging,
                    );
                }

                // Highlight ring on the port under the mouse.
                if let Some((hovered_port_ref, is_output_port)) = &hovered_port {
                    if let Some(pos) = self.port_to_screen_pos(
                        hovered_port_ref,
                        *is_output_port,
                        canvas_origin,
                        canvas_rect,
                    ) {
                        let highlight_color =
                            if *is_output_port { COLOR_PORT_OUTPUT } else { COLOR_PORT_INPUT };
                        painter.circle_stroke(
                            pos,
                            PORT_RADIUS * self.canvas_zoom * 2.0,
                            Stroke::new(2.0, highlight_color),
                        );
                    }
                }

                // Bulk-wire overlay (selection boxes and highlighted ports).
                self.draw_bulk_wire_overlay(&painter, canvas_origin, canvas_rect);
            });
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Bulk-wire state machine
    // ─────────────────────────────────────────────────────────────────────────

    /// Drive the bulk-wire state machine from the current canvas input state.
    pub fn update_bulk_wire(
        &mut self,
        canvas_response: &Response,
        pointer_screen_pos: Option<Pos2>,
        shift_held: bool,
        canvas_origin: Pos2,
        canvas_rect: Rect,
    ) {
        let pointer_canvas_pos = pointer_screen_pos
            .map(|pos| self.screen_to_canvas(pos, canvas_origin));

        match std::mem::take(&mut self.bulk_wire_state) {

            // ── Idle: start phase 1 on Shift+drag ─────────────────────────
            BulkWireState::Idle => {
                if shift_held && canvas_response.drag_started_by(PointerButton::Primary) {
                    if let Some(start) = pointer_canvas_pos {
                        self.bulk_wire_state = BulkWireState::SelectingOutputs {
                            drag_start_canvas:   start,
                            drag_current_canvas: start,
                        };
                    } else {
                        self.bulk_wire_state = BulkWireState::Idle;
                    }
                } else {
                    self.bulk_wire_state = BulkWireState::Idle;
                }
            }

            // ── Phase 1: user is dragging to select output ports ──────────
            BulkWireState::SelectingOutputs { drag_start_canvas, .. } => {
                if canvas_response.dragged_by(PointerButton::Primary) {
                    let current = pointer_canvas_pos.unwrap_or(drag_start_canvas);
                    self.bulk_wire_state = BulkWireState::SelectingOutputs {
                        drag_start_canvas,
                        drag_current_canvas: current,
                    };
                } else if canvas_response.drag_stopped() {
                    let current = pointer_canvas_pos.unwrap_or(drag_start_canvas);
                    let selection_rect = canvas_rect_from_two_points(drag_start_canvas, current);
                    let mut selected_output_ports =
                        self.collect_ports_in_canvas_rect(selection_rect, true, canvas_rect);

                    // Sort top-to-bottom so pairing with inputs is intuitive.
                    selected_output_ports.sort_by(|port_a, port_b| {
                        let a_y = self
                            .port_to_screen_pos(port_a, true, canvas_origin, canvas_rect)
                            .map(|p| p.y)
                            .unwrap_or(0.0);
                        let b_y = self
                            .port_to_screen_pos(port_b, true, canvas_origin, canvas_rect)
                            .map(|p| p.y)
                            .unwrap_or(0.0);
                        a_y.partial_cmp(&b_y).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    self.bulk_wire_state = if selected_output_ports.is_empty() {
                        BulkWireState::Idle
                    } else {
                        BulkWireState::OutputsChosen { selected_output_ports }
                    };
                } else {
                    self.bulk_wire_state = BulkWireState::Idle;
                }
            }

            // ── Waiting: outputs chosen, waiting for phase 2 Shift+drag ───
            BulkWireState::OutputsChosen { selected_output_ports } => {
                if shift_held && canvas_response.drag_started_by(PointerButton::Primary) {
                    if let Some(start) = pointer_canvas_pos {
                        self.bulk_wire_state = BulkWireState::SelectingInputs {
                            selected_output_ports,
                            drag_start_canvas:   start,
                            drag_current_canvas: start,
                        };
                    } else {
                        self.bulk_wire_state =
                            BulkWireState::OutputsChosen { selected_output_ports };
                    }
                } else if !shift_held && canvas_response.clicked() {
                    // Non-Shift click cancels the operation.
                    self.bulk_wire_state = BulkWireState::Idle;
                } else {
                    self.bulk_wire_state = BulkWireState::OutputsChosen { selected_output_ports };
                }
            }

            // ── Phase 2: user is dragging to select input ports ───────────
            BulkWireState::SelectingInputs {
                selected_output_ports,
                drag_start_canvas,
                ..
            } => {
                if canvas_response.dragged_by(PointerButton::Primary) {
                    let current = pointer_canvas_pos.unwrap_or(drag_start_canvas);
                    self.bulk_wire_state = BulkWireState::SelectingInputs {
                        selected_output_ports,
                        drag_start_canvas,
                        drag_current_canvas: current,
                    };
                } else if canvas_response.drag_stopped() {
                    let current = pointer_canvas_pos.unwrap_or(drag_start_canvas);
                    let selection_rect = canvas_rect_from_two_points(drag_start_canvas, current);
                    let mut selected_input_ports =
                        self.collect_ports_in_canvas_rect(selection_rect, false, canvas_rect);

                    selected_input_ports.sort_by(|port_a, port_b| {
                        let a_y = self
                            .port_to_screen_pos(port_a, false, canvas_origin, canvas_rect)
                            .map(|p| p.y)
                            .unwrap_or(0.0);
                        let b_y = self
                            .port_to_screen_pos(port_b, false, canvas_origin, canvas_rect)
                            .map(|p| p.y)
                            .unwrap_or(0.0);
                        a_y.partial_cmp(&b_y).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    // Pair outputs to inputs in top-to-bottom order.
                    let pair_count = selected_output_ports.len().min(selected_input_ports.len());
                    for pair_index in 0..pair_count {
                        let output_port = selected_output_ports[pair_index].clone();
                        let input_port  = selected_input_ports[pair_index].clone();
                        // Remove any existing wire driving this input port.
                        self.graph.wires.retain(|wire| {
                            !(wire.to.node == input_port.node && wire.to.port == input_port.port)
                        });
                        self.graph.wires.push(Wire { from: output_port, to: input_port });
                    }

                    self.bulk_wire_state = BulkWireState::Idle;
                } else {
                    self.bulk_wire_state = BulkWireState::Idle;
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Port collection for box-select
    // ─────────────────────────────────────────────────────────────────────────

    /// Return all ports of the requested kind (output or input) whose canvas-space
    /// position falls inside `canvas_rect_selection`.
    pub fn collect_ports_in_canvas_rect(
        &self,
        canvas_rect_selection: Rect,
        want_output_ports: bool,
        full_canvas_rect: Rect,
    ) -> Vec<PortRef> {
        let mut found_ports: Vec<PortRef> = Vec::new();
        let dummy_origin = full_canvas_rect.min;
        let input_count  = self.graph.inputs.len();
        let output_count = self.graph.outputs.len();
        let center_y     = full_canvas_rect.center().y;

        if want_output_ports {
            // Left rail input pseudo-nodes act as output sources.
            let start_y = center_y - (input_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
            for input_index in 0..input_count {
                let screen_pos = pos2(
                    full_canvas_rect.left() + 14.0,
                    start_y + input_index as f32 * IO_RAIL_STEP,
                );
                let canvas_pos = self.screen_to_canvas(screen_pos, dummy_origin);
                if canvas_rect_selection.contains(canvas_pos) {
                    found_ports.push(PortRef {
                        node: self.graph.input_node_id(input_index),
                        port: 0,
                    });
                }
            }
        } else {
            // Right rail output pseudo-nodes act as input sinks.
            let start_y = center_y - (output_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
            for output_index in 0..output_count {
                let screen_pos = pos2(
                    full_canvas_rect.right() - 14.0,
                    start_y + output_index as f32 * IO_RAIL_STEP,
                );
                let canvas_pos = self.screen_to_canvas(screen_pos, dummy_origin);
                if canvas_rect_selection.contains(canvas_pos) {
                    found_ports.push(PortRef {
                        node: self.graph.output_node_id(output_index),
                        port: 0,
                    });
                }
            }
        }

        // Internal gate nodes.
        for gate_index in 0..self.graph.nodes.len() {
            let node = &self.graph.nodes[gate_index];
            if want_output_ports {
                for port_index in 0..node.output_count {
                    let canvas_pos = output_port_canvas_pos(node, port_index);
                    if canvas_rect_selection.contains(canvas_pos) {
                        found_ports.push(PortRef {
                            node: self.graph.gate_node_id(gate_index),
                            port: port_index,
                        });
                    }
                }
            } else {
                for port_index in 0..node.input_count {
                    let canvas_pos = input_port_canvas_pos(node, port_index);
                    if canvas_rect_selection.contains(canvas_pos) {
                        found_ports.push(PortRef {
                            node: self.graph.gate_node_id(gate_index),
                            port: port_index,
                        });
                    }
                }
            }
        }

        found_ports
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Bulk-wire overlay drawing
    // ─────────────────────────────────────────────────────────────────────────

    /// Draw selection boxes and port-highlight rings for the current bulk-wire phase.
    pub fn draw_bulk_wire_overlay(&self, painter: &Painter, canvas_origin: Pos2, canvas_rect: Rect) {
        match &self.bulk_wire_state {
            BulkWireState::SelectingOutputs { drag_start_canvas, drag_current_canvas } => {
                let start_screen = self.canvas_to_screen(*drag_start_canvas, canvas_origin);
                let end_screen   = self.canvas_to_screen(*drag_current_canvas, canvas_origin);
                let screen_rect  = Rect::from_two_pos(start_screen, end_screen);
                painter.rect(
                    screen_rect,
                    Rounding::ZERO,
                    COLOR_BOX_SELECT,
                    Stroke::new(1.5, COLOR_BOX_SELECT_BORDER),
                );
                painter.text(
                    screen_rect.left_top() - vec2(0.0, 14.0),
                    Align2::LEFT_BOTTOM,
                    "Select OUTPUT ports",
                    FontId::proportional(11.0),
                    COLOR_BOX_SELECT_BORDER,
                );
            }

            BulkWireState::OutputsChosen { selected_output_ports } => {
                for port in selected_output_ports {
                    if let Some(screen_pos) =
                        self.port_to_screen_pos(port, true, canvas_origin, canvas_rect)
                    {
                        painter.circle_stroke(
                            screen_pos,
                            PORT_RADIUS * self.canvas_zoom * 2.5,
                            Stroke::new(2.5, COLOR_SIGNAL_HIGH),
                        );
                    }
                }
                painter.text(
                    canvas_rect.center_top() + vec2(0.0, 8.0),
                    Align2::CENTER_TOP,
                    format!(
                        "{} outputs selected — Shift+drag to pick inputs",
                        selected_output_ports.len()
                    ),
                    FontId::proportional(12.0),
                    COLOR_SIGNAL_HIGH,
                );
            }

            BulkWireState::SelectingInputs {
                selected_output_ports,
                drag_start_canvas,
                drag_current_canvas,
            } => {
                for port in selected_output_ports {
                    if let Some(screen_pos) =
                        self.port_to_screen_pos(port, true, canvas_origin, canvas_rect)
                    {
                        painter.circle_stroke(
                            screen_pos,
                            PORT_RADIUS * self.canvas_zoom * 2.5,
                            Stroke::new(2.5, COLOR_SIGNAL_HIGH),
                        );
                    }
                }
                let start_screen = self.canvas_to_screen(*drag_start_canvas, canvas_origin);
                let end_screen   = self.canvas_to_screen(*drag_current_canvas, canvas_origin);
                let screen_rect  = Rect::from_two_pos(start_screen, end_screen);
                painter.rect(
                    screen_rect,
                    Rounding::ZERO,
                    Color32::from_rgba_premultiplied(255, 120, 80, 25),
                    Stroke::new(1.5, Color32::from_rgb(255, 120, 80)),
                );
                painter.text(
                    screen_rect.left_top() - vec2(0.0, 14.0),
                    Align2::LEFT_BOTTOM,
                    "Select INPUT ports",
                    FontId::proportional(11.0),
                    Color32::from_rgb(255, 140, 80),
                );
            }

            BulkWireState::Idle => {}
        }
    }
}