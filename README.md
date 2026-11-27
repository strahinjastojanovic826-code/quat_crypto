# quat_crypto

**`quat_crypto`** is a high-performance, memory-efficient Rust library designed for packing 2-bit quaternary values (**quats**: `00`, `01`, `10`, `11`) into standard bytes, achieving a **75% memory saving** compared to standard vector storage. It also includes zero-copy slicing, robust error handling, collection traits, and a foundational modular-arithmetic stream cipher.

---

## 🚀 Key Features

* **Extreme Memory Efficiency:** Packs 4 distinct quaternary states into a single `u8` byte using optimized bit-shifting operations.
* **Zero External Dependencies:** Pure, foundational Rust with zero bloat.
* **Zero-Copy Slicing:** View and iterate over sub-buffers without allocating heap memory (`QuatSlice`).
* **Idiomatic Rust APIs:** Full support for `TryFrom`, `From`, `Iterator`, `ExactSizeIterator`, `FusedIterator`, `FromIterator`, and `Extend`.
* **Built-in Stream Cipher:** Perform quaternary modular arithmetic encryption and decryption out of the box (`QuatStreamCipher`).
* **Declarative Macro:** Clean syntax initialization using the custom `quat_vec!` macro.
* **Arithmetic & Logical Operators:** Full `std::ops` support (`+`, `-`, `&`, `|`, `^`, `!`) for `Quat` operating directly in $\mathbb{Z}_4$ modular arithmetic.
* **Substitution Cipher (S-Box):** Quaternary substitution cipher (`QuatSubstitutionCipher`) with custom permutation tables for enhanced cryptographic transformations.
* **Raw Bytes Interoperability:** Direct conversion from standard raw bytes (`&[u8]`) into `QuatBuffer` via `TryFrom` implementation.
* **SWAR Parallel Processing:** SIMD-within-a-register operations (`u64` and `u8` words) for fast parallel quaternary arithmetic and in-place encryption/decryption.
* **Mutable Zero-Copy Slicing (`QuatSliceMut`):** Read and mutate packed quats in-place without extra memory allocations.
* **Galois Field $GF(4)$ Arithmetic:** Field addition, subtraction, multiplication, multiplicative inverse, and division over $GF(2^2)$.
* **Cryptographic Primitives:** Constant-time S-Box (`ConstantTimeQuatSBox`), $GF(4)$ LFSR keystream generator (`QuatLfsr`), and multi-round Feistel cipher (`QuatFeistelCipher`).
* **Domain-Specific Conversions (DNA & PAM4):** Direct conversion to/from DNA bases (`A`, `C`, `G`, `T`) and Gray-coded PAM4 levels (`-3`, `-1`, `+1`, `+3`).
* **Optional `serde` Support:** Feature-gated (`--features serde`) serialization and deserialization for `Quat` and `QuatBuffer`.

---

## Benchmark

| Test / Benchmark | Tested Component / Feature | Workload / Scale | Elapsed Time | Status |
| :--- | :--- | :--- | :--- | :--- |
| `test_01_quat_basic_values` | Basic `Quat` values (`V00`–`V11`), `as_u8()`, and `From<Quat> for u8` | 100,000 iterations | `2.1922ms` | Passed |
| `test_02_quat_try_from_invalid` | Validation and conversion via `TryFrom<u8> for Quat` | 100,000 iterations | `2.8095ms` | Passed |
| `test_03_buffer_push_and_get` | `QuatBuffer` push, length, and indexed `get()` access | 10,000 iterations | `2.8119ms` | Passed |
| `test_04_buffer_memory_packing` | 2-bit packing into bytes (75% memory savings) and `byte_len()` tracking | 10,000 iterations | `2.668ms` | Passed |
| `test_05_quat_vec_macro` | Declarative `quat_vec!` buffer initialization macro | 10,000 iterations | `3.2226ms` | Passed |
| `test_06_buffer_iteration` | Collecting into `QuatBuffer` and owned iteration (`IntoIterator`) | 10,000 iterations | `7.7058ms` | Passed |
| `test_07_buffer_slicing` | Zero-copy immutable sub-slicing (`QuatSlice`) | 10,000 iterations | `3.3474ms` | Passed |
| `test_08_cipher_encrypt_decrypt` | `QuatStreamCipher` encryption and decryption symmetry | 5,000 iterations | `7.5002ms` | Passed |
| `test_09_error_edge_cases` | Error handling (`IndexOutOfBounds`, `EmptyKey`, `SliceOutOfRange`) | 10,000 iterations | `1.9764ms` | Passed |
| `test_10_stress_test_large_buffer` | Large buffer packing, stream cipher round-trip, and sub-slicing | 1 run (10,000 quats / 2,500 bytes) | `2.911ms` | Passed |
| `test_11_quat_arithmetic_and_logic` | Modular arithmetic (`+`, `-` mod 4) and bitwise ops (`&`, `\|`, `^`, `!`) | Unit assertions | — | Passed |
| `test_12_substitution_cipher_and_benchmark` | `QuatSubstitutionCipher` substitution and inverse substitution | 10,000 iterations | `3.9609ms` | Passed |
| `test_13_raw_bytes_conversion_and_benchmark` | Raw byte slice conversion (`TryFrom<&[u8]> for QuatBuffer`) | 5,000 iterations (400 quats / 100 bytes each) | `92.9872ms` | Passed |
| `test_14_mutable_slice_and_benchmark` | Zero-copy mutable slicing (`QuatSliceMut`) and in-place `set()` | 10,000 iterations | `481.2µs` | Passed |
| `test_15_swar_in_place_crypto_and_benchmark` | SWAR parallel in-place encryption and decryption | 1,000 round-trips (10,000 quats each) | `73.3742ms` | Passed |
| `test_16_gf4_field_arithmetic_and_benchmark` | Galois field $GF(4)$ inverse, multiplication, and distributivity | 100,000 iterations | `7.3901ms` | Passed |
| `test_17_constant_time_sbox_and_benchmark` | `ConstantTimeQuatSBox` permutation validation, `sub()`, and `inv_sub()` | 100,000 iterations | `15.4873ms` | Passed |
| `test_18_lfsr_and_feistel_cipher_benchmark` | `QuatLfsr` keystream generation and `QuatFeistelCipher` encrypt/decrypt | 2,000 iterations (64-quat block) | `50.5048ms` | Passed |
| `test_19_dna_and_pam4_conversions_benchmark` | DNA string (`ACGT`) and Gray-coded PAM4 symbol conversions | 10,000 iterations | `29.386ms` | Passed |

cargo test --features serde -- --nocapture

| Test / Benchmark | Tested Component / Feature | Workload / Scale | Elapsed Time | Status |
| :--- | :--- | :--- | :--- | :--- |
| `test_01_quat_basic_values` | Basic `Quat` values (`V00`–`V11`), `as_u8()`, and `From<Quat> for u8` | 100,000 iterations | `3.5344ms` | Passed |
| `test_02_quat_try_from_invalid` | Validation and conversion via `TryFrom<u8> for Quat` | 100,000 iterations | `2.9626ms` | Passed |
| `test_03_buffer_push_and_get` | `QuatBuffer` push, length, and indexed `get()` access | 10,000 iterations | `3.6054ms` | Passed |
| `test_04_buffer_memory_packing` | 2-bit packing into bytes (75% memory savings) and `byte_len()` tracking | 10,000 iterations | `2.8378ms` | Passed |
| `test_05_quat_vec_macro` | Declarative `quat_vec!` buffer initialization macro | 10,000 iterations | `3.9724ms` | Passed |
| `test_06_buffer_iteration` | Collecting into `QuatBuffer` and owned iteration (`IntoIterator`) | 10,000 iterations | `8.4717ms` | Passed |
| `test_07_buffer_slicing` | Zero-copy immutable sub-slicing (`QuatSlice`) | 10,000 iterations | `3.2747ms` | Passed |
| `test_08_cipher_encrypt_decrypt` | `QuatStreamCipher` encryption and decryption symmetry | 5,000 iterations | `9.0214ms` | Passed |
| `test_09_error_edge_cases` | Error handling (`IndexOutOfBounds`, `EmptyKey`, `SliceOutOfRange`) | 10,000 iterations | `2.3226ms` | Passed |
| `test_10_stress_test_large_buffer` | Large buffer packing, stream cipher round-trip, and sub-slicing | 1 run (10,000 quats / 2,500 bytes) | `3.0374ms` | Passed |
| `test_11_quat_arithmetic_and_logic` | Modular arithmetic (`+`, `-` mod 4) and bitwise ops (`&`, `\|`, `^`, `!`) | Unit assertions | — | Passed |
| `test_12_substitution_cipher_and_benchmark` | `QuatSubstitutionCipher` substitution and inverse substitution | 10,000 iterations | `4.2072ms` | Passed |
| `test_13_raw_bytes_conversion_and_benchmark` | Raw byte slice conversion (`TryFrom<&[u8]> for QuatBuffer`) | 5,000 iterations (400 quats / 100 bytes each) | `80.7681ms` | Passed |
| `test_14_mutable_slice_and_benchmark` | Zero-copy mutable slicing (`QuatSliceMut`) and in-place `set()` | 10,000 iterations | `526.8µs` | Passed |
| `test_15_swar_in_place_crypto_and_benchmark` | SWAR parallel in-place encryption and decryption | 1,000 round-trips (10,000 quats each) | `58.1793ms` | Passed |
| `test_16_gf4_field_arithmetic_and_benchmark` | Galois field $GF(4)$ inverse, multiplication, and distributivity | 100,000 iterations | `9.4204ms` | Passed |
| `test_17_constant_time_sbox_and_benchmark` | `ConstantTimeQuatSBox` permutation validation, `sub()`, and `inv_sub()` | 100,000 iterations | `19.5609ms` | Passed |
| `test_18_lfsr_and_feistel_cipher_benchmark` | `QuatLfsr` keystream generation and `QuatFeistelCipher` encrypt/decrypt | 2,000 iterations (64-quat block) | `50.8466ms` | Passed |
| `test_19_dna_and_pam4_conversions_benchmark` | DNA string (`ACGT`) and Gray-coded PAM4 symbol conversions | 10,000 iterations | `32.3891ms` | Passed |
| `test_20_serde_serialization_benchmark` | `serde` feature validation and buffer serialization/deserialization check | 10,000 iterations | `2.1531ms` | Passed |

---

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `quat_crypto` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
quat_crypto = "0.5.0"
```

---

## 💡 Quick Start Guide

### 1. Creating Buffers and Using the Macro
You can create quaternary buffers dynamically or use the provided `quat_vec!` macro:

```rust
use quat_crypto::{Quat, QuatBuffer, quat_vec};

fn main() {
    // Using the macro
    let mut buf = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11];

    assert_eq!(buf.len(), 4);
    
    // 4 quats fit into exactly 1 physical byte! (75% memory saved)
    assert_eq!(buf.byte_len(), 1);

    // Retrieve elements safely
    assert_eq!(buf.get(2), Ok(Quat::V10));
}
```

### 2. Zero-Copy Slicing & Iteration
Slice through large buffers instantly without duplicating underlying memory:

```rust
use quat_crypto::{Quat, quat_vec};

fn main() {
    let buf = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11, Quat::V01];

    // Create a zero-copy slice from index 1 to 4
    let slice = buf.slice(1, 4).unwrap();
    assert_eq!(slice.len(), 3);

    // Iterate over slice elements
    for quat in slice.iter() {
        println!("Quat: {}", quat);
    }
}
```

### 3. Encryption and Decryption
Encrypt and decrypt streams of quats using quaternary modular addition (`mod 4`):

```rust
use quat_crypto::{Quat, QuatStreamCipher, Cipher, quat_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a secret key sequence of quats
    let key = vec![Quat::V11, Quat::V01];
    let cipher = QuatStreamCipher::new(key)?;

    // Plaintext data
    let plaintext = quat_vec![Quat::V00, Quat::V01, Quat::V10, Quat::V11];

    // Encrypt
    let ciphertext = cipher.encrypt(&plaintext)?;
    
    // Decrypt back to original
    let decrypted = cipher.decrypt(&ciphertext)?;

    assert_eq!(plaintext.get(0), decrypted.get(0));
    Ok(())
}
```

### 4. Arithmetic & Logical Operators
Perform direct modular arithmetic ($\pmod 4$) and bitwise logic on quats:

```rust
use quat_crypto::Quat;

fn main() {
    // Addition mod 4 (1 + 2 = 3)
    let sum = Quat::V01 + Quat::V10;
    assert_eq!(sum, Quat::V11);

    // Subtraction mod 4 (3 - 1 = 2)
    let diff = Quat::V11 - Quat::V01;
    assert_eq!(diff, Quat::V10);

    // Bitwise NOT
    let inverted = !Quat::V01;
    assert_eq!(inverted, Quat::V10);
}
```

### 5. Substitution Cipher & Raw Bytes Conversion

```rust
use quat_crypto::{Quat, QuatBuffer, QuatSubstitutionCipher, quat_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // S-Box substitution
    let sbox = QuatSubstitutionCipher::new([Quat::V11, Quat::V00, Quat::V01, Quat::V10]);
    let input = quat_vec![Quat::V00, Quat::V01];
    let substituted = sbox.substitute(&input)?;

    // Converting raw standard bytes (&[u8]) into QuatBuffer
    let raw_bytes: &[u8] = &[0b11100100]; // Packs multiple quats
    let buffer = QuatBuffer::try_from(raw_bytes)?;
    
    Ok(())
}
```

### 6. Mutable Zero-Copy Slicing & SWAR In-Place Crypto
Modify packed buffers in-place without allocations and run parallel SWAR quaternary arithmetic:

```rust
use quat_crypto::{Quat, QuatStreamCipher, quat_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = quat_vec![
        Quat::V00, Quat::V01, Quat::V10, Quat::V11,
        Quat::V00, Quat::V01, Quat::V10, Quat::V11
    ];

    // Zero-copy mutable sub-slice
    let mut sub = buf.slice_mut(1, 5)?;
    sub.set(0, Quat::V11)?;

    // In-place SWAR encryption/decryption
    let key = vec![Quat::V01, Quat::V10, Quat::V11, Quat::V00];
    let cipher = QuatStreamCipher::new(key)?;
    cipher.encrypt_in_place_swar(&mut buf);
    cipher.decrypt_in_place_swar(&mut buf);

    Ok(())
}
```

### 7. GF(4) Field Arithmetic, LFSR & Feistel Cipher
Use finite field $GF(4)$ operations, constant-time S-Boxes, LFSR keystreams, and multi-round Feistel encryption:

```rust
use quat_crypto::{
    Cipher, ConstantTimeQuatSBox, Quat, QuatBuffer, QuatFeistelCipher, QuatLfsr,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prod = Quat::V10.gf4_mul(Quat::V11);
    let _inv = prod.gf4_inv();

    let mut lfsr = QuatLfsr::new(
        vec![Quat::V01, Quat::V10, Quat::V11, Quat::V01],
        vec![Quat::V11, Quat::V00, Quat::V01, Quat::V10],
    )?;
    let block: QuatBuffer = lfsr.generate_keystream(8);

    let sbox = ConstantTimeQuatSBox::new([Quat::V11, Quat::V00, Quat::V01, Quat::V10])?;
    let feistel = QuatFeistelCipher::new(
        vec![
            vec![Quat::V10, Quat::V01, Quat::V11, Quat::V00],
            vec![Quat::V01, Quat::V11, Quat::V00, Quat::V10],
        ],
        sbox,
    )?;

    let enc = feistel.encrypt(&block)?;
    let dec = feistel.decrypt(&enc)?;
    assert_eq!(dec.as_bytes(), block.as_bytes());

    Ok(())
}
```

### 8. DNA & PAM4 Conversions
Encode and decode DNA sequences (`A`, `C`, `G`, `T`) and Gray-coded PAM4 levels (`-3`, `-1`, `+1`, `+3`):

```rust
use quat_crypto::QuatBuffer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dna_buf = QuatBuffer::from_dna_str("ACGTACGT")?;
    assert_eq!(dna_buf.byte_len(), 2);
    assert_eq!(dna_buf.to_dna_string(), "ACGTACGT");

    let pam4 = dna_buf.to_pam4_symbols();
    let restored = QuatBuffer::from_pam4_symbols(&pam4)?;
    assert_eq!(restored.as_bytes(), dna_buf.as_bytes());

    Ok(())
}
```

---

## 🧠 How the 75% Memory Saving Works

In standard Rust, storing a small enum or state ranging from `0` to `3` typically consumes a full `u8` (8 bits), leaving **6 bits unused** per item (75% wasted memory). 

`quat_crypto` solves this by packing **four 2-bit values** tightly into a single `u8` container:

```text
Standard u8 vector (4 items):  [ 00000011 ] [ 00000001 ] [ 00000010 ] [ 00000000 ]  (4 bytes used)
QuatBuffer (4 items packed):   [ 11011000 ]                                         (1 byte used - 75% saved!)
```

---

## 🧪 Running Tests

To run the full suite of unit tests (ranging from basic conversions to massive 10,000-item stress tests), run:

```bash
cargo test
```