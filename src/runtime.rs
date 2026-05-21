use std::collections::HashMap;
use crate::{compile::{CompileError, Compiler}, script::CircutLangScript, simulation::{WireId, simulation::Simulation}};

pub struct Runtime{
    simulation: Simulation,
}
impl Runtime{
    pub fn new(simulation: Simulation)->Self{
        Self { simulation }
    }
    pub fn new_compile(script: CircutLangScript)->Result<Self, CompileError>{
        Ok(Self{
            simulation: Compiler::compile(script)?
        })
    }
    pub fn run_one_tick(&mut self, set_input: Option<GateInput>, external_node_library: &mut ExternalNodeLibrary)->Result<(), RuntimeError>{
        if let Some(set_input) = set_input {
            self
                .simulation
                .input_wires
                .clone()
                .into_iter()
                .zip(set_input.into_iter())
                .for_each(|(wire, set_input)|self.simulation.force_set_wire(wire, set_input));
        }
        self.simulation.run_one_tick(external_node_library)
    }
    pub fn wire_value(&self, wire: WireId) -> bool {
        wire.current_value(&self.simulation.wire_states)
    }
    pub fn output_values(&self)->GateOutput{
        self.simulation.outputs().collect()
    }
}
#[must_use]
#[derive(Debug, Clone)]
pub enum RuntimeError{
    ExternalNodeMissing(String)
}

#[derive(Default)]
pub struct ExternalNodeLibrary<'a> {
    nodes: HashMap<String, &'a mut dyn FnMut(GateInput)->GateOutput>
}
impl<'a> ExternalNodeLibrary<'a> {
    pub fn new(nodes: HashMap<String, &'a mut dyn FnMut(GateInput)->GateOutput>) -> Self {
        Self { nodes }
    }
    pub fn insert_builder(mut self, name: String, closure: &'a mut dyn FnMut(GateInput)->GateOutput) -> Self{
        self.nodes.insert(name, closure);
        self
    }
    pub fn insert(&'a mut self, name: String, closure: &'a mut dyn FnMut(GateInput)->GateOutput) -> &'a mut Self{
        self.nodes.insert(name, closure);
        self
    }
    pub fn get_output_unchecked(&mut self, node: &String, input: GateInput) -> GateOutput {
        let closure = &mut self
            .nodes
            .get_mut(node)
            .expect(format!("External Node Lib is missing node named: {}", node).as_str());
        (closure)(input)
    }
    pub fn get_output(&mut self, node: &String, input: GateInput) -> Option<GateOutput> {
        let closure = &mut self
            .nodes
            .get_mut(node)?;
        Some((closure)(input))
    }
}

type GateInput = Vec<bool>;
type GateOutput = Vec<bool>;