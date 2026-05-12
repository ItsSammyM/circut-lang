use std::collections::HashMap;

use egui::Vec2;

use crate::sim_builder::PortRef;
use crate::simulation::simulation::Simulation;

use super::graph::{BulkWireState, EditorGraph, LibraryGate};

// ─────────────────────────────────────────────────────────────────────────────
//  App
// ─────────────────────────────────────────────────────────────────────────────

pub struct App {
    /// The name shown in the top bar and used when saving to the library.
    pub title: String,
    /// The circuit currently being edited on the canvas.
    pub graph: EditorGraph,
    /// All gates the user has saved for reuse.
    pub library: Vec<LibraryGate>,

    /// The compiled, running simulation — `None` if not yet built or build failed.
    pub simulation: Option<Simulation>,
    /// wire_index → current signal value, snapshotted each tick from the simulation.
    pub live_wire_signals: HashMap<u32, bool>,
    /// Maps (node_id, port_index, is_output) → wire_index in the simulation.
    /// Built alongside the simulation so draw code can look up signal state without
    /// touching simulation internals directly.
    pub port_to_wire_index: HashMap<(usize, usize, bool), u32>,
    /// Current logical state of each external input port (true = high).
    pub input_states: Vec<bool>,
    /// Most recent logical state of each external output port (true = high).
    pub output_states: Vec<bool>,
    /// Whether the simulation is continuously stepping each frame.
    pub simulation_running: bool,
    /// Error message from the last simulation build attempt, if any.
    pub simulation_error: Option<String>,

    /// Pan offset of the canvas viewport in canvas-space units.
    pub canvas_pan: Vec2,
    /// Zoom level (screen pixels per canvas unit).
    pub canvas_zoom: f32,

    /// The output port the user clicked to start drawing a single wire from.
    pub pending_wire_start: Option<PortRef>,
    /// Index of the gate being dragged, plus the offset from its top-left corner to the click point.
    pub dragging_gate: Option<(usize, Vec2)>,

    /// State machine for the box-select bulk-wire feature (Shift+drag).
    pub bulk_wire_state: BulkWireState,

    /// Text typed into the "add input" field in the left panel.
    pub new_input_name: String,
    /// Text typed into the "add output" field in the right panel.
    pub new_output_name: String,

    /// Library gate currently showing its right-click rename field.
    pub library_rename_index: Option<usize>,
    /// Text being typed for the in-progress rename.
    pub library_rename_text: String,

    /// Drag-and-drop reorder state for the input port list in the left panel.
    /// Contains `(dragged_input_index, current_hover_target_index)` while a drag is live.
    pub input_drag_reorder: Option<(usize, usize)>,
    /// Drag-and-drop reorder state for the output port list in the right panel.
    /// Contains `(dragged_output_index, current_hover_target_index)` while a drag is live.
    pub output_drag_reorder: Option<(usize, usize)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "My Gate".into(),
            graph: EditorGraph::default(),
            library: vec![],
            simulation: None,
            live_wire_signals: HashMap::new(),
            port_to_wire_index: HashMap::new(),
            input_states: vec![false],
            output_states: vec![false],
            simulation_running: false,
            simulation_error: None,
            canvas_pan: Vec2::ZERO,
            canvas_zoom: 1.0,
            pending_wire_start: None,
            dragging_gate: None,
            bulk_wire_state: BulkWireState::Idle,
            new_input_name: String::new(),
            new_output_name: String::new(),
            library_rename_index: None,
            library_rename_text: String::new(),
            input_drag_reorder: None,
            output_drag_reorder: None,
        }
    }
}