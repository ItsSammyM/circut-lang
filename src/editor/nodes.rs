use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Stroke, pos2, vec2};

use super::app::App;
use super::constants::{
    COLOR_NODE_FILL, COLOR_NODE_HOVERED, COLOR_NODE_STROKE,
    COLOR_PORT_INPUT, COLOR_PORT_OUTPUT, COLOR_SIGNAL_HIGH, COLOR_SIGNAL_LOW,
    COLOR_TEXT, IO_RAIL_STEP, NODE_WIDTH, PORT_RADIUS, PORT_TOP_PADDING, PORT_VERTICAL_STEP,
    FONT_SIZE_IO_RAIL_LABEL, FONT_SIZE_NODE_LABEL, FONT_SIZE_NODE_PORT_LABEL,
};
use super::graph::{EditorNode, EditorNodeKind};

impl App {
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
                FontId::proportional(FONT_SIZE_IO_RAIL_LABEL),
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
                FontId::proportional(FONT_SIZE_IO_RAIL_LABEL),
                COLOR_TEXT,
            );
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
            FontId::proportional(FONT_SIZE_NODE_LABEL * self.canvas_zoom),
            COLOR_TEXT,
        );

        let node_id = self.graph.gate_node_id(gate_index);

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

            let label = node.input_labels.get(port_index).map(|s| s.as_str()).unwrap_or("");
            if !label.is_empty() {
                painter.text(
                    screen_pos + vec2(PORT_RADIUS * self.canvas_zoom + 3.0, 0.0),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(FONT_SIZE_NODE_PORT_LABEL * self.canvas_zoom),
                    Color32::from_rgb(160, 180, 220),
                );
            }
        }

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

            let label = node.output_labels.get(port_index).map(|s| s.as_str()).unwrap_or("");
            if !label.is_empty() {
                painter.text(
                    screen_pos - vec2(PORT_RADIUS * self.canvas_zoom + 3.0, 0.0),
                    Align2::RIGHT_CENTER,
                    label,
                    FontId::proportional(FONT_SIZE_NODE_PORT_LABEL * self.canvas_zoom),
                    Color32::from_rgb(160, 180, 220),
                );
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Signal-aware port colors
    // ─────────────────────────────────────────────────────────────────────────

    pub fn live_port_color_for_input(&self, node_id: usize, port_index: usize) -> Color32 {
        if self.simulation.is_none() {
            return COLOR_PORT_INPUT;
        }
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

// ─────────────────────────────────────────────────────────────────────────────
//  Node geometry & Factory helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the pixel height of a node box given its port counts.
pub fn compute_node_height(input_count: usize, output_count: usize) -> f32 {
    PORT_TOP_PADDING + PORT_VERTICAL_STEP * (input_count.max(output_count).max(1) as f32) + 10.0
}

/// Canvas-space position of the dot for input port `port_index` on `node`.
pub fn input_port_canvas_pos(node: &EditorNode, port_index: usize) -> Pos2 {
    Pos2::new(
        node.pos.x,
        node.pos.y + PORT_TOP_PADDING + port_index as f32 * PORT_VERTICAL_STEP,
    )
}

/// Canvas-space position of the dot for output port `port_index` on `node`.
pub fn output_port_canvas_pos(node: &EditorNode, port_index: usize) -> Pos2 {
    Pos2::new(
        node.pos.x + NODE_WIDTH,
        node.pos.y + PORT_TOP_PADDING + port_index as f32 * PORT_VERTICAL_STEP,
    )
}

/// Construct a default NAND gate node at the given canvas position.
pub fn make_nand_node(pos: Pos2) -> EditorNode {
    EditorNode {
        label: "NAND".into(),
        pos,
        input_count: 2,
        output_count: 1,
        kind: EditorNodeKind::Nand,
        input_labels: vec!["A".into(), "B".into()],
        output_labels: vec!["Q".into()],
    }
}

/// Construct an `EditorNode` for a library gate instance at the given canvas position.
pub fn make_saved_gate_node(
    pos: Pos2,
    library_index: usize,
    gate_label: String,
    input_labels: Vec<String>,
    output_labels: Vec<String>,
) -> EditorNode {
    let input_count  = input_labels.len();
    let output_count = output_labels.len();
    EditorNode {
        label: gate_label,
        pos,
        input_count,
        output_count,
        kind: EditorNodeKind::SavedGate(library_index),
        input_labels,
        output_labels,
    }
}