use std::collections::HashMap;

use super::app::App;
use super::graph::{BulkWireState, EditorGraph, EditorNodeKind, LibraryGate};

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    //  Library persistence
    // ─────────────────────────────────────────────────────────────────────────

    /// Save the current canvas as a `LibraryGate`.
    ///
    /// If a gate with the same name already exists it is overwritten in place and
    /// every existing instance of that gate (in the active canvas and in all other
    /// library graphs) has its label, port counts, and port labels refreshed.
    /// If the port counts changed, wires connected to ports that no longer exist
    /// are dropped — the same contract as a port removal.
    pub fn save_current_graph_to_library(&mut self) {
        let new_gate = LibraryGate {
            name:         self.title.clone(),
            input_count:  self.graph.inputs.len(),
            output_count: self.graph.outputs.len(),
            graph:        self.graph.clone(),
        };

        if let Some(library_index) = self.library.iter().position(|saved| saved.name == new_gate.name) {
            // Overwrite the library entry.
            self.library[library_index] = new_gate.clone();

            // Update every existing instance of this gate across all graphs.
            Self::update_saved_gate_instances_in_graph(
                &mut self.graph,
                library_index,
                &new_gate,
            );
            // We cannot iterate self.library mutably while also calling the function
            // because the function takes a mutable reference to the graph.
            // So we pull the library out temporarily and put it back.
            let mut library = std::mem::take(&mut self.library);
            for library_gate in &mut library {
                Self::update_saved_gate_instances_in_graph(
                    &mut library_gate.graph,
                    library_index,
                    &new_gate,
                );
            }
            self.library = library;
        } else {
            self.library.push(new_gate);
        }
    }

    /// Refresh every `SavedGate(library_index)` node inside `graph` to match
    /// the current `updated_library_gate` definition.
    ///
    /// - The node's `label`, `input_count`, `output_count`, `input_labels`, and
    ///   `output_labels` are updated.
    /// - Any wire connected to a port that no longer exists is dropped (same
    ///   semantics as removing those ports one by one, but done in bulk here for
    ///   efficiency without the intermediate node-id renumbering).
    pub fn update_saved_gate_instances_in_graph(
        graph: &mut EditorGraph,
        library_index: usize,
        updated_library_gate: &LibraryGate,
    ) {
        let new_input_count  = updated_library_gate.input_count;
        let new_output_count = updated_library_gate.output_count;
        let new_input_labels: Vec<String>  = updated_library_gate.graph.inputs.clone();
        let new_output_labels: Vec<String> = updated_library_gate.graph.outputs.clone();
        let gate_base = graph.inputs.len() + graph.outputs.len();

        // ── Pass 1: collect the flat node-ids of every matching gate instance ──
        // We need these for wire cleanup, which needs a separate mutable borrow of
        // `graph.wires` that cannot overlap with a borrow of `graph.nodes`.
        let matching_node_ids: Vec<usize> = graph
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(gate_index, node)| {
                if matches!(node.kind, EditorNodeKind::SavedGate(idx) if idx == library_index) {
                    Some(gate_base + gate_index)
                } else {
                    None
                }
            })
            .collect();

        // ── Pass 2: drop wires connected to ports that no longer exist ─────────
        graph.wires.retain(|wire| {
            for &node_id in &matching_node_ids {
                if wire.to.node == node_id && wire.to.port >= new_input_count {
                    return false;
                }
                if wire.from.node == node_id && wire.from.port >= new_output_count {
                    return false;
                }
            }
            true
        });

        // ── Pass 3: update node metadata ──────────────────────────────────────
        for node in graph.nodes.iter_mut() {
            if !matches!(node.kind, EditorNodeKind::SavedGate(idx) if idx == library_index) {
                continue;
            }
            node.label         = updated_library_gate.name.clone();
            node.input_count   = new_input_count;
            node.output_count  = new_output_count;
            node.input_labels  = new_input_labels.clone();
            node.output_labels = new_output_labels.clone();
        }
    }

    pub fn save_library_to_file(&mut self) {
        fn fallible_save(library: &Vec<LibraryGate>) -> Result<(), &'static str> {
            bincode::serialize_into(
                std::fs::File::create("my_library.logic_builder_lib")
                    .map_err(|_| "failed to create or open file to save library")?,
                &library,
            )
            .map_err(|_| "failed to serialize library for saving")
        }
        if let Err(error) = fallible_save(&self.library) {
            self.simulation_error = Some(error.to_string());
        }
    }

    pub fn load_library_from_file(&mut self) {
        fn fallible_load() -> Result<Vec<LibraryGate>, &'static str> {
            bincode::deserialize_from(
                std::fs::File::open("my_library.logic_builder_lib")
                    .map_err(|_| "failed to open file to load library")?,
            )
            .map_err(|_| "failed to deserialize library on load")
        }
        match fallible_load() {
            Ok(library) => self.library = library,
            Err(error)  => self.simulation_error = Some(error.to_string()),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Library gate management
    // ─────────────────────────────────────────────────────────────────────────

    /// Load a saved gate back onto the canvas for editing.
    pub fn open_library_gate_for_editing(&mut self, library_index: usize) {
        let gate = self.library[library_index].clone();
        self.title              = gate.name;
        self.graph              = gate.graph;
        self.input_states       = vec![false; gate.input_count];
        self.output_states      = vec![false; gate.output_count];
        self.simulation         = None;
        self.simulation_error   = None;
        self.simulation_running = false;
        self.pending_wire_start = None;
        self.dragging_gate      = None;
        self.live_wire_signals  = HashMap::new();
        self.port_to_wire_index = HashMap::new();
    }

    /// Delete a library gate by index.
    ///
    /// Every `SavedGate(deleted_index)` node in all graphs (active canvas and all
    /// library graphs) is removed along with its connected wires, exactly as if the
    /// user had deleted those gate nodes individually.
    ///
    /// Remaining `SavedGate` references with an index greater than `deleted_index`
    /// are decremented by one to stay consistent with the new library layout.
    pub fn delete_library_gate(&mut self, deleted_index: usize) {
        // Step 1: remove every instance of the deleted gate from the active canvas.
        self.graph.remove_all_gates_of_library_index(deleted_index);

        // Step 2: remove every instance from every library graph.
        // We must take the library out of self to avoid a borrow conflict.
        let mut library = std::mem::take(&mut self.library);
        for library_gate in &mut library {
            library_gate.graph.remove_all_gates_of_library_index(deleted_index);
        }
        self.library = library;

        // Step 3: actually remove the library entry.
        self.library.remove(deleted_index);

        // Step 4: remap all remaining SavedGate indices that are above deleted_index.
        self.graph.remap_saved_gate_indices_after_library_deletion(deleted_index);
        let mut library = std::mem::take(&mut self.library);
        for library_gate in &mut library {
            library_gate.graph.remap_saved_gate_indices_after_library_deletion(deleted_index);
        }
        self.library = library;

        // Clear rename state — it may now point at a stale index.
        self.library_rename_index = None;
        self.library_rename_text.clear();
    }

    // ─────────────────────────────────────────────────────────────────────────
    //  Canvas
    // ─────────────────────────────────────────────────────────────────────────

    /// Reset the canvas to a blank single-input / single-output graph.
    pub fn clear_canvas(&mut self) {
        self.graph              = EditorGraph::default();
        self.simulation         = None;
        self.simulation_error   = None;
        self.simulation_running = false;
        self.input_states       = vec![false];
        self.output_states      = vec![false];
        self.pending_wire_start = None;
        self.dragging_gate      = None;
        self.live_wire_signals  = HashMap::new();
        self.port_to_wire_index = HashMap::new();
        self.bulk_wire_state    = BulkWireState::Idle;
    }
}