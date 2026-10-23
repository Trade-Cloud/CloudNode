//! CloudNode in the console: the same hosts, keys and sessions as the desktop app, drawn with
//! Ratatui (`cloudnode-ui-tui`).
//!
//! Nothing is logged to the console while the interface owns it, since that would tear the
//! screen. Logging to a file in CloudNode's data folder comes with the platform work (M4 in
//! ROADMAP.md).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("cloudnode-tui {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    match cloudnode_ui_tui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cloudnode-tui: {e}");
            ExitCode::FAILURE
        }
    }
}
