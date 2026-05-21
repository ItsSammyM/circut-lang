use std::collections::{HashMap, HashSet};

use crate::{
    external_node_descriptions::ExternalNodeDescrptions, script::{CircutLangScript, GateKind, GraphDesc}, simulation::{
        WireId, node::{Node, NodeKind}, simulation::{Nodes, Simulation}, wire_state::WireState
    }
};


#[derive(Debug)]
pub enum CompileError{
    MissingEntryPoint(String),
    MissingLibraryNode(String),
    
    ExternalNodeBadIOCount(String),
    LibraryNodeBadIOCount(String),
    NandBadIOCount(String),

    CircularDependency(String),
}

pub struct Compiler{
    script: CircutLangScript,

    external_node_descriptions: Option<ExternalNodeDescrptions>,

    compilation_finished: HashMap<String, Simulation>,
    // gate is in started & gate is NOT in finished -> compilation has started & not finished
    // a compilation has started and not finished -> it is currently compiling
    // we go to compile it while it is currently compiling -> circular dep 
    compilation_started: HashSet<String>
}


impl Compiler{
    pub fn compile(script: CircutLangScript, external_node_descriptions: Option<ExternalNodeDescrptions>) -> Result<Simulation, CompileError> {
        let entry = &script.entry_point.clone();
        let mut compiler = Compiler{
            script,
            external_node_descriptions,
            compilation_finished: HashMap::new(),
            compilation_started: HashSet::new()
        };
        
        compiler.compile_or_get_graph(entry)
    }

    fn compile_or_get_graph(&mut self, name: &String) -> Result<Simulation, CompileError> {
        if let Some(sim) = self.compilation_finished.get(name) {
            return Ok(sim.clone());
        }

        
        if !self.compilation_started.insert(name.to_string()) {
            return Err(CompileError::CircularDependency(name.to_string()))
        }
        let graph = self.compile_graph(name)?;

        self.compilation_finished.insert(name.clone(), graph.clone());
        Ok(graph)
    }

    pub fn compile_graph(
        &mut self,
        name: &String,
    ) -> Result<Simulation, CompileError> {

        // ── Wire-id assignment ────────────────────────────────────────────────────
        //
        // One unique wire id is assigned to each *output port* in the graph.
        // Input ports do not produce their own wire ids — they simply read from
        // whatever wire id drives them (looked up via the wires list).
        //
        // Layout:
        //   wire ids 0 .. n_inputs-1       → the output port of each input pseudo-node
        //   wire ids n_inputs ..            → one id per output port of each internal gate
        //
        // `wire_id_of_output_port[node_id][port_index]` gives the wire id, or None
        // if that (node, port) pair has no output port (output pseudo-nodes).


        let desc = self.script
            .gates
            .get(name)
            .ok_or(CompileError::MissingEntryPoint(format!("Missing: {}", self.script.entry_point)))?
            .clone();


        let total_node_count = desc.n_inputs + desc.n_outputs + desc.gates.len();
        let mut wire_id_of_output_port: Vec<Vec<Option<u32>>> = vec![vec![]; total_node_count];
        let mut next_free_wire_id: u32 = 0;

        // Assign one wire id per input pseudo-node output.
        for input_index in 0..desc.n_inputs {
            let node_id = GraphDesc::input_base() + input_index;
            wire_id_of_output_port[node_id] = vec![Some(next_free_wire_id)];
            next_free_wire_id += 1;
        }

        // Output pseudo-nodes are sinks; they produce no wire ids.
        for output_index in 0..desc.n_outputs {
            let node_id = desc.output_base() + output_index;
            wire_id_of_output_port[node_id] = vec![];
        }

        // Assign wire ids to each output port of every internal gate.
        for (gate_slot, (_, gate_output_count, _)) in desc.gates.iter().enumerate() {
            let node_id = desc.gate_base() + gate_slot;
            let mut port_ids = Vec::with_capacity(*gate_output_count);
            for _ in 0..*gate_output_count {
                port_ids.push(Some(next_free_wire_id));
                next_free_wire_id += 1;
            }
            wire_id_of_output_port[node_id] = port_ids;
        }

        let total_wire_count = next_free_wire_id;

        // ── Helper: find the wire id that drives a given input port ──────────────
        //
        // Scans the wires list for a wire whose `.to` matches (node_id, port_index).
        // Returns wire id 0 (always false at start) for unconnected inputs.
        let find_driving_wire_id = |target_node_id: usize, target_port_index: usize| -> u32 {
            desc.wires
                .iter()
                .find_map(|wire| {
                    if wire.to.node == target_node_id && wire.to.port == target_port_index {
                        wire_id_of_output_port
                            .get(wire.from.node)
                            .and_then(|port_ids| port_ids.get(wire.from.port))
                            .and_then(|maybe_id| *maybe_id)
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        };

        // ── Build simulation nodes ────────────────────────────────────────────────

        let mut sim_nodes: Vec<Node> = Vec::with_capacity(desc.gates.len());

        for (gate_slot, (gate_input_count, gate_output_count, gate_kind)) in
            desc.gates.iter().enumerate()
        {
            let node_id = desc.gate_base() + gate_slot;

            match gate_kind {
                GateKind::Nand => {
                    if *gate_input_count < 2 || *gate_output_count < 1 {
                        return Err(CompileError::NandBadIOCount(format!("Inside: {}", name)));
                    }
                    let wire_a   = WireId::new_unchecked(find_driving_wire_id(node_id, 0));
                    let wire_b   = WireId::new_unchecked(find_driving_wire_id(node_id, 1));
                    let wire_out = WireId::new_unchecked(wire_id_of_output_port[node_id][0].unwrap());

                    sim_nodes.push(Node {
                        kind: NodeKind::Nand {
                            input_a: wire_a,
                            input_b: wire_b,
                            output:  wire_out,
                        },
                    });
                }
                GateKind::SavedGate(lib_node_name) => {
                    // Resolve the outer wires — these live in the parent simulation's wire space.
                    let outer_input_wires: Vec<WireId> = (0..*gate_input_count)
                        .map(|port_index| WireId::new_unchecked(find_driving_wire_id(node_id, port_index)))
                        .collect();

                    let outer_output_wires: Vec<WireId> = wire_id_of_output_port[node_id]
                        .iter()
                        .filter_map(|maybe_id| maybe_id.map(WireId::new_unchecked))
                        .collect();

                    let inner_simulation = self.compile_or_get_graph(lib_node_name)?;

                    if
                        inner_simulation.input_wires.len() != *gate_input_count || 
                        inner_simulation.output_wires.len() != *gate_output_count
                    {
                        return Err(CompileError::LibraryNodeBadIOCount(format!("Inside: {}", name)));
                    }

                    sim_nodes.push(Node {
                        kind: NodeKind::Graph {
                            inputs:     outer_input_wires,
                            outputs:    outer_output_wires,
                            simulation: Box::new(inner_simulation),
                        },
                    });
                }
                GateKind::ExternalGate(name) => {
                    // Resolve the outer wires — these live in the parent simulation's wire space.
                    let outer_input_wires: Vec<WireId> = (0..*gate_input_count)
                        .map(|port_index| WireId::new_unchecked(find_driving_wire_id(node_id, port_index)))
                        .collect();

                    let outer_output_wires: Vec<WireId> = wire_id_of_output_port[node_id]
                        .iter()
                        .filter_map(|maybe_id| maybe_id.map(WireId::new_unchecked))
                        .collect();

                    if
                        let Some(ref ext) = self.external_node_descriptions &&
                        let Some(ext) = ext.nodes.get(name) &&
                        ext.inputs.len() != *gate_input_count &&
                        ext.outputs.len() != *gate_output_count
                    {
                        return Err(CompileError::ExternalNodeBadIOCount(format!("Inside: {}", name)));
                    }

                    sim_nodes.push(Node {
                        kind: NodeKind::External {
                            inputs:     outer_input_wires,
                            outputs:    outer_output_wires,
                            closure_name: name.to_string(),
                        },
                    });
                }
            }
        }

        // ── Output wire ids: which parent wire feeds each output pseudo-node ──────

        let sim_output_wire_ids: Vec<WireId> = (0..desc.n_outputs)
            .map(|output_index| {
                WireId::new_unchecked(find_driving_wire_id(desc.output_base() + output_index, 0))
            })
            .collect();

        // ── Input wire ids: the wire that each input pseudo-node drives ───────────

        let sim_input_wire_ids: Vec<WireId> = (0..desc.n_inputs)
            .map(|input_index| {
                WireId::new_unchecked(wire_id_of_output_port[GraphDesc::input_base() + input_index][0].unwrap())
            })
            .collect();

        Ok(Simulation {
            wire_states:  WireState::new(total_wire_count),
            nodes:        Nodes { nodes: sim_nodes },
            input_wires:  sim_input_wire_ids,
            output_wires: sim_output_wire_ids,
        })
    }
}