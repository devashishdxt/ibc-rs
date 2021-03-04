use std::convert::TryFrom;

use ibc_proto::cosmos::crypto::multisig::v1beta1::CompactBitArray;

const MASK: u8 = 0b1000_0000;

pub trait BitArray {
    fn new(num_bits: usize) -> Self;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> bool;

    fn set(&mut self, index: usize, value: bool) -> bool;

    fn num_true_bits_before(&self, index: usize) -> usize;
}

impl BitArray for CompactBitArray {
    fn new(num_bits: usize) -> Self {
        let extra_bits_stored = num_bits & 7; // equivalent to `num_bits % 8`

        let mut elem_size = num_bits >> 3; // equivalent to `num_bits / 8`

        if extra_bits_stored > 0 {
            elem_size += 1;
        }

        Self {
            extra_bits_stored: u32::try_from(extra_bits_stored).unwrap(),
            elems: vec![0; elem_size],
        }
    }

    fn len(&self) -> usize {
        if self.extra_bits_stored == 0 {
            return self.elems.len() * 8;
        }

        ((self.elems.len() - 1) * 8) + usize::try_from(self.extra_bits_stored).unwrap()
    }

    fn get(&self, index: usize) -> bool {
        if index >= self.len() {
            return false;
        }

        return (self.elems[index >> 3] & (MASK >> (index & 7))) > 0; // equivalent to `(self.elems[index / 8] & (MASK >> (index % 8)))`
    }

    fn set(&mut self, index: usize, value: bool) -> bool {
        if index >= self.len() {
            return false;
        }

        if value {
            self.elems[index >> 3] |= MASK >> (index & 7); // equivalent to `self.elems[index / 8] |= MASK >> (index % 8)`
        } else {
            self.elems[index >> 3] &= !(MASK >> (index & 7)); // equivalent to `self.elems[index / 8] &= !(MASK >> (index % 8))`
        }

        return true;
    }

    fn num_true_bits_before(&self, index: usize) -> usize {
        let mut num_true_values = 0;

        for i in 0..index {
            if self.get(i) {
                num_true_values += 1;
            }
        }

        num_true_values
    }
}
