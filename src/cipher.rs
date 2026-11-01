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