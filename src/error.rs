use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuatError {
    InvalidValue(u8),
    IndexOutOfBounds { index: usize, len: usize },
    EmptyKey,
    SliceOutOfRange { start: usize, end: usize, len: usize },
}

impl std::error::Error for QuatError {}

impl fmt::Display for QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuatError::InvalidValue(val) => {
                write!(f, "Invalid raw value for Quat: {val}. Expected a 2-bit value (0-3).")
            }
            QuatError::IndexOutOfBounds { index, len } => {
                write!(f, "Index {index} is out of bounds for QuatBuffer of length {len}.")
            }
            QuatError::EmptyKey => {
                write!(f, "Cipher key cannot be empty.")
            }
            QuatError::SliceOutOfRange { start, end, len } => {
                write!(f, "Slice range [{start}..{end}] is out of bounds for buffer length {len}.")
            }
        }
    }
}