use crate::runtime::ExternalNodeLibrary;

use super::prelude::*;

fn not_sim()->Simulation{
    Simulation{
        wire_states: WireState::new(2),
        nodes: Nodes { nodes: vec![
            Node{kind: NodeKind::Nand { input_a: WireId(0), input_b: WireId(0), output: WireId(1) }}
        ] },
        input_wires: vec![WireId(0)],
        output_wires: vec![WireId(1)],
    }
}
fn and_sim()->Simulation{
    Simulation{
        wire_states: WireState::new(4),
        nodes: Nodes { nodes: vec![
            Node{kind: NodeKind::Nand { input_a: WireId(0), input_b: WireId(1), output: WireId(2) }},
            Node{kind: NodeKind::Graph { inputs: vec![WireId(2)], outputs: vec![WireId(3)], simulation: Box::new(not_sim()) }},
        ] },
        input_wires: vec![WireId(0), WireId(1)],
        output_wires: vec![WireId(3)],
    }
}
fn or_sim()->Simulation{
    // a || b == !!(a||b) == !(!a && !b) == nand(!a, !b) == nand(not(a), not(b))
    Simulation{
        wire_states: WireState::new(5),
        nodes: Nodes { nodes: vec![
            Node{kind: NodeKind::Graph { inputs: vec![WireId(0)], outputs: vec![WireId(2)], simulation: Box::new(not_sim()) }},
            Node{kind: NodeKind::Graph { inputs: vec![WireId(1)], outputs: vec![WireId(3)], simulation: Box::new(not_sim()) }},
            Node{kind: NodeKind::Nand { input_a: WireId(2), input_b: WireId(3), output: WireId(4) }},
        ] },
        input_wires: vec![WireId(0), WireId(1)],
        output_wires: vec![WireId(4)],
    }
}
fn xor_sim()->Simulation{
    Simulation{
        wire_states: WireState::new(5),
        nodes: Nodes { nodes: vec![
            Node{kind: NodeKind::Graph { inputs: vec![WireId(0), WireId(1)], outputs: vec![WireId(2)], simulation: Box::new(or_sim()) }},
            Node{kind: NodeKind::Nand { input_a: WireId(0), input_b: WireId(1), output: WireId(3) }},
            Node{kind: NodeKind::Graph { inputs: vec![WireId(2), WireId(3)], outputs: vec![WireId(4)], simulation: Box::new(and_sim()) }},
            
        ] },
        input_wires: vec![WireId(0), WireId(1)],
        output_wires: vec![WireId(4)],
    }
}

#[test]
fn test_not_false(){
    let mut sim = not_sim();
    sim.run_ticks(&mut ExternalNodeLibrary::default(), 1);
    assert_eq!(sim.outputs().collect::<Box<[bool]>>()[0], true);
}
#[test]
fn test_not_true(){
    let mut sim = not_sim();
    sim.force_set_wire(WireId(0), true);
    sim.run_ticks(&mut ExternalNodeLibrary::default(), 1);
    assert_eq!(sim.outputs().collect::<Box<[bool]>>()[0], false);
}