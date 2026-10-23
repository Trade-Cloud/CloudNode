//! CloudNode, the desktop app.
//!
//! A thin shell: it sets up logging, then hands over to the Slint interface (`cloudnode-ui-slint`)
//! until the window closes. The console interface is the separate `cloudnode-tui` binary, so this
//! one can be a windowed app on Windows, with no console window flashing up.

#![cfg_attr(windows, windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::process::ExitCode;

/// Log levels when `RUST_LOG` is unset: CloudNode's own crates at `info` (every crate name starts
/// with `cloudnode`), everything else at `warn`.
const DEFAULT_LOG_FILTER: &str = "warn,cloudnode=info";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("cloudnode {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    // A second logger can't be installed; that is the only way this fails, so it is ignored.
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER)).try_init();
    log::info!("CloudNode {}", env!("CARGO_PKG_VERSION"));

    match cloudnode_ui_slint::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error!("could not open the window: {e}");
            ExitCode::FAILURE
        }
    }
}
