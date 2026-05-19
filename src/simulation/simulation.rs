use serde::{Deserialize, Serialize};
use crate::runtime::ExternalNodeLibrary;
use super::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Simulation{
    pub wire_states: WireState,
    pub nodes: Nodes,

    // Which inner wires map to this node's external input ports
    pub input_wires:  Vec<WireId>,
    // Which inner wires map to this node's external output ports
    pub output_wires: Vec<WireId>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Nodes{
    pub nodes: Vec<Node>
}
impl Simulation{
    pub fn run_one_tick(&mut self, external_nodes: &mut ExternalNodeLibrary){
        for node in (0..self.nodes.nodes.len()).map(|i|NodeId(i as u32)) {
            node.evaluate(&mut self.nodes, &mut self.wire_states, external_nodes);
        }
        self.wire_states.update();
    }

    pub fn run_ticks(&mut self, external_nodes: &mut ExternalNodeLibrary, ticks: u32){
        (0..ticks).for_each(|_|self.run_one_tick(external_nodes));
    }
    pub fn force_set_wire(&mut self, id: WireId, val: bool){
        self.wire_states.set_in_current(id, val);
        self.wire_states.set_in_next(id, val);
    }
    fn debug_output(&mut self, external_nodes: &mut ExternalNodeLibrary, frames: u32){
        println!("I: {:?}", self.outputs().collect::<Box<[bool]>>());
        (0..frames)
            .into_iter()
            .for_each(|i|{
                self.run_one_tick(external_nodes);
                println!("{}: {:?}", i, self.outputs().collect::<Box<[bool]>>());
            });
    }
    fn debug(&mut self, external_nodes: &mut ExternalNodeLibrary, frames: u32){
        println!("I: {}", self.wire_states.current().bools_as_string());
        (0..frames)
            .into_iter()
            .for_each(|i|{
                self.run_one_tick(external_nodes);
                println!("{i}: {}", self.wire_states.current().bools_as_string());
            });
    }
    pub fn outputs(&self)->impl Iterator<Item = bool>{
        self.output_wires
            .iter()
            .map(|wire|wire.current_value(&self.wire_states))
    }
}


