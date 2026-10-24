//! Writing a file so that it is either the old one or the new one, never half of either.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Writes `bytes` to `path` through a temporary file beside it and a rename, so a crash, a power
/// cut or a full disk leaves the previous file untouched. The data reaches the disk (`sync_all`)
/// before the rename. On Unix the file is created readable and writable by its owner only
/// (`0600`), since it may hold secrets.
///
/// When `path` is a symbolic link (dotfile managers link `~/.ssh/config` into a repository), the
/// file it points at is written and the link stays. The parent folder must exist.
pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let target = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path)?,
        _ => path.to_path_buf(),
    };
    let tmp = temp_path(&target)?;
    if let Err(e) = write_new(&tmp, bytes) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    // On Windows `rename` replaces an existing file too (MoveFileExW with MOVEFILE_REPLACE_EXISTING).
    if let Err(e) = fs::rename(&tmp, &target) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// `<name>.tmp` in the same folder, so the rename never crosses file systems.
fn temp_path(path: &Path) -> io::Result<PathBuf> {
    let Some(name) = path.file_name() else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{} has no file name", path.display())));
    };
    let mut tmp = name.to_os_string();
    tmp.push(".tmp");
    Ok(path.with_file_name(tmp))
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder for one test, removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("cloudnode-fs-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn writes_and_replaces() {
        let dir = TempDir::new("replace");
        let path = dir.0.join("vault.cnv");
        write(&path, b"first").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"first");
        write(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
    }

    #[test]
    fn leaves_no_temporary_file() {
        let dir = TempDir::new("no-temp");
        let path = dir.0.join("vault.cnv");
        write(&path, b"data").unwrap();
        let names: Vec<_> = fs::read_dir(&dir.0).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("vault.cnv")]);
    }

    #[test]
    fn missing_folder_is_an_error_not_a_panic() {
        let dir = TempDir::new("missing");
        assert!(write(&dir.0.join("nope").join("vault.cnv"), b"data").is_err());
    }

    #[test]
    fn a_path_without_a_file_name_is_rejected() {
        assert_eq!(write(Path::new("/"), b"data").map_err(|e| e.kind()), Err(io::ErrorKind::InvalidInput));
    }

    #[cfg(unix)]
    #[test]
    fn writes_through_a_symlink_and_keeps_it() {
        let dir = TempDir::new("symlink");
        let real = dir.0.join("dotfiles-config");
        let link = dir.0.join("config");
        fs::write(&real, b"old").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        write(&link, b"new").unwrap();
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        assert_eq!(fs::read(&real).unwrap(), b"new");
    }

    #[cfg(unix)]
    #[test]
    fn only_the_owner_can_read_it() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new("mode");
        let path = dir.0.join("vault.cnv");
        write(&path, b"secret").unwrap();
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }
}
