use crate::quat::Quat;
use crate::buffer::QuatBuffer;
use crate::error::QuatError;
use std::convert::TryInto;

pub trait Cipher {
    fn encrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError>;
    fn decrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError>;
}

pub struct QuatStreamCipher {
    key: Vec<Quat>,
}

impl QuatStreamCipher {
    pub fn new(key: Vec<Quat>) -> Result<Self, QuatError> {
        if key.is_empty() {
            return Err(QuatError::EmptyKey);
        }
        Ok(Self { key })
    }
}

impl Cipher for QuatStreamCipher {
    fn encrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let mut output = QuatBuffer::with_capacity(input.len());

        for (i, q_in) in input.into_iter().enumerate() {
            let key_quat = self.key[i % self.key.len()];
            let sum = (q_in.as_u8() + key_quat.as_u8()) % 4;
            let q_out: Quat = sum.try_into()?;
            output.push(q_out);
        }

        Ok(output)
    }

    fn decrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let mut output = QuatBuffer::with_capacity(input.len());

        for (i, q_in) in input.into_iter().enumerate() {
            let key_quat = self.key[i % self.key.len()];
            // Modular subtraction mod 4 in quaternary space
            let diff = (4 + q_in.as_u8() - key_quat.as_u8()) % 4;
            let q_out: Quat = diff.try_into()?;
            output.push(q_out);
        }

        Ok(output)
    }
}

pub struct QuatSubstitutionCipher {
    sbox: [Quat; 4],
    inv_sbox: [Quat; 4],
}

impl QuatSubstitutionCipher {
    /// Creates a new substitution cipher with a given 4-element S-Box permutation.
    pub fn new(sbox: [Quat; 4]) -> Self {
        let mut inv_sbox = [Quat::V00; 4];
        for (i, &q) in sbox.iter().enumerate() {
            inv_sbox[q as usize] = (i as u8).try_into().unwrap();
        }
        Self { sbox, inv_sbox }
    }

    /// Substitutes input buffer quats using the S-Box.
    pub fn substitute(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let mut output = QuatBuffer::with_capacity(input.len());
        for i in 0..input.len() {
            let q = input.get(i)?;
            let substituted = self.sbox[q as usize];
            output.push(substituted);
        }
        Ok(output)
    }

    /// Reverses the substitution using the inverse S-Box.
    pub fn unsubstitute(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let mut output = QuatBuffer::with_capacity(input.len());
        for i in 0..input.len() {
            let q = input.get(i)?;
            let unsubstituted = self.inv_sbox[q as usize];
            output.push(unsubstituted);
        }
        Ok(output)
    }
}