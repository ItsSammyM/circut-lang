use crate::runtime::ExternalNodeLibrary;
use super::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node{
    pub kind: NodeKind  
}
#[derive(Debug, Serialize, Deserialize)]
pub enum NodeKind{
    Nand{
        input_a: WireId,
        input_b: WireId,
        output: WireId,
    },
    // PureLut{},
    // StateLut{},
    Graph{
        inputs: Vec<WireId>,
        outputs: Vec<WireId>,
        simulation: Box<Simulation>,
    },
    External{
        inputs: Vec<WireId>,
        outputs: Vec<WireId>,
        closure_name: String
    },
}

impl NodeId{
    pub fn get<'s>(&self, nodes: &'s Nodes)->&'s Node{
        nodes.nodes.get(self.0 as usize).unwrap()
    }
    pub fn get_mut<'s>(&self, nodes: &'s mut Nodes)->&'s mut Node{
        nodes.nodes.get_mut(self.0 as usize).unwrap()
    }
    /// This function is responsible for
    /// - Reading the current state
    /// - Setting the value of the nodes output wires in the next state
    /// - Dirtying the wires that it changes
    pub fn evaluate(&self, nodes: &mut Nodes, wire_states: &mut WireState, external_node_lib: &mut ExternalNodeLibrary){
        match &mut self.get_mut(nodes).kind {
            NodeKind::Nand { input_a, input_b, output } => {
                let value = !(input_a.current_value(wire_states) && input_b.current_value(wire_states));
                output.set_next(wire_states, value);
            },
            NodeKind::Graph { inputs, outputs, simulation } => {
                for (inner_wire, outer_wire) in simulation.input_wires.iter().zip(inputs.iter()){
                    let value = outer_wire.current_value(wire_states);
                    inner_wire.set_current(
                        &mut simulation.wire_states,
                        value
                    );
                    inner_wire.set_next(
                        &mut simulation.wire_states,
                        value
                    );
                }
                
                simulation.run_one_tick(external_node_lib);
                
                for (inner_wire, outer_wire) in simulation.output_wires.iter().zip(outputs.iter()){
                    let value = inner_wire.current_value(&simulation.wire_states);
                    outer_wire.set_next(
                        wire_states,
                        value
                    );
                }
            },
            NodeKind::External {
                inputs,
                outputs,
                closure_name
            } => {

                let in_vals = inputs.iter().map(|wire|wire.current_value(wire_states)).collect();
                
                let out_vals = external_node_lib.get_output(closure_name, in_vals);
                outputs
                    .into_iter()
                    .zip(out_vals.into_iter())
                    .for_each(|(outputs, out_vals)|{
                        outputs.set_next(wire_states, out_vals);
                    });
            }
        }
    }

    pub fn input_wires(&self, nodes: &Nodes)->Box<[WireId]>{
        match &self.get(nodes).kind {
            NodeKind::Nand { input_a, input_b, .. } => [*input_a, *input_b].into(),
            NodeKind::Graph { inputs, .. } => inputs.as_slice().into(),
            NodeKind::External { inputs, .. } => inputs.as_slice().into(),
        }
    }
}