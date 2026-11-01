use crate::quat::Quat;
use crate::error::QuatError;
use crate::slice::QuatSlice;
use std::convert::TryInto;

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