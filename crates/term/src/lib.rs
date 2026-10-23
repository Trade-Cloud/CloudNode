//! CloudNode terminal emulation: turning the bytes a server sends into a grid of cells.
//!
//! Design (milestone M3 in `ROADMAP.md`): the escape-sequence parser and the grid come from
//! `alacritty_terminal` (Apache-2.0), wrapped so that nothing above this crate depends on it. The
//! GUI draws the grid into a pixel buffer, the TUI into Ratatui cells; both read the same
//! [`cloudnode_core::TerminalPalette`].

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod size;

pub use size::Size;
