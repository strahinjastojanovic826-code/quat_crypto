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

    // --- TEST 1: Najlakši - Osnovne vrednosti i konverzija u u8 ---
    #[test]
    fn test_01_quat_basic_values() {
        assert_eq!(Quat::V00.as_u8(), 0b00);
        assert_eq!(Quat::V01.as_u8(), 0b01);
        assert_eq!(Quat::V10.as_u8(), 0b10);
        assert_eq!(Quat::V11.as_u8(), 0b11);

        let val: u8 = Quat::V10.into();
        assert_eq!(val, 2);
    }

    // --- TEST 2: Validacija nevažećih vrednosti (TryFrom) ---
    #[test]
fn test_02_quat_try_from_invalid() {
    // Vrednost 4 (0b100) maskirana sa 0b11 daje 0 (0b00), pa vraća Ok(Quat::V00)
    let masked_val: Result<Quat, QuatError> = Quat::try_from(4);
    assert_eq!(masked_val, Ok(Quat::V00));

    let valid: Result<Quat, QuatError> = Quat::try_from(3);
    assert_eq!(valid, Ok(Quat::V11));
}

    // --- TEST 3: Osnovno dodavanje i čitanje iz bafera ---
    #[test]
    fn test_03_buffer_push_and_get() {
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

    // --- TEST 4: Provera uštede memorije (pakovanje 4 kvata u 1 bajt) ---
    #[test]
    fn test_04_buffer_memory_packing() {
        let mut buf = QuatBuffer::new();
        
        // 0 kvata -> 0 bajtova
        assert_eq!(buf.byte_len(), 0);

        // 1 do 4 kvata moraju stati u tačno 1 bajt (75% uštede)
        buf.push(Quat::V00);
        assert_eq!(buf.byte_len(), 1);
        buf.push(Quat::V01);
        buf.push(Quat::V10);
        buf.push(Quat::V11);
        assert_eq!(buf.byte_len(), 1);

        // 5. kvat prelazi u drugi bajt
        buf.push(Quat::V01);
        assert_eq!(buf.byte_len(), 2);
        assert_eq!(buf.len(), 5);
    }

    // --- TEST 5: Korišćenje quat_vec! makroa ---
    #[test]
    fn test_05_quat_vec_macro() {
        let buf = quat_vec![Quat::V11, Quat::V00, Quat::V10, Quat::V01, Quat::V11];
        
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.get(0), Ok(Quat::V11));
        assert_eq!(buf.get(3), Ok(Quat::V01));
        assert_eq!(buf.get(4), Ok(Quat::V11));
    }

    // --- TEST 6: Iteracija kroz bafer (owned i borrowed) ---
    #[test]
    fn test_06_buffer_iteration() {
        let original = vec![Quat::V01, Quat::V10, Quat::V11, Quat::V00];
        let buf: QuatBuffer = original.iter().copied().collect();

        assert_eq!(buf.len(), 4);

        let collected: Vec<Quat> = buf.into_iter().collect();
        assert_eq!(collected, original);
    }

    // --- TEST 7: Zero-Copy Slicing nad baferom ---
    #[test]
    fn test_07_buffer_slicing() {
        let buf = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11, Quat::V01];
        
        let slice = buf.slice(1, 4).unwrap();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice.get(0), Ok(Quat::V01));
        assert_eq!(slice.get(1), Ok(Quat::V10));
        assert_eq!(slice.get(2), Ok(Quat::V11));
    }

    // --- TEST 8: Kriptografska simetrija (Encryption & Decryption) ---
    #[test]
    fn test_08_cipher_encrypt_decrypt() {
        let key = vec![Quat::V01, Quat::V11];
        let cipher = QuatStreamCipher::new(key).unwrap();

        let plaintext = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11];
        
        let ciphertext = cipher.encrypt(&plaintext).unwrap();
        assert_ne!(ciphertext.get(0), plaintext.get(0));

        let decrypted = cipher.decrypt(&ciphertext).unwrap();
        
        for i in 0..plaintext.len() {
            assert_eq!(plaintext.get(i), decrypted.get(i));
        }
    }

    // --- TEST 9: Upravljanje greškama (Index out of bounds, Empty key, Slice out of range) ---
    #[test]
    fn test_09_error_edge_cases() {
        let mut buf = QuatBuffer::new();
        buf.push(Quat::V01);

        // Indeks van granica
        assert!(matches!(buf.get(5), Err(QuatError::IndexOutOfBounds { .. })));

        // Prazan ključ za šifru
        let cipher_res = QuatStreamCipher::new(vec![]);
        assert!(matches!(cipher_res, Err(QuatError::EmptyKey)));

        // Neispravan opseg slajsa
        let slice_res = buf.slice(0, 5);
        assert!(matches!(slice_res, Err(QuatError::SliceOutOfRange { .. })));
    }

    // --- TEST 10: Najstresniji test - Veliki podaci, masivno pakovanje i kripto integritet (10.000 kvatova) ---
    #[test]
    fn test_10_stress_test_large_buffer() {
        let total_quats = 10_000;
        let mut original_vec = Vec::with_capacity(total_quats);
        let mut buf = QuatBuffer::with_capacity(total_quats);

        // Generisanje pseudonasumičnog niza od 10.000 kvatova
        for i in 0..total_quats {
            let q_val = (i % 4) as u8;
            let q = Quat::try_from(q_val).unwrap();
            original_vec.push(q);
            buf.push(q);
        }

        // Provera dužine i fizičke veličine memorije (10000 kvatova / 4 = 2500 bajtova)
        assert_eq!(buf.len(), total_quats);
        assert_eq!(buf.byte_len(), total_quats / 4);

        // Testiranje šifrovanja i dešifrovanja celog velikog niza
        let key = vec![Quat::V11, Quat::V01, Quat::V10, Quat::V00, Quat::V11];
        let cipher = QuatStreamCipher::new(key).unwrap();

        let encrypted = cipher.encrypt(&buf).expect("Encryption failed under stress");
        let decrypted = cipher.decrypt(&encrypted).expect("Decryption failed under stress");

        assert_eq!(decrypted.len(), total_quats);

        // Detaljna verifikacija svake pojedinačne vrednosti nakon teškog procesiranja
        for i in 0..total_quats {
            assert_eq!(
                decrypted.get(i).unwrap(),
                original_vec[i],
                "Data corruption detected at index {i}"
            );
        }

        // Testiranje sub-slajsa na masivnom baferu
        let sub_slice = buf.slice(2500, 7500).unwrap();
        assert_eq!(sub_slice.len(), 5000);
        assert_eq!(sub_slice.get(0).unwrap(), original_vec[2500]);
        assert_eq!(sub_slice.get(4999).unwrap(), original_vec[7499]);
    }
}