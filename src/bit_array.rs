use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitArray {
    // Each u32 holds 32 bits. Word size is 32.
    words: Vec<u32>,
    len: u32,
}

impl BitArray {
    pub fn new_empty() -> Self {
        Self {
            words: vec![],
            len: 0,
        }
    }
    pub fn new(len: u32) -> Self {
        // ceiling division — e.g. 33 bits needs 2 words
        let words = (len + 31) / 32;
        Self {
            words: vec![0; words as usize],
            len,
        }
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn get(&self, i: u32) -> bool {
        let (word, bit) = Self::get_word_and_bit(i);
        (self.words[word as usize] >> bit) & 1 == 1
    }

    pub fn set(&mut self, i: u32, val: bool) {
        let (word, bit) = Self::get_word_and_bit(i);
        if val {
            self.words[word as usize] |= 1 << bit;
        } else {
            self.words[word as usize] &= !(1 << bit);
        }
    }

    /// Returns the new length
    pub fn push(&mut self, val: bool)->&u32{
        if self.len % 32 == 0 {
            self.words.push(0);
        }

        self.set(self.len, val);

        self.len += 1;
        &self.len
    }

    /// Trashes self and puts other in its place
    /// Same as what clone would probably do but avoids allocating a new vec like clone would.
    pub fn set_as_clone(&mut self, other: &Self){
        self.len = other.len;
        self.words.iter_mut().zip(&other.words).for_each(|(my_word, other_word)|*my_word = *other_word);
    }

    pub const fn get_word_and_bit(i: u32)->(u32, u32){
        (i / 32, i % 32)
    }


    pub fn iter_bools(&self) -> impl Iterator<Item = bool> {
        (0..self.len).into_iter()
            .map(|i|self.get(i))
    }

    pub fn bools_as_string(&self)->String{
        "[".to_string()+&self.iter_bools().enumerate().map(|(i, val)|format!("{}: {:?}", i, val)).collect::<Box<[String]>>().join("\n")+"]"
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word_and_bit() {
        assert_eq!(BitArray::get_word_and_bit(0), (0, 0));
        assert_eq!(BitArray::get_word_and_bit(5), (0, 5));
        assert_eq!(BitArray::get_word_and_bit(32), (1, 0));
        assert_eq!(BitArray::get_word_and_bit(33), (1, 1));
        assert_eq!(BitArray::get_word_and_bit(32*2-1), (1, 31));
    }

    #[test]
    fn test_set_get() {
        let mut set = BitArray::new(32*3);
        assert_eq!(set.get(36), false);
        set.set(36, true);
        for i in 0..set.len {
            assert_eq!(
                set.get(i),
                if i == 36 {
                    true
                }else{
                    false
                }
            )
        }
    }
}