use crate::error::QuatError;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, Sub, BitAnd, BitOr, BitXor, Not};

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

impl Add for Quat {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let val = (self as u8 + rhs as u8) % 4;
        val.try_into().unwrap()
    }
}

impl Sub for Quat {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        let val = (4 + self as u8 - rhs as u8) % 4;
        val.try_into().unwrap()
    }
}

impl BitAnd for Quat {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        let val = (self as u8) & (rhs as u8);
        val.try_into().unwrap()
    }
}

impl BitOr for Quat {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        let val = (self as u8) | (rhs as u8);
        val.try_into().unwrap()
    }
}

impl BitXor for Quat {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let val = (self as u8) ^ (rhs as u8);
        val.try_into().unwrap()
    }
}

impl Not for Quat {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        let val = !(self as u8) & 0b11;
        val.try_into().unwrap()
    }
}