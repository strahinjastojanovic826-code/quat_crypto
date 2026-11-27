use crate::error::QuatError;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, Sub, BitAnd, BitOr, BitXor, Not};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

impl Quat {
    /// Addition in GF(4), equivalent to bitwise XOR.
    #[inline(always)]
    pub const fn gf4_add(self, rhs: Self) -> Self {
        match (self as u8) ^ (rhs as u8) {
            0b00 => Quat::V00,
            0b01 => Quat::V01,
            0b10 => Quat::V10,
            _ => Quat::V11,
        }
    }

    /// Subtraction in GF(4) (in characteristic 2, subtraction is identical to addition).
    #[inline(always)]
    pub const fn gf4_sub(self, rhs: Self) -> Self {
        self.gf4_add(rhs)
    }

    /// Multiplication in GF(4) modulo x^2 + x + 1.
    /// Constant-time lookup via bit-manipulation (no memory indexing).
    #[inline(always)]
    pub const fn gf4_mul(self, rhs: Self) -> Self {
        let a = self as u8;
        let b = rhs as u8;

        // Polynomial multiplication before reduction:
        // (a1*x + a0) * (b1*x + b0) = (a1*b1)*x^2 + (a1*b0 ^ a0*b1)*x + (a0*b0)
        let a0 = a & 1;
        let a1 = (a >> 1) & 1;
        let b0 = b & 1;
        let b1 = (b >> 1) & 1;

        let p0 = a0 & b0;
        let p1 = (a1 & b0) ^ (a0 & b1);
        let p2 = a1 & b1;

        // Reduce x^2 -> x + 1
        let res = ((p1 ^ p2) << 1) | (p0 ^ p2);
        match res {
            0b00 => Quat::V00,
            0b01 => Quat::V01,
            0b10 => Quat::V10,
            _ => Quat::V11,
        }
    }

    /// Multiplicative inverse in GF(4). Returns `None` for zero element (`V00`).
    /// In GF(4), x^-1 = x^2 for all non-zero elements.
    #[inline(always)]
    pub const fn gf4_inv(self) -> Option<Self> {
        match self {
            Quat::V00 => None,
            Quat::V01 => Some(Quat::V01), // 1^-1 = 1
            Quat::V10 => Some(Quat::V11), // α^-1 = α + 1
            Quat::V11 => Some(Quat::V10), // (α + 1)^-1 = α
        }
    }

    /// Division in GF(4): `self * rhs^-1`.
    #[inline(always)]
    pub const fn gf4_div(self, rhs: Self) -> Option<Self> {
        match rhs.gf4_inv() {
            Some(inv) => Some(self.gf4_mul(inv)),
            None => None,
        }
    }

    // --- Domain-Specific Conversions (DNA & Telecommunications PAM4) ---

    /// Converts a `Quat` into its corresponding DNA nucleobase (`A`, `C`, `G`, `T`).
    #[inline(always)]
    pub const fn to_dna_base(self) -> char {
        match self {
            Quat::V00 => 'A',
            Quat::V01 => 'C',
            Quat::V10 => 'G',
            Quat::V11 => 'T',
        }
    }

    /// Parses a DNA nucleobase (`A`/`a`, `C`/`c`, `G`/`g`, `T`/`t`, `U`/`u`) into a `Quat`.
    #[inline]
    pub const fn from_dna_base(base: u8) -> Result<Self, QuatError> {
        match base {
            b'A' | b'a' => Ok(Quat::V00),
            b'C' | b'c' => Ok(Quat::V01),
            b'G' | b'g' => Ok(Quat::V10),
            b'T' | b't' | b'U' | b'u' => Ok(Quat::V11),
            _ => Err(QuatError::InvalidValue(base)),
        }
    }

    /// Maps a `Quat` to a PAM4 (4-level Pulse Amplitude Modulation) symbol using Gray coding:
    /// `00 -> -3`, `01 -> -1`, `11 -> +1`, `10 -> +3`.
    #[inline(always)]
    pub const fn to_pam4_level(self) -> i8 {
        match self {
            Quat::V00 => -3,
            Quat::V01 => -1,
            Quat::V11 => 1,
            Quat::V10 => 3,
        }
    }

    /// Demodulates a PAM4 voltage level (`-3, -1, +1, +3`) back into a `Quat`.
    #[inline]
    pub const fn from_pam4_level(level: i8) -> Result<Self, QuatError> {
        match level {
            -3 => Ok(Quat::V00),
            -1 => Ok(Quat::V01),
            1 => Ok(Quat::V11),
            3 => Ok(Quat::V10),
            _ => Err(QuatError::InvalidValue(level as u8)),
        }
    }


}