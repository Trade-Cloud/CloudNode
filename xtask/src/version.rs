//! `cargo xtask version [set X.Y.Z[-pre]]`. The version lives in one place only:
//! `[workspace.package] version` in the root `Cargo.toml`.

use std::fs;
use std::path::Path;

const NOT_FOUND: &str = "no `version = \"…\"` line in [workspace.package]";

pub fn run(root: &Path, args: &[&str]) -> Result<(), String> {
    let manifest = root.join("Cargo.toml");
    let text = fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    match args {
        [] => {
            let (_, version) = find(&text).ok_or(NOT_FOUND)?;
            println!("{version}");
            Ok(())
        }
        ["set", new] => {
            check(new)?;
            let updated = replace(&text, new).ok_or(NOT_FOUND)?;
            fs::write(&manifest, updated).map_err(|e| format!("{}: {e}", manifest.display()))?;
            println!("version set to {new}; the next cargo build updates Cargo.lock");
            Ok(())
        }
        _ => Err("usage: cargo xtask version [set X.Y.Z[-pre]]".into()),
    }
}

/// The line index and value of `version = "…"` in `[workspace.package]`.
fn find(text: &str) -> Option<(usize, &str)> {
    let mut in_package = false;
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[workspace.package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version") {
            let value = rest.trim_start().strip_prefix('=')?.trim();
            return Some((i, value.trim_matches('"')));
        }
    }
    None
}

/// `text` with the workspace version set to `new`.
fn replace(text: &str, new: &str) -> Option<String> {
    let (index, _) = find(text)?;
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    *lines.get_mut(index)? = format!("version = \"{new}\"");
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// `X.Y.Z`, optionally followed by `-` and a pre-release tag (`0.2.0-rc.1`).
fn check(version: &str) -> Result<(), String> {
    let (core, pre) = version.split_once('-').map_or((version, None), |(core, pre)| (core, Some(pre)));
    let parts: Vec<&str> = core.split('.').collect();
    let core_ok = parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let pre_ok = pre.is_none_or(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'));
    if core_ok && pre_ok { Ok(()) } else { Err(format!("`{version}` is not X.Y.Z or X.Y.Z-pre")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.package]\nversion = \"0.1.0\"\nrust-version = \"1.90\"\n";

    #[test]
    fn finds_the_workspace_version() {
        assert_eq!(find(MANIFEST), Some((4, "0.1.0")));
        assert_eq!(find("[package]\nversion = \"9.9.9\"\n"), None);
    }

    #[test]
    fn replaces_only_that_line() {
        let out = replace(MANIFEST, "0.2.0-rc.1").unwrap();
        assert_eq!(find(&out), Some((4, "0.2.0-rc.1")));
        assert!(out.contains("rust-version = \"1.90\""));
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn checks_the_format() {
        for ok in ["0.1.0", "1.20.300", "0.2.0-rc.1", "1.0.0-beta-2"] {
            assert_eq!(check(ok), Ok(()), "{ok}");
        }
        for bad in ["", "1.0", "1.0.0.0", "1.a.0", "1.0.0-", "v1.0.0", "1.0.0-rc 1"] {
            assert!(check(bad).is_err(), "{bad}");
        }
    }
}
