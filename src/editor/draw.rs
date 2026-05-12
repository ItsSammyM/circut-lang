use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Stroke, pos2, vec2};

use super::app::App;
use super::constants::{
    COLOR_GRID, COLOR_NODE_FILL, COLOR_NODE_HOVERED, COLOR_NODE_STROKE,
    COLOR_PORT_INPUT, COLOR_PORT_OUTPUT, COLOR_SIGNAL_HIGH, COLOR_SIGNAL_LOW,
    COLOR_TEXT, COLOR_WIRE, COLOR_WIRE_HIGH, COLOR_WIRE_LOW,
    GRID_CELL_SIZE, IO_RAIL_STEP, NODE_WIDTH, PORT_RADIUS,
};
use super::geometry::{
    compute_node_height, draw_bezier_wire, input_port_canvas_pos, output_port_canvas_pos,
};
use super::graph::Wire;

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Background grid
    // ─────────────────────────────────────────────────────────────────────────

    pub fn draw_grid(&self, painter: &Painter, rect: Rect) {
        let grid_pixel_size = GRID_CELL_SIZE * self.canvas_zoom;
        let pan_remainder_x =
            (self.canvas_pan.x * self.canvas_zoom).rem_euclid(grid_pixel_size);
        let pan_remainder_y =
            (self.canvas_pan.y * self.canvas_zoom).rem_euclid(grid_pixel_size);
        let stroke = Stroke::new(0.5, COLOR_GRID);

        let mut x = rect.left() + pan_remainder_x;
        while x <= rect.right() {
            painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
            x += grid_pixel_size;
        }
        let mut y = rect.top() + pan_remainder_y;
        while y <= rect.bottom() {
            painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
            y += grid_pixel_size;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  I/O rails
    // ─────────────────────────────────────────────────────────────────────────

    /// Draw the left-rail input ports and right-rail output ports.
    pub fn draw_io_rails(&self, painter: &Painter, rect: Rect) {
        let center_y     = rect.center().y;
        let input_count  = self.graph.inputs.len();
        let output_count = self.graph.outputs.len();

        let inputs_start_y = center_y - (input_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
        for (input_index, input_name) in self.graph.inputs.iter().enumerate() {
            let screen_pos = pos2(
                rect.left() + 14.0,
                inputs_start_y + input_index as f32 * IO_RAIL_STEP,
            );
            let is_on        = self.input_states.get(input_index).copied().unwrap_or(false);
            let signal_color = if is_on { COLOR_SIGNAL_HIGH } else { COLOR_SIGNAL_LOW };
            painter.circle_filled(screen_pos, PORT_RADIUS + 3.0, signal_color);
            painter.circle_stroke(screen_pos, PORT_RADIUS + 3.0, Stroke::new(1.5, COLOR_PORT_INPUT));
            painter.text(
                screen_pos + vec2(14.0, 0.0),
                Align2::LEFT_CENTER,
                input_name,
                FontId::proportional(12.0),
                COLOR_TEXT,
            );
        }

        let outputs_start_y = center_y - (output_count as f32 - 1.0) * IO_RAIL_STEP / 2.0;
        for (output_index, output_name) in self.graph.outputs.iter().enumerate() {
            let screen_pos = pos2(
                rect.right() - 14.0,
                outputs_start_y + output_index as f32 * IO_RAIL_STEP,
            );
            let is_on        = self.output_states.get(output_index).copied().unwrap_or(false);
            let signal_color = if is_on { COLOR_SIGNAL_HIGH } else { COLOR_SIGNAL_LOW };
            painter.circle_filled(screen_pos, PORT_RADIUS + 3.0, signal_color);
            painter.circle_stroke(screen_pos, PORT_RADIUS + 3.0, Stroke::new(1.5, COLOR_PORT_OUTPUT));
            painter.text(
                screen_pos - vec2(14.0, 0.0),
                Align2::RIGHT_CENTER,
                output_name,
                FontId::proportional(12.0),
                COLOR_TEXT,
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Wires
    // ─────────────────────────────────────────────────────────────────────────

    pub fn draw_wires(
        &self,
        painter: &Painter,
        canvas_origin: Pos2,
        canvas_rect: Rect,
        hovered_wire: &Option<Wire>,
    ) {
        for wire in &self.graph.wires {
            let from_screen = self.port_to_screen_pos(&wire.from, true,  canvas_origin, canvas_rect);
            let to_screen   = self.port_to_screen_pos(&wire.to,   false, canvas_origin, canvas_rect);

            if let (Some(from_pos), Some(to_pos)) = (from_screen, to_screen) {
                let wire_color = if self.simulation.is_some() {
                    let wire_index = self.port_to_wire_index
                        .get(&(wire.from.node, wire.from.port, true))
                        .copied();
                    match wire_index {
                        Some(idx) if self.live_wire_signals.get(&idx).copied().unwrap_or(false) => {
                            COLOR_WIRE_HIGH
                        }
                        Some(_) => COLOR_WIRE_LOW,
                        None    => COLOR_WIRE,
                    }
                } else {
                    COLOR_WIRE
                };

                let is_hovered = hovered_wire.as_ref() == Some(wire);
                let line_width = if is_hovered { 4.5 } else { 2.5 };
                let color      = if is_hovered { Color32::from_rgb(255, 80, 80) } else { wire_color };
                draw_bezier_wire(painter, from_pos, to_pos, color, line_width);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Gate nodes
    // ─────────────────────────────────────────────────────────────────────────

    pub fn draw_gate_node(
        &self,
        gate_index: usize,
        painter: &Painter,
        canvas_origin: Pos2,
        canvas_rect: Rect,
        is_hovered: bool,
        is_dragging: bool,
    ) {
        let node        = &self.graph.nodes[gate_index];
        let node_height = compute_node_height(node.input_count, node.output_count);
        let top_left_screen = self.canvas_to_screen(node.pos, canvas_origin);
        let node_rect = Rect::from_min_size(
            top_left_screen,
            vec2(NODE_WIDTH * self.canvas_zoom, node_height * self.canvas_zoom),
        );

        let fill_color = if is_dragging {
            Color32::from_rgb(68, 74, 110)
        } else if is_hovered {
            COLOR_NODE_HOVERED
        } else {
            COLOR_NODE_FILL
        };
        let border_color = if is_dragging || is_hovered { Color32::WHITE } else { COLOR_NODE_STROKE };

        painter.rect(
            node_rect,
            Rounding::same(5.0 * self.canvas_zoom),
            fill_color,
            Stroke::new(1.5, border_color),
        );
        painter.text(
            node_rect.center_top() + vec2(0.0, 10.0 * self.canvas_zoom),
            Align2::CENTER_CENTER,
            &node.label,
            FontId::proportional(11.0 * self.canvas_zoom),
            COLOR_TEXT,
        );

        let node_id = self.graph.gate_node_id(gate_index);

        // ── Input port dots and labels ────────────────────────────────────────

        for port_index in 0..node.input_count {
            let canvas_pos = input_port_canvas_pos(node, port_index);
            let screen_pos = self.canvas_to_screen(canvas_pos, canvas_origin);
            let port_color = self.live_port_color_for_input(node_id, port_index);
            painter.circle_filled(screen_pos, PORT_RADIUS * self.canvas_zoom, port_color);
            painter.circle_stroke(
                screen_pos,
                PORT_RADIUS * self.canvas_zoom,
                Stroke::new(1.0, Color32::WHITE),
            );

            // Port label — drawn just inside the node body to the right of the dot.
            let label = node
                .input_labels
                .get(port_index)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !label.is_empty() {
                painter.text(
                    screen_pos + vec2(PORT_RADIUS * self.canvas_zoom + 3.0, 0.0),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(9.0 * self.canvas_zoom),
                    Color32::from_rgb(160, 180, 220),
                );
            }
        }

        // ── Output port dots and labels ───────────────────────────────────────

        for port_index in 0..node.output_count {
            let canvas_pos = output_port_canvas_pos(node, port_index);
            let screen_pos = self.canvas_to_screen(canvas_pos, canvas_origin);
            let port_color = self.live_port_color_for_output(node_id, port_index);
            painter.circle_filled(screen_pos, PORT_RADIUS * self.canvas_zoom, port_color);
            painter.circle_stroke(
                screen_pos,
                PORT_RADIUS * self.canvas_zoom,
                Stroke::new(1.0, Color32::WHITE),
            );

            // Port label — drawn just inside the node body to the left of the dot.
            let label = node
                .output_labels
                .get(port_index)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !label.is_empty() {
                painter.text(
                    screen_pos - vec2(PORT_RADIUS * self.canvas_zoom + 3.0, 0.0),
                    Align2::RIGHT_CENTER,
                    label,
                    FontId::proportional(9.0 * self.canvas_zoom),
                    Color32::from_rgb(160, 180, 220),
                );
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Signal-aware port colors
    // ─────────────────────────────────────────────────────────────────────────

    /// Return the display color for an input port, reflecting the live signal on
    /// the wire that drives it (if a simulation is running).
    pub fn live_port_color_for_input(&self, node_id: usize, port_index: usize) -> Color32 {
        if self.simulation.is_none() {
            return COLOR_PORT_INPUT;
        }
        // Find the wire that drives this input port.
        let driving_wire = self.graph.wires.iter().find(|wire| {
            wire.to.node == node_id && wire.to.port == port_index
        });
        let Some(wire) = driving_wire else { return COLOR_PORT_INPUT };
        let Some(&wire_idx) = self.port_to_wire_index.get(&(wire.from.node, wire.from.port, true))
        else {
            return COLOR_PORT_INPUT;
        };
        if self.live_wire_signals.get(&wire_idx).copied().unwrap_or(false) {
            COLOR_SIGNAL_HIGH
        } else {
            COLOR_SIGNAL_LOW
        }
    }

    /// Return the display color for an output port, reflecting its live signal value.
    pub fn live_port_color_for_output(&self, node_id: usize, port_index: usize) -> Color32 {
        if self.simulation.is_none() {
            return COLOR_PORT_OUTPUT;
        }
        let Some(&wire_idx) = self.port_to_wire_index.get(&(node_id, port_index, true)) else {
            return COLOR_PORT_OUTPUT;
        };
        if self.live_wire_signals.get(&wire_idx).copied().unwrap_or(false) {
            COLOR_SIGNAL_HIGH
        } else {
            COLOR_SIGNAL_LOW
        }
    }
}