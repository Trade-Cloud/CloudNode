//! CloudNode keys: the SSH keys on the user's machine.
//!
//! For now this finds the keys in the user's `~/.ssh` folder so Key Management can list them
//! ([`scan_home`]). It reads public key files (`*.pub`) in full, and of every other file only its
//! first few bytes, to tell whether it is a private key. **Private key contents are never read
//! here.** Generating, importing and exporting keys come later (M4 in `ROADMAP.md`).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod local;

pub use local::{Details, LocalKey, PublicDetails, Scan, ScanError, home_ssh_dir, parse_public_key, scan_dir, scan_home, tilde};
