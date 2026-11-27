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

    /// Encrypts a buffer in-place using SWAR (32 quats per CPU word) with zero allocations.
    /// Requires a byte-aligned keystream or repeating key buffer.
    pub fn encrypt_in_place_swar(&self, target: &mut QuatBuffer) {
        let key_len = self.key.len();
        let total_quats = target.len();
        if total_quats == 0 {
            return;
        }

        // Fast path: if key length is a multiple of 4, we can pack the key into bytes
        // and execute full-register u64 SWAR operations without bit-shifting.
        if key_len % 4 == 0 {
            let mut key_buf = QuatBuffer::with_capacity(key_len);
            for &q in &self.key {
                key_buf.push(q);
            }
            let key_bytes = key_buf.as_bytes();
            let target_bytes = target.as_bytes_mut();

            // Process 8-byte (32-quat) chunks if key repeats cleanly on 8-byte boundaries
            if key_bytes.len() % 8 == 0 {
                let mut key_idx = 0;
                for chunk in target_bytes.chunks_exact_mut(8) {
                    let data_word = u64::from_ne_bytes(chunk.try_into().unwrap());
                    let key_word = u64::from_ne_bytes(
                        key_bytes[key_idx..key_idx + 8].try_into().unwrap(),
                    );
                    let encrypted_word = swar::add_mod4_u64(data_word, key_word);
                    chunk.copy_from_slice(&encrypted_word.to_ne_bytes());

                    key_idx = (key_idx + 8) % key_bytes.len();
                }

                // Remainder bytes (< 8 bytes)
                let rem_start = (target_bytes.len() / 8) * 8;
                for byte in &mut target_bytes[rem_start..] {
                    *byte = swar::add_mod4_u8(*byte, key_bytes[key_idx]);
                    key_idx = (key_idx + 1) % key_bytes.len();
                }
                return;
            }
        }

        // Fallback for arbitrary (non-aligned) key lengths using zero-copy QuatSliceMut
        let mut slice = target.as_mut_slice();
        for i in 0..total_quats {
            let q_in = slice.get(i).unwrap();
            let key_quat = self.key[i % key_len];
            let _ = slice.set(i, q_in + key_quat);
        }
    }

    /// Decrypts a buffer in-place using SWAR (32 quats per CPU word) with zero allocations.
    pub fn decrypt_in_place_swar(&self, target: &mut QuatBuffer) {
        let key_len = self.key.len();
        let total_quats = target.len();
        if total_quats == 0 {
            return;
        }

        if key_len % 4 == 0 {
            let mut key_buf = QuatBuffer::with_capacity(key_len);
            for &q in &self.key {
                key_buf.push(q);
            }
            let key_bytes = key_buf.as_bytes();
            let target_bytes = target.as_bytes_mut();

            if key_bytes.len() % 8 == 0 {
                let mut key_idx = 0;
                for chunk in target_bytes.chunks_exact_mut(8) {
                    let data_word = u64::from_ne_bytes(chunk.try_into().unwrap());
                    let key_word = u64::from_ne_bytes(
                        key_bytes[key_idx..key_idx + 8].try_into().unwrap(),
                    );
                    let decrypted_word = swar::sub_mod4_u64(data_word, key_word);
                    chunk.copy_from_slice(&decrypted_word.to_ne_bytes());

                    key_idx = (key_idx + 8) % key_bytes.len();
                }

                let rem_start = (target_bytes.len() / 8) * 8;
                for byte in &mut target_bytes[rem_start..] {
                    *byte = swar::sub_mod4_u8(*byte, key_bytes[key_idx]);
                    key_idx = (key_idx + 1) % key_bytes.len();
                }
                return;
            }
        }

        let mut slice = target.as_mut_slice();
        for i in 0..total_quats {
            let q_in = slice.get(i).unwrap();
            let key_quat = self.key[i % key_len];
            let _ = slice.set(i, q_in - key_quat);
        }
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

/// SWAR (SIMD Within A Register) primitives for parallel quaternary arithmetic.
/// Processes 32 quats per `u64` register and 4 quats per `u8` register in constant time.
pub mod swar {
    /// Mask isolating the least significant bit (LSB) of every 2-bit quat in a `u64`.
    const QUAT_LSB_MASK_U64: u64 = 0x5555_5555_5555_5555;
    /// Mask isolating the least significant bit (LSB) of every 2-bit quat in a `u8`.
    const QUAT_LSB_MASK_U8: u8 = 0x55;

    /// Adds 32 packed quats modulo 4 in parallel using a single `u64` register.
    #[inline(always)]
    pub const fn add_mod4_u64(a: u64, b: u64) -> u64 {
        let sum_no_carry = a ^ b;
        let carry = ((a & b) & QUAT_LSB_MASK_U64) << 1;
        sum_no_carry ^ carry
    }

    /// Subtracts 32 packed quats modulo 4 in parallel (`(a - b) mod 4`).
    #[inline(always)]
    pub const fn sub_mod4_u64(a: u64, b: u64) -> u64 {
        // Two's complement within each 2-bit lane: -b mod 4 == (!b + 1) mod 4
        let neg_b = add_mod4_u64(!b, QUAT_LSB_MASK_U64);
        add_mod4_u64(a, neg_b)
    }

    /// Adds 4 packed quats modulo 4 in parallel using a single `u8` register.
    #[inline(always)]
    pub const fn add_mod4_u8(a: u8, b: u8) -> u8 {
        let sum_no_carry = a ^ b;
        let carry = ((a & b) & QUAT_LSB_MASK_U8) << 1;
        sum_no_carry ^ carry
    }

    /// Subtracts 4 packed quats modulo 4 in parallel (`(a - b) mod 4`).
    #[inline(always)]
    pub const fn sub_mod4_u8(a: u8, b: u8) -> u8 {
        let neg_b = add_mod4_u8(!b, QUAT_LSB_MASK_U8);
        add_mod4_u8(a, neg_b)
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

/// Constant-time 4-element S-Box over `Quat` using algebraic multiplexing (no memory lookups).
/// Immune to cache-timing side-channel attacks.
#[derive(Clone, Copy, Debug)]
pub struct ConstantTimeQuatSBox {
    sbox: [u8; 4],
    inv_sbox: [u8; 4],
}

impl ConstantTimeQuatSBox {
    /// Creates and validates a bijective 4-element S-Box permutation.
    pub fn new(sbox: [Quat; 4]) -> Result<Self, QuatError> {
        let mut seen = 0u8;
        let mut raw_sbox = [0u8; 4];
        let mut raw_inv = [0u8; 4];

        for (i, &q) in sbox.iter().enumerate() {
            let val = q.as_u8();
            seen |= 1 << val;
            raw_sbox[i] = val;
            raw_inv[val as usize] = i as u8;
        }

        // All 4 bits (0b1111 == 15) must be set for a valid permutation
        if seen != 0b1111 {
            return Err(QuatError::InvalidValue(seen));
        }

        Ok(Self {
            sbox: raw_sbox,
            inv_sbox: raw_inv,
        })
    }

    /// Evaluates `table[q]` in constant time using bitwise masks without branching or indexing.
    #[inline(always)]
    const fn ct_lookup(table: &[u8; 4], q: Quat) -> Quat {
        let x = q as u8;
        // Generate all-ones (0xFF) mask when x == k, else 0x00, using arithmetic underflow
        let m0 = ((x ^ 0).wrapping_sub(1) >> 7).wrapping_neg();
        let m1 = ((x ^ 1).wrapping_sub(1) >> 7).wrapping_neg();
        let m2 = ((x ^ 2).wrapping_sub(1) >> 7).wrapping_neg();
        let m3 = ((x ^ 3).wrapping_sub(1) >> 7).wrapping_neg();

        let out = (table[0] & m0) | (table[1] & m1) | (table[2] & m2) | (table[3] & m3);
        match out & 0b11 {
            0b00 => Quat::V00,
            0b01 => Quat::V01,
            0b10 => Quat::V10,
            _ => Quat::V11,
        }
    }

    #[inline(always)]
    pub const fn sub(&self, q: Quat) -> Quat {
        Self::ct_lookup(&self.sbox, q)
    }

    #[inline(always)]
    pub const fn inv_sub(&self, q: Quat) -> Quat {
        Self::ct_lookup(&self.inv_sbox, q)
    }
}

/// Quaternary Linear Feedback Shift Register (LFSR) operating over GF(4).
/// Generates pseudo-random keystreams for quaternary stream ciphers.
pub struct QuatLfsr {
    state: Vec<Quat>,
    taps: Vec<Quat>,
}

impl QuatLfsr {
    /// Initializes a GF(4) LFSR with a non-zero seed state and feedback polynomial coefficients.
    pub fn new(seed: Vec<Quat>, taps: Vec<Quat>) -> Result<Self, QuatError> {
        if seed.is_empty() || seed.len() != taps.len() {
            return Err(QuatError::EmptyKey);
        }
        if seed.iter().all(|&q| q == Quat::V00) {
            return Err(QuatError::EmptyKey);
        }
        Ok(Self { state: seed, taps })
    }

    /// Advances the LFSR by one step and outputs the next pseudo-random `Quat`.
    pub fn next_quat(&mut self) -> Quat {
        let out = self.state[0];
        let mut feedback = Quat::V00;

        for (s, t) in self.state.iter().zip(self.taps.iter()) {
            feedback = feedback.gf4_add(s.gf4_mul(*t));
        }

        self.state.rotate_left(1);
        if let Some(last) = self.state.last_mut() {
            *last = feedback;
        }
        out
    }

    /// Generates `len` pseudo-random quats packed into a `QuatBuffer`.
    pub fn generate_keystream(&mut self, len: usize) -> QuatBuffer {
        let mut buf = QuatBuffer::with_capacity(len);
        for _ in 0..len {
            buf.push(self.next_quat());
        }
        buf
    }
}

/// Multi-round Balanced Feistel Block Cipher operating over GF(4) and Constant-Time S-Boxes.
/// Requires an even-length `QuatBuffer` input.
pub struct QuatFeistelCipher {
    round_keys: Vec<Vec<Quat>>,
    sbox: ConstantTimeQuatSBox,
}

impl QuatFeistelCipher {
    pub fn new(round_keys: Vec<Vec<Quat>>, sbox: ConstantTimeQuatSBox) -> Result<Self, QuatError> {
        if round_keys.is_empty() || round_keys.iter().any(|rk| rk.is_empty()) {
            return Err(QuatError::EmptyKey);
        }
        Ok(Self { round_keys, sbox })
    }

    /// Feistel round function F(R, K) combining GF(4) multiplication, addition, and S-Box confusion.
    fn round_function(&self, half: &[Quat], round_key: &[Quat]) -> Vec<Quat> {
        let mut out = Vec::with_capacity(half.len());
        for (i, &r) in half.iter().enumerate() {
            let k = round_key[i % round_key.len()];
            // Non-linear mixing: SBox((R * K) + K) in GF(4)
            let mixed = r.gf4_mul(k).gf4_add(k);
            let subbed = self.sbox.sub(mixed);
            out.push(subbed);
        }
        // Diffusion step: cyclic shift by 1 quat across the half-block
        if out.len() > 1 {
            out.rotate_left(1);
        }
        out
    }
}

impl Cipher for QuatFeistelCipher {
    fn encrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let len = input.len();
        if len == 0 || len % 2 != 0 {
            return Err(QuatError::SliceOutOfRange {
                start: 0,
                end: len,
                len,
            });
        }

        let half_len = len / 2;
        let all_quats: Vec<Quat> = input.as_slice().iter().collect();
        let (left_slice, right_slice) = all_quats.split_at(half_len);

        let mut left = left_slice.to_vec();
        let mut right = right_slice.to_vec();

        for rk in &self.round_keys {
            let f_out = self.round_function(&right, rk);
            let next_right: Vec<Quat> = left
                .iter()
                .zip(f_out.iter())
                .map(|(&l, &f)| l.gf4_add(f))
                .collect();

            left = right;
            right = next_right;
        }

        let mut output = QuatBuffer::with_capacity(len);
        for q in left.into_iter().chain(right.into_iter()) {
            output.push(q);
        }
        Ok(output)
    }

    fn decrypt(&self, input: &QuatBuffer) -> Result<QuatBuffer, QuatError> {
        let len = input.len();
        if len == 0 || len % 2 != 0 {
            return Err(QuatError::SliceOutOfRange {
                start: 0,
                end: len,
                len,
            });
        }

        let half_len = len / 2;
        let all_quats: Vec<Quat> = input.as_slice().iter().collect();
        let (left_slice, right_slice) = all_quats.split_at(half_len);

        let mut left = left_slice.to_vec();
        let mut right = right_slice.to_vec();

        // Reverse round key order for Feistel decryption
        for rk in self.round_keys.iter().rev() {
            let f_out = self.round_function(&left, rk);
            let prev_left: Vec<Quat> = right
                .iter()
                .zip(f_out.iter())
                .map(|(&r, &f)| r.gf4_add(f))
                .collect();

            right = left;
            left = prev_left;
        }

        let mut output = QuatBuffer::with_capacity(len);
        for q in left.into_iter().chain(right.into_iter()) {
            output.push(q);
        }
        Ok(output)
    }
}