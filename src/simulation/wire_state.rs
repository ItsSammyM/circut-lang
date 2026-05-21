use serde::{Deserialize, Serialize};

use super::WireId;
use crate::bit_array::BitArray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireState {
    current: BitArray,
    next:    BitArray,
}

impl WireState {
    pub fn new(wire_count: u32) -> Self {
        Self {
            current: BitArray::new(wire_count),
            next:    BitArray::new(wire_count),
        }
    }

    pub fn get_current(&self, id: WireId) -> bool {
        self.current.get(id.0)
    }
    pub fn get_next(&self, id: WireId) -> bool {
        self.next.get(id.0)
    }

    pub fn set_in_next(&mut self, id: WireId, val: bool) {
        self.next.set(id.0, val);
    }
    pub fn set_in_current(&mut self, id: WireId, val: bool) {
        self.current.set(id.0, val);
    }

    pub fn update(&mut self) {
        self.current.set_as_clone(&self.next)
    }

    pub fn current(&self) -> &BitArray {
        &self.current
    }
    pub fn next(&self) -> &BitArray {
        &self.next
    }
    // pub fn commit(&mut self) -> Vec<WireId> {
    //     let mut changed = Vec::new();

    //     for word_idx in 0..self.current.set.len() {
    //         let curr = self.current.set[word_idx];
    //         let next = self.next.set[word_idx];
    //         let diff = curr ^ next;

    //         if diff != 0 {
    //             // at least one bit changed in this word
    //             // find exactly which bits
    //             for bit in 0..32u32 {
    //                 if (diff >> bit) & 1 == 1 {
    //                     let wire_id = word_idx as u32 * 32 + bit;
    //                     if wire_id < self.current.len {
    //                         changed.push(WireId(wire_id));
    //                     }
    //                 }
    //             }
    //             self.current.set[word_idx] = next;
    //         }
    //     }

    //     changed
    // }
}