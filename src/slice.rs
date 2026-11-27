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

/// A mutable, zero-copy view into a contiguous sequence of packed 2-bit quats.
pub struct QuatSliceMut<'a> {
    bytes: &'a mut [u8],
    start_bit_offset: usize,
    len: usize,
}

impl<'a> QuatSliceMut<'a> {
    pub(crate) fn new(bytes: &'a mut [u8], start_bit_offset: usize, len: usize) -> Self {
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

    /// Downgrades the mutable slice into an immutable `QuatSlice`.
    #[inline]
    pub fn as_slice(&self) -> QuatSlice<'_> {
        QuatSlice::new(self.bytes, self.start_bit_offset, self.len)
    }

    #[inline]
    pub fn get(&self, index: usize) -> Result<Quat, QuatError> {
        self.as_slice().get(index)
    }

    /// Mutates a single quat in-place at the specified logical index.
    #[inline]
    pub fn set(&mut self, index: usize, quat: Quat) -> Result<(), QuatError> {
        if index >= self.len {
            return Err(QuatError::IndexOutOfBounds {
                index,
                len: self.len,
            });
        }

        let absolute_quat_index = self.start_bit_offset + index;
        let byte_index = absolute_quat_index / 4;
        let bit_shift = (3 - (absolute_quat_index % 4)) * 2;

        if byte_index >= self.bytes.len() {
            return Err(QuatError::IndexOutOfBounds {
                index,
                len: self.len,
            });
        }

        let val: u8 = quat.into();
        self.bytes[byte_index] &= !(0b11 << bit_shift);
        self.bytes[byte_index] |= val << bit_shift;

        Ok(())
    }

    /// Creates a sub-slice view from `start..end` without allocating memory.
    pub fn slice_mut(&mut self, start: usize, end: usize) -> Result<QuatSliceMut<'_>, QuatError> {
        if start > end || end > self.len {
            return Err(QuatError::SliceOutOfRange {
                start,
                end,
                len: self.len,
            });
        }

        Ok(QuatSliceMut::new(
            self.bytes,
            self.start_bit_offset + start,
            end - start,
        ))
    }

    /// Exposes the underlying raw byte buffer if the slice is byte-aligned
    /// (i.e., starts at a byte boundary and covers whole bytes).
    #[inline]
    pub fn as_aligned_bytes_mut(&mut self) -> Option<&mut [u8]> {
        if self.start_bit_offset % 4 == 0 && self.len % 4 == 0 {
            let start_byte = self.start_bit_offset / 4;
            let byte_len = self.len / 4;
            Some(&mut self.bytes[start_byte..start_byte + byte_len])
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for QuatSliceIter<'a> {}
impl<'a> std::iter::FusedIterator for QuatSliceIter<'a> {}