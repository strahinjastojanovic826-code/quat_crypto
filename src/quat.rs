use crate::error::QuatError;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quat {
    V00 = 0b00,
    V01 = 0b01,
    V10 = 0b10,
    V11 = 0b11,
}

impl Quat {
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Quat {
    type Error = QuatError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & 0b11 {
            0b00 => Ok(Quat::V00),
            0b01 => Ok(Quat::V01),
            0b10 => Ok(Quat::V10),
            0b11 => Ok(Quat::V11),
            _ => Err(QuatError::InvalidValue(value)),
        }
    }
}

impl From<Quat> for u8 {
    #[inline(always)]
    fn from(quat: Quat) -> Self {
        quat as u8
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val: u8 = (*self).into();
        write!(f, "{:02b}", val)
    }
}