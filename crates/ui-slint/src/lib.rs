//! CloudNode's desktop interface, in Slint.
//!
//! The `.slint` files in `ui/` hold the layout. This crate compiles them, fills their `Theme`
//! global from [`cloudnode_core::Tokens`] and runs the window. It stays thin: what it shows comes
//! from the crates below it, and the app shell (`apps/cloudnode`) owns start-up, logging and exit.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod theme;

use slint::ComponentHandle;

/// The code Slint generates from `ui/`. It is not ours to lint and may contain `unsafe`, which is
/// why this crate denies `unsafe_code` instead of forbidding it (see Cargo.toml). Nothing else
/// here allows it. `clippy::restriction` covers the never-crash lints denied above.
#[allow(unsafe_code, unused, clippy::all, clippy::restriction)]
mod generated {
    slint::include_modules!();
}

pub use generated::{MainWindow, Theme};
pub use slint::PlatformError;

/// Opens the main window and runs until it is closed.
pub fn run() -> Result<(), PlatformError> {
    let window = MainWindow::new()?;
    theme::apply(&window, &cloudnode_core::Tokens::DARK);
    window.run()
}
