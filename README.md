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

---

## Benchmark

| Test Name | Tested Component / Feature | Iterations / Scale | Execution Time | Status |
| :--- | :--- | :--- | :--- | :--- |
| **`test_01_quat_basic_values`** | Basic values & u8 conversion | 100,000 iterations | 2.0000 ms[cite: 19] | Passed[cite: 19] |
| **`test_02_quat_try_from_invalid`** | Validation of invalid values (`TryFrom`) | 100,000 iterations | 2.7028 ms[cite: 19] | Passed[cite: 19] |
| **`test_03_buffer_push_and_get`** | Basic buffer push and get operations | 10,000 iterations | 2.6369 ms[cite: 19] | Passed[cite: 19] |
| **`test_04_buffer_memory_packing`** | Memory saving / bit-packing (4 quats per byte) | 10,000 iterations | 2.6568 ms[cite: 19] | Passed[cite: 19] |
| **`test_05_quat_vec_macro`** | Custom `quat_vec!` macro evaluation | 10,000 iterations | 3.2729 ms[cite: 19] | Passed[cite: 19] |
| **`test_06_buffer_iteration`** | Buffer iteration (owned & borrowed) | 10,000 iterations | 6.6639 ms[cite: 19] | Passed[cite: 19] |
| **`test_07_buffer_slicing`** | Zero-copy slicing over buffer | 10,000 iterations | 3.4257 ms[cite: 19] | Passed[cite: 19] |
| **`test_08_cipher_encrypt_decrypt`** | Stream cipher encryption & decryption symmetry | 5,000 iterations | 7.0849 ms[cite: 19] | Passed[cite: 19] |
| **`test_09_error_edge_cases`** | Error handling (bounds, empty key, range) | 10,000 iterations | 1.9618 ms[cite: 19] | Passed[cite: 19] |
| **`test_10_stress_test_large_buffer`** | Large buffer stress test (10,000 quats + crypto) | 1 run (10,000 quats) | 2.8394 ms[cite: 19] | Passed[cite: 19] |
| **`test_11_quat_arithmetic_and_logic`** | Arithmetic (`+`, `-`) and logic (`&`, ``, `^`, `!`) | Standard unit test | N/A | Passed[cite: 19] |
| **`test_12_substitution_cipher`** | Quat Substitution Cipher (S-Box) | 10,000 iterations | 4.0351 ms[cite: 19] | Passed[cite: 19] |
| **`test_13_raw_bytes_conversion`** | `QuatBuffer TryFrom<&[u8]>` raw byte parsing | 5,000 iterations (400 quats each) | 76.7261 ms[cite: 19] | Passed[cite: 19] |

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
quat_crypto = "0.4.0"
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