use crate::quat::Quat;
use crate::error::QuatError;
use std::convert::TryInto;

#[derive(Clone, Copy)]
pub struct QuatSlice<'a> {
    bytes: &'a [u8],
    start_bit_offset: usize,
    len: usize,
}

impl<'a> QuatSlice<'a> {
    pub(crate) fn new(bytes: &'a [u8], start_bit_offset: usize, len: usize) -> Self {
        Self {
            bytes,
            start_bit_offset,
            len,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> Result<Quat, QuatError> {
        if index >= self.len {
            return Err(QuatError::IndexOutOfBounds { index, len: self.len });
        }

        let absolute_quat_index = self.start_bit_offset + index;
        let byte_index = absolute_quat_index / 4;
        let bit_shift = (3 - (absolute_quat_index % 4)) * 2;

        if byte_index >= self.bytes.len() {
            return Err(QuatError::IndexOutOfBounds { index, len: self.len });
        }

        let byte_val = self.bytes[byte_index];
        let quat_val = (byte_val >> bit_shift) & 0b11;
        quat_val.try_into()
    }

    pub fn iter(&self) -> QuatSliceIter<'a> {
        QuatSliceIter {
            slice: *self,
            index: 0,
        }
    }
}

pub struct QuatSliceIter<'a> {
    slice: QuatSlice<'a>,
    index: usize,
}

impl<'a> Iterator for QuatSliceIter<'a> {
    type Item = Quat;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let quat = self.slice.get(self.index).ok()?;
            self.index += 1;
            Some(quat)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for QuatSliceIter<'a> {}
impl<'a> std::iter::FusedIterator for QuatSliceIter<'a> {}