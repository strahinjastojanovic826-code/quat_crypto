pub mod error;
pub mod quat;
pub mod slice;
pub mod buffer;
pub mod cipher;
pub mod macros;

pub use error::QuatError;
pub use quat::Quat;
pub use slice::QuatSlice;
pub use buffer::QuatBuffer;
pub use cipher::{Cipher, QuatStreamCipher};

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use crate::cipher::QuatSubstitutionCipher;
    use std::time::Instant;

    // --- TEST 1: Najlakši - Osnovne vrednosti i konverzija u u8 ---
    #[test]
    fn test_01_quat_basic_values() {
        let start = Instant::now();
        let iterations = 100_000;

        for _ in 0..iterations {
            assert_eq!(Quat::V00.as_u8(), 0b00);
            assert_eq!(Quat::V01.as_u8(), 0b01);
            assert_eq!(Quat::V10.as_u8(), 0b10);
            assert_eq!(Quat::V11.as_u8(), 0b11);

            let val: u8 = Quat::V10.into();
            assert_eq!(val, 2);
        }

        println!("[BENCHMARK] test_01_quat_basic_values ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 2: Validation of invalid values (TryFrom) ---
    #[test]
    fn test_02_quat_try_from_invalid() {
        let start = Instant::now();
        let iterations = 100_000;

        for _ in 0..iterations {
            let masked_val: Result<Quat, QuatError> = Quat::try_from(4);
            assert_eq!(masked_val, Ok(Quat::V00));

            let valid: Result<Quat, QuatError> = Quat::try_from(3);
            assert_eq!(valid, Ok(Quat::V11));
        }

        println!("[BENCHMARK] test_02_quat_try_from_invalid ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 3: Basic buffer push and get ---
    #[test]
    fn test_03_buffer_push_and_get() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let mut buf = QuatBuffer::new();
            assert!(buf.is_empty());

            buf.push(Quat::V01);
            buf.push(Quat::V11);
            buf.push(Quat::V00);

            assert_eq!(buf.len(), 3);
            assert_eq!(buf.get(0), Ok(Quat::V01));
            assert_eq!(buf.get(1), Ok(Quat::V11));
            assert_eq!(buf.get(2), Ok(Quat::V00));
        }

        println!("[BENCHMARK] test_03_buffer_push_and_get ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 4: Memory saving check (packing 4 quats into 1 byte) ---
    #[test]
    fn test_04_buffer_memory_packing() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let mut buf = QuatBuffer::new();
            assert_eq!(buf.byte_len(), 0);

            buf.push(Quat::V00);
            assert_eq!(buf.byte_len(), 1);
            buf.push(Quat::V01);
            buf.push(Quat::V10);
            buf.push(Quat::V11);
            assert_eq!(buf.byte_len(), 1);

            buf.push(Quat::V01);
            assert_eq!(buf.byte_len(), 2);
            assert_eq!(buf.len(), 5);
        }

        println!("[BENCHMARK] test_04_buffer_memory_packing ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 5: Using quat_vec! macro ---
    #[test]
    fn test_05_quat_vec_macro() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let buf = quat_vec![Quat::V11, Quat::V00, Quat::V10, Quat::V01, Quat::V11];

            assert_eq!(buf.len(), 5);
            assert_eq!(buf.get(0), Ok(Quat::V11));
            assert_eq!(buf.get(3), Ok(Quat::V01));
            assert_eq!(buf.get(4), Ok(Quat::V11));
        }

        println!("[BENCHMARK] test_05_quat_vec_macro ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 6: Iteration through buffer (owned and borrowed) ---
    #[test]
    fn test_06_buffer_iteration() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let original: Vec<Quat> = vec![Quat::V01, Quat::V10, Quat::V11, Quat::V00];
            let buf: QuatBuffer = original.iter().copied().collect();

            assert_eq!(buf.len(), 4);

            let collected: Vec<Quat> = buf.into_iter().collect();
            assert_eq!(collected, original);
        }

        println!("[BENCHMARK] test_06_buffer_iteration ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 7: Zero-Copy Slicing over buffer ---
    #[test]
    fn test_07_buffer_slicing() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let buf = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11, Quat::V01];

            let slice: QuatSlice<'_> = buf.slice(1, 4).unwrap();
            assert_eq!(slice.len(), 3);
            assert_eq!(slice.get(0), Ok(Quat::V01));
            assert_eq!(slice.get(1), Ok(Quat::V10));
            assert_eq!(slice.get(2), Ok(Quat::V11));
        }

        println!("[BENCHMARK] test_07_buffer_slicing ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 8: Cryptographic symmetry (Encryption & Decryption) ---
    #[test]
    fn test_08_cipher_encrypt_decrypt() {
        let start = Instant::now();
        let iterations = 5_000;

        for _ in 0..iterations {
            let key: Vec<Quat> = vec![Quat::V01, Quat::V11];
            let cipher = QuatStreamCipher::new(key).unwrap();

            let plaintext = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11];

            let ciphertext = cipher.encrypt(&plaintext).unwrap();
            assert_ne!(ciphertext.get(0), plaintext.get(0));

            let decrypted = cipher.decrypt(&ciphertext).unwrap();
            for i in 0..plaintext.len() {
                assert_eq!(plaintext.get(i), decrypted.get(i));
            }
        }

        println!("[BENCHMARK] test_08_cipher_encrypt_decrypt ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 9: Error handling (Index out of bounds, Empty key, Slice out of range) ---
    #[test]
    fn test_09_error_edge_cases() {
        let start = Instant::now();
        let iterations = 10_000;

        for _ in 0..iterations {
            let mut buf = QuatBuffer::new();
            buf.push(Quat::V01);

            // Index out of bounds
            assert!(matches!(buf.get(5), Err(QuatError::IndexOutOfBounds { .. })));

            // Empty key for cipher
            let cipher_res: Result<QuatStreamCipher, _> = QuatStreamCipher::new(vec![]);
            assert!(matches!(cipher_res, Err(QuatError::EmptyKey)));

            // Invalid slice range
            let slice_res: Result<QuatSlice<'_>, _> = buf.slice(0, 5);
            assert!(matches!(slice_res, Err(QuatError::SliceOutOfRange { .. })));
        }

        println!("[BENCHMARK] test_09_error_edge_cases ({} iterations): {:?}", iterations, start.elapsed());
    }

    // --- TEST 10: Stress test - Large data, massive packing and crypto integrity (10,000 quats) ---
    #[test]
    fn test_10_stress_test_large_buffer() {
        let start = Instant::now();

        let total_quats: usize = 10_000;
        let mut original_vec: Vec<Quat> = Vec::with_capacity(total_quats);
        let mut buf = QuatBuffer::with_capacity(total_quats);

        // Generating pseudo-random array of 10,000 quats
        for i in 0..total_quats {
            let q_val: u8 = (i % 4) as u8;
            let q = Quat::try_from(q_val).unwrap();
            original_vec.push(q);
            buf.push(q);
        }

        // Checking length and physical memory size (10000 quats / 4 = 2500 bytes)
        assert_eq!(buf.len(), total_quats);
        assert_eq!(buf.byte_len(), total_quats / 4);

        // Testing encryption and decryption of entire large array
        let key: Vec<Quat> = vec![Quat::V11, Quat::V01, Quat::V10, Quat::V00, Quat::V11];
        let cipher = QuatStreamCipher::new(key).unwrap();

        let encrypted = cipher.encrypt(&buf).expect("Encryption failed under stress");
        let decrypted = cipher.decrypt(&encrypted).expect("Decryption failed under stress");

        assert_eq!(decrypted.len(), total_quats);

        // Detailed verification of each individual value after heavy processing
        for i in 0..total_quats {
            assert_eq!(
                decrypted.get(i).unwrap(),
                original_vec[i],
                "Data corruption detected at index {i}"
            );
        }

        // Testing sub-slice in massive buffer
        let sub_slice = buf.slice(2500, 7500).unwrap();
        assert_eq!(sub_slice.len(), 5000);
        assert_eq!(sub_slice.get(0).unwrap(), original_vec[2500]);
        assert_eq!(sub_slice.get(4999).unwrap(), original_vec[7499]);

        println!("[BENCHMARK] test_10_stress_test_large_buffer (1 run with 10,000 quats): {:?}", start.elapsed());
    }

    #[test]
fn test_11_quat_arithmetic_and_logic() {
    // Test Addition (mod 4)
    assert_eq!(Quat::V01 + Quat::V10, Quat::V11); // 1 + 2 = 3 (V11)
    assert_eq!(Quat::V11 + Quat::V01, Quat::V00); // 3 + 1 = 4 -> 0 (V00)

    // Test Subtraction (mod 4)
    assert_eq!(Quat::V11 - Quat::V01, Quat::V10); // 3 - 1 = 2 (V10)
    assert_eq!(Quat::V00 - Quat::V01, Quat::V11); // 0 - 1 -> 3 (V11)

    // Test Bitwise Operations
    assert_eq!(Quat::V10 & Quat::V11, Quat::V10); // 0b10 & 0b11 = 0b10
    assert_eq!(Quat::V01 | Quat::V10, Quat::V11); // 0b01 | 0b10 = 0b11
    assert_eq!(Quat::V11 ^ Quat::V10, Quat::V01); // 0b11 ^ 0b10 = 0b01
    assert_eq!(!Quat::V01, Quat::V10);             // !0b01 & 0b11 = 0b10
}

// --- TEST 12: Quat Substitution Cipher (S-Box) & Benchmark ---
#[test]
fn test_12_substitution_cipher_and_benchmark() {
    let sbox_mapping = [Quat::V11, Quat::V00, Quat::V01, Quat::V10];
    let cipher = QuatSubstitutionCipher::new(sbox_mapping);

    // Create a test buffer using macro
    let input_buf = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11];

    // --- INTEGRATED BENCHMARK START ---
    let start_time = std::time::Instant::now();
    
    // Run substitution 10,000 times to get meaningful performance numbers
    let iterations = 10_000;
    let mut encrypted_buf = input_buf.clone();
    for _ in 0..iterations {
        encrypted_buf = cipher.substitute(&input_buf).unwrap();
    }
    let decrypted_buf = cipher.unsubstitute(&encrypted_buf).unwrap();

    let duration = start_time.elapsed();
    println!("[BENCHMARK] QuatSubstitutionCipher ({} iterations): {:?}", iterations, duration);
    // --- INTEGRATED BENCHMARK END ---

    // Verify correctness
    assert_eq!(decrypted_buf.get(0).unwrap(), input_buf.get(0).unwrap());
    assert_eq!(decrypted_buf.get(1).unwrap(), input_buf.get(1).unwrap());
    assert_eq!(decrypted_buf.get(2).unwrap(), input_buf.get(2).unwrap());
    assert_eq!(decrypted_buf.get(3).unwrap(), input_buf.get(3).unwrap());
}

// --- TEST 13: Raw Bytes to QuatBuffer TryFrom Conversion & Benchmark ---
#[test]
fn test_13_raw_bytes_conversion_and_benchmark() {
    // 100 raw bytes will produce 400 quats
    let raw_bytes: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();

    // --- INTEGRATED BENCHMARK START ---
    let start_time = std::time::Instant::now();
    
    let iterations = 5_000;
    let mut buffer = QuatBuffer::new();
    for _ in 0..iterations {
        buffer = QuatBuffer::try_from(raw_bytes.as_slice()).unwrap();
    }

    let duration = start_time.elapsed();
    println!("[BENCHMARK] QuatBuffer TryFrom<&[u8]> ({} iterations, 400 quats each): {:?}", iterations, duration);
    // --- INTEGRATED BENCHMARK END ---

    // Verify length and packing correctness
    assert_eq!(buffer.len(), 400);
    assert_eq!(buffer.byte_len(), 100);
}

}