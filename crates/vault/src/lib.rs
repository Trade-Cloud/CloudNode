//! CloudNode vault: the encrypted store on the user's disk.
//!
//! Everything CloudNode keeps (hosts, groups, private keys, saved passwords, settings) lives in
//! one vault file in the user's data folder. Nothing is ever sent anywhere: there is no sync, no
//! account and no telemetry.
//!
//! Design (milestone M1 in `ROADMAP.md`):
//!
//! - The master password is stretched with Argon2id (RustCrypto `argon2`) into a 256-bit key. The
//!   salt and the cost parameters are stored in the file header, so the cost can grow later.
//! - The payload is sealed with XChaCha20-Poly1305 (RustCrypto `chacha20poly1305`, pure Rust)
//!   under a fresh random 192-bit nonce on every save. The header is authenticated as associated
//!   data, so a tampered header fails to open instead of being trusted.
//! - Keys and decrypted secrets live in `zeroize`-on-drop buffers and are never logged or shown
//!   in `Debug` output.
//! - Every save goes through `cloudnode_fs::atomic::write`: a crash or a full disk never leaves
//!   half a vault, and on Unix the file is readable by its owner only.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
