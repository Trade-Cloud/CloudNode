//! CloudNode core: the data every part of CloudNode shares.
//!
//! Pure data and logic, no I/O: saved hosts and their groups, descriptions of the keys in Key
//! Management, and the colour tokens. The vault stores these types, the SSH layer reads them and
//! both interfaces show them, so this crate sits at the bottom and depends on no other workspace
//! crate.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod host;
pub mod key;
pub mod theme;
pub mod time;

pub use host::{Auth, Group, GroupId, Host, HostError, HostId};
pub use key::{KeyAlgorithm, KeyId, KeyInfo};
pub use theme::{Rgba, TerminalPalette, Tokens};
