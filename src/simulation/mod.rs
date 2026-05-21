use serde::{Deserialize, Serialize};

use self::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(u32);
impl NodeId{
    pub fn new_unchecked(i: u32)->Self{
        NodeId(i)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireId(u32);
impl WireId{
    pub fn new_unchecked(i: u32)->Self{
        WireId(i)
    }
    pub fn new_from_sim(i: u32, sim: &Simulation)->Option<Self>{
        if sim.wire_states.current().len() > i {
            Some(WireId(i))
        }else{
            None
        }
    }
    pub fn i(&self)->u32{
        self.0
    }
    // fn next_value(&self, state: &WireState)->bool{
    //     state.get_next(*self)
    // }
    pub fn current_value(&self, state: &WireState)->bool{
        state.get_current(*self)
    }
    fn set_next(&self, state: &mut WireState, value: bool) {
        state.set_in_next(*self, value)
    }
    fn set_current(&self, state: &mut WireState, value: bool) {
        state.set_in_current(*self, value)
    }
}

pub mod simulation;
pub mod node;
pub mod wire_state;
#[cfg(test)]
mod test_gates;
pub mod prelude;