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
quat_crypto = "0.3.0"
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