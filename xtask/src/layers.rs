//! `cargo xtask layers`: a crate may depend only on crates in lower layers, and nothing below the
//! interfaces may pull in a UI toolkit.
//!
//! | Layer | Crates | May use |
//! |---|---|---|
//! | L0 | `core` | no workspace crate |
//! | L1 | `vault`, `keys`, `ssh`, `term` | L0 |
//! | L2 | `ui-slint`, `ui-tui` | L0, L1, UI toolkits |
//! | apps | `apps/*` | everything |
//!
//! A new crate under `crates/` must be added to [`LAYERS`], or this check fails.

use std::fs;
use std::path::Path;

/// Each crate folder under `crates/` and its layer.
const LAYERS: &[(&str, u8)] = &[("core", 0), ("vault", 1), ("keys", 1), ("ssh", 1), ("term", 1), ("ui-slint", 2), ("ui-tui", 2)];

/// The layer of the interfaces, the only one allowed UI toolkits.
const UI_LAYER: u8 = 2;

/// Dependencies that open windows or take over the console.
const UI_TOOLKITS: &[&str] = &["slint", "slint-build", "ratatui", "crossterm", "winit"];

/// Workspace crates are named `cloudnode-<folder>`.
const PREFIX: &str = "cloudnode-";

pub fn run(root: &Path) -> Result<(), String> {
    let crates_dir = root.join("crates");
    let entries = fs::read_dir(&crates_dir).map_err(|e| format!("{}: {e}", crates_dir.display()))?;
    let mut problems = Vec::new();
    let mut checked = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", crates_dir.display()))?;
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let dir = entry.file_name().to_string_lossy().into_owned();
        let text = fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
        problems.extend(check_crate(&dir, &text));
        checked += 1;
    }
    if problems.is_empty() {
        eprintln!("layers: {checked} crates OK");
        Ok(())
    } else {
        Err(format!("layering broken:\n  {}", problems.join("\n  ")))
    }
}

/// What is wrong with one crate's dependencies, if anything.
fn check_crate(dir: &str, manifest: &str) -> Vec<String> {
    let Some(layer) = layer_of(dir) else {
        return vec![format!("crates/{dir} has no layer: add it to LAYERS in xtask/src/layers.rs")];
    };
    let mut problems = Vec::new();
    for dep in dependencies(manifest) {
        if let Some(dep_dir) = dep.strip_prefix(PREFIX) {
            match layer_of(dep_dir) {
                Some(dep_layer) if dep_layer < layer => {}
                Some(dep_layer) => problems.push(format!("crates/{dir} (L{layer}) uses {dep} (L{dep_layer}): only lower layers")),
                None => problems.push(format!("crates/{dir} uses {dep}, which has no layer")),
            }
        } else if layer < UI_LAYER && UI_TOOLKITS.contains(&dep.as_str()) {
            problems.push(format!("crates/{dir} (L{layer}) uses {dep}: UI toolkits belong in L{UI_LAYER}"));
        }
    }
    problems
}

fn layer_of(dir: &str) -> Option<u8> {
    LAYERS.iter().find(|(name, _)| *name == dir).map(|&(_, layer)| layer)
}

/// The names of the normal and build dependencies in a manifest: `[dependencies]`,
/// `[build-dependencies]` and their `[target.….dependencies]` forms, but not dev-dependencies,
/// which tests may use freely. A line-based reading of TOML, enough for this workspace's
/// manifests: one dependency per line, `name = …` or `name.workspace = true`.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            let section = line.trim_matches(['[', ']']);
            in_deps = section.ends_with("dependencies") && !section.ends_with("dev-dependencies");
            continue;
        }
        if !in_deps || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.split(|c: char| c == '=' || c == '.' || c.is_whitespace()).next().unwrap_or("");
        if !name.is_empty() {
            deps.push(name.trim_matches('"').to_string());
        }
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[package]
name = "cloudnode-ui-slint"

[dependencies]
cloudnode-core = { workspace = true }
slint.workspace = true
# a comment

[target.'cfg(windows)'.dependencies]
winreg = "0.55"

[dev-dependencies]
cloudnode-ui-tui = { workspace = true }

[build-dependencies]
slint-build = { workspace = true }
"#;

    #[test]
    fn reads_normal_target_and_build_dependencies() {
        assert_eq!(dependencies(MANIFEST), ["cloudnode-core", "slint", "winreg", "slint-build"]);
    }

    #[test]
    fn the_ui_layer_may_use_lower_layers_and_toolkits() {
        assert!(check_crate("ui-slint", MANIFEST).is_empty());
    }

    #[test]
    fn lower_layers_may_not_use_toolkits_or_peers() {
        let manifest = "[dependencies]\ncloudnode-term = { workspace = true }\nratatui = \"0.30\"\n";
        assert_eq!(check_crate("ssh", manifest).len(), 2);
        assert_eq!(check_crate("core", "[dependencies]\ncloudnode-vault = { workspace = true }\n").len(), 1);
    }

    #[test]
    fn unregistered_crates_fail() {
        assert_eq!(check_crate("new-thing", "").len(), 1);
    }
}
