use crate::de::{Deserialize, DeserializeError};
use crate::ser::{Serialize, SerializeError};
use crate::SSZ;
use bitvec::field::BitField;
use bitvec::prelude::BitVec;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

/// A homogenous collection of a fixed number of boolean values.
/// NOTE: a `Bitvector` of length `0` is illegal.
///
/// NOTE: once `const_generics` and `const_evaluatable_checked` features stabilize,
/// this type can use something like
/// bitvec::array::BitArray<T, {N / 8}> where T: BitRegister, [T; {N / 8}]: BitViewSized
/// Refer: https://stackoverflow.com/a/65462213
#[derive(Debug, PartialEq, Eq)]
pub struct Bitvector<const N: usize>(BitVec);

impl<const N: usize> Default for Bitvector<N> {
    fn default() -> Self {
        assert!(N > 0);
        Self(BitVec::repeat(false, N))
    }
}

impl<const N: usize> Bitvector<N> {
    /// Return the bit at `index`. `None` if index is out-of-bounds.
    pub fn get(&mut self, index: usize) -> Option<bool> {
        self.0.get(index).map(|value| *value)
    }

    /// Set the bit at `index` to `value`. Return the previous value
    /// or `None` if index is out-of-bounds.
    pub fn set(&mut self, index: usize, value: bool) -> Option<bool> {
        self.get_mut(index).map(|mut slot| {
            let old = *slot;
            *slot = value;
            old
        })
    }
}

impl<const N: usize> Deref for Bitvector<N> {
    type Target = BitVec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Bitvector<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> SSZ for Bitvector<N> {
    fn is_variable_size() -> bool {
        false
    }

    fn size_hint() -> usize {
        (N + 7) / 8
    }
}

impl<const N: usize> Serialize for Bitvector<N> {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        assert!(N > 0);
        let bytes_to_write = Self::size_hint();
        buffer.reserve(bytes_to_write);
        for byte in self.0.chunks(8) {
            buffer.push(byte.load_le());
        }
        Ok(bytes_to_write)
    }
}

impl<const N: usize> Deserialize for Bitvector<N> {
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError> {
        assert!(N > 0);

        let mut result = Self::default();
        for (slot, byte) in result.chunks_mut(8).zip(encoding.iter().copied()) {
            slot.store_le(byte);
        }
        Ok(result)
    }
}

impl<const N: usize> FromIterator<bool> for Bitvector<N> {
    // NOTE: only takes the first `N` values from `iter` and
    // uses the default `false` for missing values.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        assert!(N > 0);

        let mut result: Bitvector<N> = Default::default();
        for (index, bit) in iter.into_iter().enumerate().take(N) {
            result.set(index, bit);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialize;

    const COUNT: usize = 12;

    #[test]
    fn encode_bitvector() {
        let value: Bitvector<4> = Bitvector::default();
        let encoding = serialize(&value).expect("can encode");
        let expected = [0u8];
        assert_eq!(encoding, expected);

        let value: Bitvector<COUNT> = Bitvector::default();
        let encoding = serialize(&value).expect("can encode");
        let expected = [0u8, 0u8];
        assert_eq!(encoding, expected);

        let mut value: Bitvector<COUNT> = Bitvector::default();
        value.set(3, true).expect("test data correct");
        value.set(4, true).expect("test data correct");
        assert_eq!(value.get(4).expect("test data correct"), true);
        assert_eq!(value.get(0).expect("test data correct"), false);
        let encoding = serialize(&value).expect("can encode");
        let expected = [24u8, 0u8];
        assert_eq!(encoding, expected);
    }

    #[test]
    fn decode_bitvector() {
        let bytes = vec![12u8];
        let result = Bitvector::<4>::deserialize(&bytes).expect("test data is correct");
        let expected = Bitvector::from_iter(vec![false, false, true, true]);
        assert_eq!(result, expected);

        let bytes = vec![24u8, 1u8];
        let result = Bitvector::<COUNT>::deserialize(&bytes).expect("test data is correct");
        let expected = Bitvector::from_iter(vec![
            false, false, false, true, true, false, false, false, true, false, false, false,
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn roundtrip_bitvector() {
        let input = Bitvector::<COUNT>::from_iter(vec![
            false, false, false, true, true, false, false, false, false, false, false, false,
        ]);
        let mut buffer = vec![];
        let _ = input.serialize(&mut buffer).expect("can serialize");
        let recovered = Bitvector::<COUNT>::deserialize(&buffer).expect("can decode");
        assert_eq!(input, recovered);
    }
}