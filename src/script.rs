use std::collections::HashMap;

use bincode::Options;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CircutLangScript{
    pub entry_point: String,
    pub gates: HashMap<String, GraphDesc>
}
impl CircutLangScript{
    pub fn bincode_options()-> impl bincode::config::Options {
        bincode::config::DefaultOptions::new()
            .with_limit(10 * 1024 * 1024)
    }
}


/// Identifies one port on one node using the flat node-id numbering that
/// `EditorGraph` uses:
///   `0 .. n_inputs`                      → input pseudo-nodes  (port 0 is their one output)
///   `n_inputs .. n_inputs + n_outputs`   → output pseudo-nodes (port 0 is their one input)
///   `n_inputs + n_outputs ..`            → internal gate nodes
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PortRef {
    pub node: usize,
    pub port: usize,
}

/// Describes one wire: a directed edge from an output port to an input port.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WireDesc {
    pub from: PortRef, // output port that drives the wire
    pub to:   PortRef, // input port that is driven by the wire
}


/// The kind of an internal gate node inside a `GraphDesc`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GateKind {
    Nand,
    /// Name of the gate in the library, identifying which saved gate this instance represents.
    SavedGate(String),
    ExternalGate(String),
}

/// A complete description of one level of the circuit as seen by the editor.
///
/// Node-id conventions must match `EditorGraph`:
///   `input_base  = 0`
///   `output_base = n_inputs`
///   `gate_base   = n_inputs + n_outputs`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphDesc {
    pub n_inputs:  usize,
    pub n_outputs: usize,
    /// One entry per internal gate: `(gate_input_count, gate_output_count, kind)`.
    pub gates: Vec<(usize, usize, GateKind)>,
    pub wires: Vec<WireDesc>,
}
impl GraphDesc{
    pub const fn input_base()->usize{
        0
    }
    pub const fn output_base(&self)->usize{
        self.n_inputs
    }
    pub const fn gate_base(&self)->usize{
        self.n_inputs + self.n_outputs
    }
}