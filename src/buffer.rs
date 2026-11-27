use crate::quat::Quat;
use crate::error::QuatError;
use crate::slice::QuatSlice;
use std::convert::TryInto;
use crate::slice::QuatSliceMut;

#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct QuatBuffer {
    bytes: Vec<u8>,
    len: usize,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let byte_capacity = (capacity + 3) / 4;
        Self {
            bytes: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    pub fn push(&mut self, quat: Quat) {
        let byte_index = self.len / 4;
        let bit_shift = (3 - (self.len % 4)) * 2;

        if byte_index >= self.bytes.len() {
            self.bytes.push(0);
        }

        let val: u8 = quat.into();
        self.bytes[byte_index] &= !(0b11 << bit_shift);
        self.bytes[byte_index] |= val << bit_shift;
        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Result<Quat, QuatError> {
        if index >= self.len {
            return Err(QuatError::IndexOutOfBounds { index, len: self.len });
        }
        let byte_index = index / 4;
        let bit_shift = (3 - (index % 4)) * 2;
        let byte_val = self.bytes[byte_index];
        let quat_val = (byte_val >> bit_shift) & 0b11;

        quat_val.try_into()
    }

    pub fn as_slice(&self) -> QuatSlice<'_> {
        QuatSlice::new(&self.bytes, 0, self.len)
    }

    pub fn slice(&self, start: usize, end: usize) -> Result<QuatSlice<'_>, QuatError> {
        if start > end || end > self.len {
            return Err(QuatError::SliceOutOfRange { start, end, len: self.len });
        }
        Ok(QuatSlice::new(&self.bytes, start, end - start))
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    /// Encodes a DNA sequence string (e.g., "ACGT") into a packed `QuatBuffer`.
    pub fn from_dna_str(dna: &str) -> Result<Self, QuatError> {
        let mut buffer = Self::with_capacity(dna.len());
        for &byte in dna.as_bytes() {
            let quat = Quat::from_dna_base(byte)?;
            buffer.push(quat);
        }
        Ok(buffer)
    }

    /// Decodes the packed `QuatBuffer` into a DNA sequence `String`.
    pub fn to_dna_string(&self) -> String {
        let mut out = String::with_capacity(self.len);
        for quat in self.as_slice().iter() {
            out.push(quat.to_dna_base());
        }
        out
    }

    /// Modulates the `QuatBuffer` into a vector of Gray-coded PAM4 symbol levels (`-3, -1, +1, +3`).
    pub fn to_pam4_symbols(&self) -> Vec<i8> {
        self.as_slice().iter().map(|q| q.to_pam4_level()).collect()
    }

    /// Demodulates a slice of PAM4 levels (`-3, -1, +1, +3`) into a packed `QuatBuffer`.
    pub fn from_pam4_symbols(symbols: &[i8]) -> Result<Self, QuatError> {
        let mut buffer = Self::with_capacity(symbols.len());
        for &lvl in symbols {
            buffer.push(Quat::from_pam4_level(lvl)?);
        }
        Ok(buffer)
    }

    /// Returns a mutable zero-copy view over the entire buffer.
    #[inline]
    pub fn as_mut_slice(&mut self) -> QuatSliceMut<'_> {
        QuatSliceMut::new(&mut self.bytes, 0, self.len)
    }

    /// Returns a mutable sub-slice view in range `start..end`.
    pub fn slice_mut(&mut self, start: usize, end: usize) -> Result<QuatSliceMut<'_>, QuatError> {
        if start > end || end > self.len {
            return Err(QuatError::SliceOutOfRange {
                start,
                end,
                len: self.len,
            });
        }
        Ok(QuatSliceMut::new(&mut self.bytes, start, end - start))
    }

    /// Provides direct read access to the underlying packed bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Provides direct mutable access to the underlying packed bytes.
    /// Note: Modifying padding bits beyond `self.len` is safe, as `len` bounds logical access.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

}

impl Default for QuatBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// --- Collection Traits ---

impl FromIterator<Quat> for QuatBuffer {
    fn from_iter<T: IntoIterator<Item = Quat>>(iter: T) -> Self {
        let mut buffer = QuatBuffer::new();
        buffer.extend(iter);
        buffer
    }
}

impl Extend<Quat> for QuatBuffer {
    fn extend<T: IntoIterator<Item = Quat>>(&mut self, iter: T) {
        for quat in iter {
            self.push(quat);
        }
    }
}

// --- Iterators ---

pub struct IntoIter {
    buffer: QuatBuffer,
    index: usize,
}

impl Iterator for IntoIter {
    type Item = Quat;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            let quat = self.buffer.get(self.index).ok()?;
            self.index += 1;
            Some(quat)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for IntoIter {}
impl std::iter::FusedIterator for IntoIter {}

impl IntoIterator for QuatBuffer {
    type Item = Quat;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            buffer: self,
            index: 0,
        }
    }
}

impl<'a> IntoIterator for &'a QuatBuffer {
    type Item = Quat;
    type IntoIter = crate::slice::QuatSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl TryFrom<&[u8]> for QuatBuffer {
    type Error = QuatError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut buffer = QuatBuffer::with_capacity(bytes.len() * 4);
        for &b in bytes {
            // Extract 4 quats from high to low bits (Big-Endian)
            for i in 0..4 {
                let shift = (3 - i) * 2;
                let val = (b >> shift) & 0b11;
                let quat = Quat::try_from(val)?;
                buffer.push(quat);
            }
        }
        Ok(buffer)
    }
}

//Serde Serialization & Deserialization 
#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct QuatBufferWire<'a> {
    /// Exact number of 2-bit quats stored in the buffer.
    len: usize,
    /// Packed bytes containing 4 quats per byte (75% memory reduction preserved).
    #[serde(borrow)]
    bytes: std::borrow::Cow<'a, [u8]>,
}

#[cfg(feature = "serde")]
impl Serialize for QuatBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = QuatBufferWire {
            len: self.len,
            bytes: std::borrow::Cow::Borrowed(&self.bytes),
        };
        wire.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for QuatBuffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = QuatBufferWire::deserialize(deserializer)?;
        let expected_bytes = (wire.len + 3) / 4;

        if wire.bytes.len() != expected_bytes {
            return Err(de::Error::custom(format!(
                "corrupted QuatBuffer payload: expected {} packed bytes for {} quats, got {} bytes",
                expected_bytes,
                wire.len,
                wire.bytes.len()
            )));
        }

        let mut bytes = wire.bytes.into_owned();

        // Sanitize unused trailing padding bits in the last byte to maintain canonical representation
        let remainder_quats = wire.len % 4;
        if remainder_quats != 0 {
            if let Some(last_byte) = bytes.last_mut() {
                let valid_bits = remainder_quats * 2;
                let mask = !((1u8 << (8 - valid_bits)) - 1);
                *last_byte &= mask;
            }
        }

        Ok(Self {
            bytes,
            len: wire.len,
        })
    }
}