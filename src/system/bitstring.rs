use std::{array, collections::VecDeque, ops::ControlFlow};

use crate::PostSystem;

#[derive(Debug, Clone)]
pub struct BitString {
    /// The words of the bit string.
    /// The bits are stored in little-endian order.
    /// There is always at least one word.
    words: VecDeque<usize>,

    /// The index of the first bit in the first word.
    start: u8,
    /// The index of the last bit in the last word.
    end: u8,

    /// Length of the bit string.
    len: usize,
}

impl BitString {
    /// Create a new empty bit string.
    fn new() -> Self {
        Self {
            words: [0].into_iter().collect(),
            start: 0,
            end: 0,
            len: 0,
        }
    }

    /// Append `count` bits to the end of the bit string, from the little-endian `bits`.
    ///
    /// `count` must be at most `usize::BITS`, and `bits` must not have any bits set beyond the `count`-th bit.
    fn append(&mut self, bits: usize, count: u8) {
        debug_assert!(count <= usize::BITS as u8);

        let rotated = bits.rotate_left(self.end as u32);

        let lower_mask = usize::MAX << self.end;
        let upper_mask = !lower_mask;

        *self.words.back_mut().unwrap() |= rotated & lower_mask;
        self.end += count;

        if self.end >= usize::BITS as u8 {
            self.end %= usize::BITS as u8;

            self.words.push_back(rotated & upper_mask);
        }

        self.len += count as usize;
    }

    /// Delete `count` bits from the start of the bit string, returning them.
    ///
    /// `count` must be strictly less than `usize::BITS`.
    /// If `count` is greater than the number of bits in the bit string, the result is truncated and the string is left empty.
    fn delete(&mut self, count: u8) -> usize {
        debug_assert!(count <= usize::BITS as u8);

        let mask = usize::MAX >> (usize::BITS as u8 - count);

        let lower = *self.words.front_mut().unwrap() >> self.start;
        self.start += count;

        let upper = if self.start >= usize::BITS as u8 {
            self.start %= usize::BITS as u8;

            self.words.pop_front().unwrap();
            if self.words.len() <= 1 && self.start > self.end {
                self.end = self.start;
            }
            if self.words.is_empty() {
                self.words.push_back(0);
                self.start = 0;
                self.end = 0;
            }

            *self.words.front_mut().unwrap() << (count - self.start)
        } else {
            0
        };

        let ret = (lower | upper) & mask;

        self.len -= count as usize;

        ret
    }
}

impl PartialEq for BitString {
    fn eq(&self, other: &Self) -> bool {
        if self.length() != other.length() {
            return false;
        }

        if self.start > other.start {
            return other.eq(self);
        }

        let offset = other.start - self.start;
        let overflow_mask = (1 << offset) - 1;

        // Bits which overflowed from the previous self word, to be compared with the next other word.
        let mut overflowed = other.words.front().unwrap() & overflow_mask;

        for (&self_word, &other_word) in self.words.iter().zip(other.words.iter()) {
            let rotated = self_word.rotate_left(offset as u32);
            if overflowed | (rotated & !overflow_mask) != other_word {
                return false;
            }
            overflowed = rotated & overflow_mask;
        }

        if other.words.len() > self.words.len()
            && other.words.back().unwrap() & overflow_mask != overflowed
        {
            return false;
        }

        true
    }
}
impl Eq for BitString {}

impl PostSystem for BitString {
    fn new_decompressed(compressed: &[bool]) -> Self {
        let mut this = Self::new();

        for &b in compressed {
            this.append(
                match b {
                    false => 0b000,
                    true => 0b001,
                },
                3,
            );
        }

        this
    }

    fn length(&self) -> usize {
        self.len
    }

    fn as_list(&self) -> VecDeque<bool> {
        let mut list: VecDeque<_> = self
            .words
            .iter()
            .flat_map(|&word| (0..usize::BITS).map(move |i| (word >> i) & 1 == 1))
            .collect();

        for _ in 0..self.start {
            list.pop_front();
        }
        for _ in 0..(usize::BITS as u8 - self.end) {
            list.pop_back();
        }

        list
    }

    fn evolve(&mut self) -> ControlFlow<()> {
        if self.length() < 3 {
            return ControlFlow::Break(());
        }

        let deleted = self.delete(3);

        match deleted & 1 {
            0 => self.append(0b00, 2),
            1 => self.append(0b1011, 4),
            _ => unreachable!(),
        }

        ControlFlow::Continue(())
    }

    const PREFERRED_TIMESTEP: u8 = 11;

    fn evolve_preferred(&mut self) {
        debug_assert!(self.length() >= 3 * Self::PREFERRED_TIMESTEP as usize);

        let deleted = self.delete(3 * Self::PREFERRED_TIMESTEP);

        let mut key = 0;
        for i in 0..Self::PREFERRED_TIMESTEP {
            key |= ((deleted >> (3 * i)) & 1) << i;
        }

        let lut_entry = LUT.with(|lut| lut[key]);
        let bits = (lut_entry & 0xFFFF_FFFF_FFFF) as usize;
        let len = (lut_entry >> 48) as u8;

        self.append(bits, len);
    }
}

thread_local! {
    /// A lookup table for bit strings of length `3 * BitString::PREFERRED_TIMESTEP` = `3 * 10`.
    ///
    /// The result is a `u64` with the lower 48 bits containing the bits to append,
    /// and the upper 16 bits containing the number of bits to append.
    static LUT: [u64; const { 1 << BitString::PREFERRED_TIMESTEP }] = {
        array::from_fn(|key| {
            let mut bits: u64 = 0;
            let mut len: u64 = 0;

            for i in 0..BitString::PREFERRED_TIMESTEP {
                match (key >> i) & 1 {
                    0 => len += 2,
                    1 => {
                        bits |= 0b1011 << len;
                        len += 4;
                    }
                    _ => unreachable!(),
                }
            }

            bits | (len << 48)
        })
    };
}

#[cfg(test)]
mod tests {
    use std::usize;

    use crate::tests_for_system;

    use super::*;

    tests_for_system!(BitString);

    #[test]
    fn tests_equality() {
        let mut bit_string = BitString::new();
        let mut other = BitString::new();

        assert_eq!(bit_string, other);

        bit_string.append(0b101, 3);
        assert_ne!(bit_string, other);

        other.append(0b101, 3);
        assert_eq!(bit_string, other);

        bit_string.append(0b010, 3);
        assert_ne!(bit_string, other);

        other.append(0b010, 3);
        assert_eq!(bit_string, other);

        bit_string.append(0b0, 1);
        assert_ne!(bit_string, other);

        other.append(0b0, 1);
        assert_eq!(bit_string, other);

        bit_string.append(usize::MAX, usize::BITS as u8);
        assert_ne!(bit_string, other);

        other.append(usize::MAX, usize::BITS as u8);
        assert_eq!(bit_string, other);

        let mut bit_string = BitString::new();
        let mut other = BitString::new();

        bit_string.append(0b1010, 4);
        other.append(0b10, 2);
        assert_ne!(bit_string, other);

        bit_string.delete(2);
        assert_eq!(bit_string, other);

        bit_string.append(usize::MAX, usize::BITS as u8);
        other.append(usize::MAX, usize::BITS as u8);
        assert_eq!(bit_string, other);

        bit_string.append(0b1010, 4);
        assert_ne!(bit_string, other);
    }

    #[test]
    fn appends() {
        let mut bit_string = BitString::new();
        assert_eq!(bit_string.as_list().make_contiguous(), []);

        bit_string.append(0b101, 3);
        assert_eq!(bit_string.as_list().make_contiguous(), [true, false, true]);

        bit_string.append(0b010, 3);
        assert_eq!(
            bit_string.as_list().make_contiguous(),
            [true, false, true, false, true, false]
        );

        bit_string.append(0b0, 1);
        assert_eq!(
            bit_string.as_list().make_contiguous(),
            [true, false, true, false, true, false, false]
        );

        bit_string.append(usize::MAX, usize::BITS as u8);
        assert_eq!(
            bit_string.as_list().make_contiguous().len(),
            (usize::BITS + 7) as _
        );
    }

    #[test]
    fn deletes() {
        let mut bit_string = BitString::new();
        bit_string.append(0xAAAA_AAAA_AAAA_AAA7, 64);
        bit_string.append(0xF, 4);

        assert_eq!(bit_string.delete(8), 0xA7);
        assert_eq!(bit_string.delete(64), 0x0FAA_AAAA_AAAA_AAAA);

        assert_eq!(bit_string.as_list().make_contiguous(), []);
    }

    #[test]
    fn gets_length() {
        let mut bit_string = BitString::new();
        for l in 0..usize::BITS * 4 {
            assert_eq!(bit_string.length(), l as _);
            bit_string.append(0, 1);
        }

        bit_string.delete(7);
        assert_eq!(bit_string.length(), usize::BITS as usize * 4 - 7);
    }
}
