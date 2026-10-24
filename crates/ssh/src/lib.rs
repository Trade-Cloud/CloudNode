//! CloudNode SSH: connecting to a [`cloudnode_core::Host`] and running its shell.
//!
//! Today this crate reads and adds to the user's SSH client config ([`config`]), which is where
//! Home's server list comes from until the vault holds servers.
//!
//! Sessions, the design (milestone M2 in `ROADMAP.md`):
//!
//! - The protocol is `russh` (pure Rust, Apache-2.0) on the Tokio runtime, at 0.62.5 or newer
//!   (earlier versions have published security fixes). No libssh2 or OpenSSL.
//! - Key parsing and generation go through `russh::keys` (the RustCrypto `ssh-key` stack):
//!   Ed25519 first, then ECDSA P-256 and RSA.
//! - Host keys are checked against CloudNode's own `known_hosts` in the vault. An unknown key asks
//!   the user with its SHA-256 fingerprint; a changed key refuses to connect until the user
//!   decides. Nothing is accepted silently.
//! - The server is untrusted input: every length, size and string it sends is bounded before use,
//!   and nothing it sends can panic the client.
//! - One session per tab, each with its own PTY channel; window resizes reach the server as
//!   `window-change` requests.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod config;
