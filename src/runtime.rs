use std::collections::HashMap;
use crate::{script::CircutLangScript, simulation::{WireId, simulation::Simulation}};

pub struct Runtime{
    simulation: Simulation,
}
impl Runtime{
    pub fn new(simulation: Simulation)->Self{
        Self { simulation }
    }
    pub fn new_from_script(script: CircutLangScript)->Result<Self, String>{
        Ok(Self{
            simulation: Self::compile(script)?
        })
    }
    pub fn run_one_tick(&mut self, set_input: Option<GateInput>, external_node_library: &mut ExternalNodeLibrary){
        if let Some(set_input) = set_input {
            self
                .simulation
                .input_wires
                .clone()
                .into_iter()
                .zip(set_input.into_iter())
                .for_each(|(wire, set_input)|self.simulation.force_set_wire(wire, set_input));
        }
        self.simulation.run_one_tick(external_node_library);
    }
    pub fn wire_value(&self, wire: impl Into<WireId>) -> bool {
        wire.into().current_value(&self.simulation.wire_states)
    }
    pub fn output_values(&self)->GateOutput{
        self.simulation.outputs().collect()
    }
}

#[derive(Default)]
pub struct ExternalNodeLibrary<'a>{
    nodes: HashMap<String, &'a mut dyn FnMut(GateInput)->GateOutput>
}
impl<'a> ExternalNodeLibrary<'a>{
    pub fn get_output(&mut self, node: &String, input: GateInput) -> GateOutput {
        let closure = &mut self
            .nodes
            .get_mut(node)
            .expect("external node lib doesnt have that node");
        (closure)(input)
    }
}

type GateInput = Vec<bool>;
type GateOutput = Vec<bool>;