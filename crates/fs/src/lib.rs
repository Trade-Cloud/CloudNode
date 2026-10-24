//! CloudNode files: where things live on the user's machine, and writing files so that a crash
//! never leaves half of one.
//!
//! The vault, the keys in `~/.ssh` and the SSH client config all use this, so it sits at the
//! bottom (L0) and depends on no other workspace crate.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod atomic;

use std::fs;
use std::io;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

/// The user's `~/.ssh` (on Windows `%USERPROFILE%\.ssh`, where Windows' OpenSSH keeps its files).
pub fn home_ssh_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ssh"))
}

/// `path` with the home folder written as `~`, as people read and type it: `~/.ssh`.
pub fn tilde(path: &Path) -> String {
    match dirs::home_dir().as_deref().and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~{MAIN_SEPARATOR}{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// Makes `dir` (and its parents) if it doesn't exist. A folder this makes is readable by its
/// owner only on Unix (`0700`), as OpenSSH expects of `~/.ssh`; an existing one is left as it is.
pub fn ensure_private_dir(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_home_folder_reads_as_a_tilde() {
        if let Some(home) = dirs::home_dir() {
            assert_eq!(tilde(&home), "~");
            assert_eq!(tilde(&home.join(".ssh")), format!("~{MAIN_SEPARATOR}.ssh"));
        }
        let elsewhere = Path::new("/definitely/not/home");
        assert_eq!(tilde(elsewhere), elsewhere.display().to_string());
    }

    #[test]
    fn makes_a_private_folder_once() {
        let dir = std::env::temp_dir().join(format!("cloudnode-fs-{}-private", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let nested = dir.join("a").join(".ssh");
        ensure_private_dir(&nested).unwrap();
        assert!(nested.is_dir());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&nested).unwrap().permissions().mode() & 0o777, 0o700);
        }
        ensure_private_dir(&nested).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}
