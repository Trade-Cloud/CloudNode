//! Workspace tooling: `cargo xtask <command>`.
//!
//! Pure Rust, std only. Cargo is run through `std::process::Command`.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod layers;
mod version;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  ci              fmt --check, clippy -D warnings, tests, layers (stops at the first failure)
  layers          check that every crate depends only on the layers below it
  version [set X.Y.Z[-pre]]
                  print the workspace version, or set it in Cargo.toml
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let result = match args.first().map(String::as_str) {
        Some("ci") => cmd_ci(),
        Some("layers") => layers::run(&root()),
        Some("version") => version::run(&root(), &rest),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root: the parent of the xtask crate.
fn root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir).to_path_buf()
}

/// Runs `cargo <args>` in the workspace root and fails when it does.
fn cargo_step(args: &[&str]) -> Result<(), String> {
    let what = format!("cargo {}", args.join(" "));
    eprintln!("$ {what}");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo).args(args).current_dir(root()).status().map_err(|e| format!("{what}: could not start: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("{what}: exited with {status}")) }
}

fn cmd_ci() -> Result<(), String> {
    cargo_step(&["fmt", "--all", "--", "--check"])?;
    cargo_step(&["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    cargo_step(&["test", "--workspace"])?;
    layers::run(&root())?;
    eprintln!("ci: all green");
    Ok(())
}
