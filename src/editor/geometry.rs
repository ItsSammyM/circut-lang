use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2, vec2};

use super::constants::{
    GRID_CELL_SIZE, NODE_WIDTH, PORT_TOP_PADDING, PORT_VERTICAL_STEP,
};
use super::graph::{EditorNode, EditorNodeKind};

// ─────────────────────────────────────────────────────────────────────────────
//  Node geometry
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

// ─────────────────────────────────────────────────────────────────────────────
//  Grid helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Snap a canvas-space position to the nearest grid corner.
pub fn snap_to_grid(pos: Pos2) -> Pos2 {
    Pos2::new(
        (pos.x / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
        (pos.y / GRID_CELL_SIZE).round() * GRID_CELL_SIZE,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
//  Rect construction
// ─────────────────────────────────────────────────────────────────────────────

/// Build a canvas-space `Rect` from any two opposite corners (order does not matter).
pub fn canvas_rect_from_two_points(corner_a: Pos2, corner_b: Pos2) -> Rect {
    Rect::from_min_max(
        Pos2::new(corner_a.x.min(corner_b.x), corner_a.y.min(corner_b.y)),
        Pos2::new(corner_a.x.max(corner_b.x), corner_a.y.max(corner_b.y)),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
//  Factory helpers
// ─────────────────────────────────────────────────────────────────────────────

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
/// Port labels are taken directly from the library gate's own input/output names.
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

// ─────────────────────────────────────────────────────────────────────────────
//  Bezier wire rendering
// ─────────────────────────────────────────────────────────────────────────────

/// Draw a smooth cubic Bezier wire between two screen-space points.
///
/// The control points are placed horizontally from each endpoint so the wire
/// curves nicely between gates laid out left-to-right.
pub fn draw_bezier_wire(painter: &Painter, from: Pos2, to: Pos2, color: Color32, line_width: f32) {
    let horizontal_ctrl_offset = ((to.x - from.x).abs() * 0.45).max(60.0);
    let ctrl1 = pos2(from.x + horizontal_ctrl_offset, from.y);
    let ctrl2 = pos2(to.x   - horizontal_ctrl_offset, to.y);
    let mut prev_point = from;
    for step in 1..=32usize {
        let t = step as f32 / 32.0;
        let u = 1.0 - t;
        let next_point = pos2(
            u*u*u*from.x + 3.0*u*u*t*ctrl1.x + 3.0*u*t*t*ctrl2.x + t*t*t*to.x,
            u*u*u*from.y + 3.0*u*u*t*ctrl1.y + 3.0*u*t*t*ctrl2.y + t*t*t*to.y,
        );
        painter.line_segment([prev_point, next_point], Stroke::new(line_width, color));
        prev_point = next_point;
    }
}