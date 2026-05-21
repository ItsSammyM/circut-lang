use std::collections::HashMap;
use crate::{compile::{CompileError, Compiler}, external_node_descriptions::ExternalNodeDescrptions, script::CircutLangScript, simulation::{WireId, simulation::Simulation}};

pub struct Runtime{
    simulation: Simulation,
}
impl Runtime{
    pub fn new(simulation: Simulation)->Self{
        Self { simulation }
    }
    /// External node descriptions is only here to cause the compiler to produce an error if wired improperly
    pub fn new_compile(script: CircutLangScript, external_node_descriptions: Option<ExternalNodeDescrptions>)->Result<Self, CompileError>{
        Ok(Self{
            simulation: Compiler::compile(script, external_node_descriptions)?
        })
    }
    pub fn run_one_tick(&mut self, external_node_library: &mut ExternalNodeLibrary)->Result<(), RuntimeError>{
        self.simulation.run_one_tick(external_node_library)
    }
    pub fn run_one_tick_with_io(&mut self, input: GateInput, external_node_library: &mut ExternalNodeLibrary)->Result<GateOutput, RuntimeError>{
        self.set_input_values(input);
        self.simulation.run_one_tick(external_node_library)?;
        Ok(self.output_values())
    }
    pub fn wire_value(&self, wire: WireId) -> bool {
        wire.current_value(&self.simulation.wire_states)
    }
    pub fn output_values(&self)->GateOutput{
        self.simulation.outputs().collect()
    }
    pub fn set_input_values(&mut self, input: GateInput){
        self
            .simulation
            .input_wires
            .clone()
            .into_iter()
            .zip(input.into_iter())
            .for_each(|(wire, val)|self.simulation.force_set_wire(wire, val));
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
    pub fn get_output(&mut self, node: &String, input: GateInput) -> Option<GateOutput> {
        let closure = &mut self
            .nodes
            .get_mut(node)?;
        Some((closure)(input))
    }
}

type GateInput = Vec<bool>;
type GateOutput = Vec<bool>;