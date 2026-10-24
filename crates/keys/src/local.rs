//! The keys in a folder of OpenSSH key files, normally `~/.ssh`.
//!
//! A key is a pair of files named like `ssh-keygen` names them: the private key (`id_ed25519`)
//! and the public key beside it (`id_ed25519.pub`). Either may be missing. Everything here is
//! read-only, and the folder's contents are treated as untrusted: sizes are capped, unreadable or
//! odd files are reported or skipped, and nothing panics.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use cloudnode_core::KeyAlgorithm;
use ssh_key::{HashAlg, PublicKey};

/// Public key files bigger than this are not public keys (a 16384-bit RSA key is under 3 KiB).
const MAX_PUBLIC_KEY_BYTES: u64 = 64 * 1024;
/// Folder entries looked at. `~/.ssh` holds a handful of files; this keeps an odd one from
/// stalling the scan.
const MAX_ENTRIES: usize = 1024;
/// Bytes read from the start of a file to see whether it is a private key.
const MAX_HEADER_BYTES: u64 = 64;
/// Files OpenSSH keeps in `~/.ssh` that are never keys.
const NOT_KEYS: &[&str] = &["config", "known_hosts", "known_hosts.old", "authorized_keys", "authorized_keys2", "environment", "rc"];

/// What a scan of one folder found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scan {
    /// The folder that was read.
    pub dir: PathBuf,
    /// The keys, sorted by name.
    pub keys: Vec<LocalKey>,
}

/// One key: a private key file, a public key file, or both, sharing a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalKey {
    /// The private key's file name, which is also the public key's without `.pub`: `id_ed25519`.
    pub name: String,
    /// The public key file, when there is one.
    pub public_file: Option<PathBuf>,
    /// The private key file, when there is one.
    pub private_file: Option<PathBuf>,
    pub details: Details,
    /// When the key was made: the private key file's creation time (its last change where the
    /// file system keeps no creation time), else the public key file's.
    pub created: Option<SystemTime>,
}

/// What can be shown about a key without touching its private half.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Details {
    /// Read from the public key file.
    Public(PublicDetails),
    /// There is only a private key file. Its details stay unread until the key is used.
    PrivateOnly,
    /// The public key file can't be read or isn't a public key: the reason, for the user.
    Unreadable(String),
}

/// What a public key file says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicDetails {
    pub algorithm: KeyAlgorithm,
    /// The SHA-256 fingerprint as `ssh-keygen -l` prints it: `SHA256:…`.
    pub fingerprint: String,
    /// Often `user@machine`; empty when the key has none.
    pub comment: String,
}

/// Why a folder couldn't be scanned.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("this system has no home folder")]
    NoHome,
    #[error("could not read {}: {source}", .path.display())]
    Folder { path: PathBuf, source: io::Error },
}

/// The keys in the user's `~/.ssh`. A missing folder means no keys, not an error.
pub fn scan_home() -> Result<Scan, ScanError> {
    let dir = cloudnode_fs::home_ssh_dir().ok_or(ScanError::NoHome)?;
    scan_dir(&dir)
}

/// The keys in `dir`. A missing folder means no keys, not an error; files that vanish or can't
/// be opened while scanning are skipped.
pub fn scan_dir(dir: &Path) -> Result<Scan, ScanError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Scan { dir: dir.to_path_buf(), keys: Vec::new() }),
        Err(source) => return Err(ScanError::Folder { path: dir.to_path_buf(), source }),
    };

    let mut pairs: BTreeMap<String, Pair> = BTreeMap::new();
    for entry in entries.take(MAX_ENTRIES) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        // `metadata` follows symlinks: dotfile managers often link keys into ~/.ssh.
        let Ok(metadata) = fs::metadata(&path) else { continue };
        if !metadata.is_file() {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()).map(str::to_owned) else { continue };
        if file_name.starts_with('.') {
            continue;
        }
        let made = metadata.created().or_else(|_| metadata.modified()).ok();
        if let Some(name) = file_name.strip_suffix(".pub") {
            // `*-cert.pub` files are certificates for a key, not keys of their own.
            if !name.is_empty() && !name.ends_with("-cert") {
                let pair = pairs.entry(name.to_owned()).or_default();
                pair.public = Some(path);
                pair.public_made = made;
            }
        } else if !NOT_KEYS.contains(&file_name.as_str()) && looks_like_private_key(&path) {
            let pair = pairs.entry(file_name).or_default();
            pair.private = Some(path);
            pair.private_made = made;
        }
    }

    let keys = pairs
        .into_iter()
        .map(|(name, pair)| {
            let details = describe(pair.public.as_deref());
            let created = pair.private_made.or(pair.public_made);
            LocalKey { name, public_file: pair.public, private_file: pair.private, details, created }
        })
        .collect();
    Ok(Scan { dir: dir.to_path_buf(), keys })
}

/// The two files of one key, as the scan finds them.
#[derive(Default)]
struct Pair {
    public: Option<PathBuf>,
    private: Option<PathBuf>,
    public_made: Option<SystemTime>,
    private_made: Option<SystemTime>,
}

/// Whether a file starts like a private key: OpenSSH's own format, the older PEM ones (RSA, EC,
/// DSA, PKCS#8, encrypted or not), or PuTTY's. Only the first [`MAX_HEADER_BYTES`] are read.
fn looks_like_private_key(path: &Path) -> bool {
    let Ok(file) = File::open(path) else { return false };
    let mut head = Vec::new();
    if file.take(MAX_HEADER_BYTES).read_to_end(&mut head).is_err() {
        return false;
    }
    let head = String::from_utf8_lossy(&head);
    let first_line = head.trim_start().lines().next().unwrap_or("");
    (first_line.starts_with("-----BEGIN ") && first_line.contains("PRIVATE KEY-----")) || first_line.starts_with("PuTTY-User-Key-File-")
}

fn describe(public_file: Option<&Path>) -> Details {
    let Some(path) = public_file else { return Details::PrivateOnly };
    match read_public_key(path) {
        Ok(details) => Details::Public(details),
        Err(reason) => Details::Unreadable(reason),
    }
}

fn read_public_key(path: &Path) -> Result<PublicDetails, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(MAX_PUBLIC_KEY_BYTES + 1).read_to_string(&mut text).map_err(|e| e.to_string())?;
    if text.len() as u64 > MAX_PUBLIC_KEY_BYTES {
        return Err("the file is too large to be a public key".into());
    }
    parse_public_key(&text)
}

/// Reads an OpenSSH public key (`<type> <base64> [comment]`, as in `*.pub` files): its first line
/// that isn't blank or a `#` comment. The error is the reason, for the user.
pub fn parse_public_key(text: &str) -> Result<PublicDetails, String> {
    let line = text.lines().map(str::trim).find(|line| !line.is_empty() && !line.starts_with('#')).ok_or("the file is empty")?;
    let key = PublicKey::from_openssh(line).map_err(|e| format!("not an OpenSSH public key ({e})"))?;
    let Some(algorithm) = algorithm(&key) else {
        return Err(format!("unsupported key type {}", key.algorithm().as_str()));
    };
    Ok(PublicDetails { algorithm, fingerprint: key.fingerprint(HashAlg::Sha256).to_string(), comment: key.comment().to_owned() })
}

/// The key's algorithm; `None` for one CloudNode doesn't know.
fn algorithm(key: &PublicKey) -> Option<KeyAlgorithm> {
    algorithm_named(key.algorithm().as_str(), || rsa_bits(key))
}

/// The algorithm OpenSSH calls `name` (the inverse of [`KeyAlgorithm::ssh_name`]). `rsa_bits`
/// is asked only for an RSA key.
fn algorithm_named(name: &str, rsa_bits: impl FnOnce() -> u16) -> Option<KeyAlgorithm> {
    let algorithm = match name {
        "ssh-ed25519" => KeyAlgorithm::Ed25519,
        "ssh-rsa" => KeyAlgorithm::Rsa { bits: rsa_bits() },
        "ecdsa-sha2-nistp256" => KeyAlgorithm::EcdsaP256,
        "ecdsa-sha2-nistp384" => KeyAlgorithm::EcdsaP384,
        "ecdsa-sha2-nistp521" => KeyAlgorithm::EcdsaP521,
        "sk-ssh-ed25519@openssh.com" => KeyAlgorithm::SkEd25519,
        "sk-ecdsa-sha2-nistp256@openssh.com" => KeyAlgorithm::SkEcdsaP256,
        "ssh-dss" => KeyAlgorithm::Dsa,
        _ => return None,
    };
    Some(algorithm)
}

/// The size of an RSA key's modulus in bits; 0 if the key holds no RSA modulus.
fn rsa_bits(key: &PublicKey) -> u16 {
    let Some(modulus) = key.key_data().rsa().and_then(|rsa| rsa.n.as_positive_bytes()) else { return 0 };
    let Some(first) = modulus.first() else { return 0 };
    let bits = modulus.len().saturating_mul(8).saturating_sub(first.leading_zeros() as usize);
    u16::try_from(bits).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Throwaway keys made with `ssh-keygen` for these tests; the fingerprints are what
    // `ssh-keygen -l -E sha256` printed for them.
    const ED25519: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICu6Gn4Igz4lmP1DPivf9DFJLKdDKrE3V62x2i3n+rWt ada@cloudnode.test\n";
    const ED25519_FINGERPRINT: &str = "SHA256:TzlVtMBmDRzEgPObeZb2trFuPcBhqprw51B8E0icwXk";
    const RSA_3072: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDBvsT74MrSk7D321Ze61ue2FXHKVN7ca65j9KBJxS2vsFhcXmG46z/smNC8ojy5BIPmC5OC3TYIgJNjc5EP6PdzoBflkdEDMhiu19zwRmasW2q8tOCJyhcmmogDLdkAjT6oMMJ3Q7sJmGY4m9fDnGf2nXz8p/JMOHlrn0MosawxxoEJGR0ca4w8yXyCz6x+YDMamCI+dINg43DFAEpQj3a6ijp0jcz1kdidm3sQX8+XpyYL+CvR2P3XMiSDpOoQwUrUaf5hiepe802naeelhnTCbNta2OUGMcWIO4lkjmI18QMoZfcN9cV3hpB2FPkmJqyNFCBNdKzbtemsVAWY4Fh83utKe1F8nqflep3Tt9gIE1XzL02gWBiQvD/wKpOaKNJdONxRzh6D3C/BKyfAmunr/LgH0XjXhXniAR9/AHP/5P/Sb7ox1Y1ZRTsGFB5tbfst9XGIs1KNqAIisxEWhy2UHGsNI3qEJqQgbrnXLSTGg7eooDeVnV5k3TS9aoEXYc= rsa@cloudnode.test\n";
    const RSA_3072_FINGERPRINT: &str = "SHA256:9SAi2OxqR9wlm+AKPAzVFtWb6p4hKW2HmQ5QRAdU2RI";
    const ECDSA_384: &str = "ecdsa-sha2-nistp384 AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzODQAAABhBGk+LweZQTrAzwimPZaqU/8sPRJHT5UFk6D6gLHMtazkssvz/7erc4Epe60nkhV+UDU2F6xEDSMGCQX6NZXLxAGizIn0Dw+GkFW0dsa3DMZOh8CqEuUB4X+RVWVsickLWA== \n";
    const ECDSA_384_FINGERPRINT: &str = "SHA256:0RS2kOMZ4x+X04X3rRj3Yyf2tNWKZDuhZpXvdXYuovA";
    /// Not a real key: only its first line matters to the scan.
    const PRIVATE: &str = "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n";

    /// A fresh folder for one test, removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("cloudnode-keys-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn write(&self, name: &str, text: &str) {
            fs::write(self.0.join(name), text).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn parses_what_ssh_keygen_writes() {
        let ed = parse_public_key(ED25519).unwrap();
        assert_eq!(
            ed,
            PublicDetails { algorithm: KeyAlgorithm::Ed25519, fingerprint: ED25519_FINGERPRINT.into(), comment: "ada@cloudnode.test".into() }
        );

        let rsa = parse_public_key(RSA_3072).unwrap();
        assert_eq!(rsa.algorithm, KeyAlgorithm::Rsa { bits: 3072 });
        assert_eq!(rsa.fingerprint, RSA_3072_FINGERPRINT);

        let ecdsa = parse_public_key(ECDSA_384).unwrap();
        assert_eq!(ecdsa.algorithm, KeyAlgorithm::EcdsaP384);
        assert_eq!(ecdsa.fingerprint, ECDSA_384_FINGERPRINT);
        assert_eq!(ecdsa.comment, "");
    }

    #[test]
    fn skips_blank_and_comment_lines() {
        let text = format!("\n# my laptop\n{ED25519}");
        assert_eq!(parse_public_key(&text).unwrap().fingerprint, ED25519_FINGERPRINT);
    }

    #[test]
    fn junk_is_an_error_not_a_panic() {
        for text in ["", "\n\n", "# only a comment", "ssh-ed25519", "ssh-ed25519 !!!notbase64", "ssh-ed25519 AAAA", PRIVATE] {
            assert!(parse_public_key(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn every_algorithm_round_trips_by_its_openssh_name() {
        let all = [
            KeyAlgorithm::Ed25519,
            KeyAlgorithm::Rsa { bits: 2048 },
            KeyAlgorithm::EcdsaP256,
            KeyAlgorithm::EcdsaP384,
            KeyAlgorithm::EcdsaP521,
            KeyAlgorithm::SkEd25519,
            KeyAlgorithm::SkEcdsaP256,
            KeyAlgorithm::Dsa,
        ];
        for expected in all {
            assert_eq!(algorithm_named(expected.ssh_name(), || 2048), Some(expected));
        }
        assert_eq!(algorithm_named("ssh-made-up", || 0), None);
    }

    #[test]
    fn pairs_files_by_name_and_skips_what_is_not_a_key() {
        let dir = TempDir::new("pairs");
        dir.write("id_ed25519", PRIVATE);
        dir.write("id_ed25519.pub", ED25519);
        dir.write("id_rsa", PRIVATE);
        dir.write("work.pub", RSA_3072);
        dir.write("broken.pub", "this is not a key\n");
        dir.write("id_ed25519-cert.pub", "ssh-ed25519-cert-v01@openssh.com AAAA cert\n");
        dir.write("known_hosts", "example.com ssh-ed25519 AAAA\n");
        dir.write("config", "Host *\n");
        dir.write("notes.txt", "hello\n");
        dir.write(".DS_Store", "");
        fs::create_dir(dir.0.join("sockets")).unwrap();

        let scan = scan_dir(&dir.0).unwrap();
        let names: Vec<&str> = scan.keys.iter().map(|k| k.name.as_str()).collect();
        assert_eq!(names, ["broken", "id_ed25519", "id_rsa", "work"]);

        let [broken, ed, rsa, work] = scan.keys.as_slice() else { panic!("four keys") };
        assert!(matches!(broken.details, Details::Unreadable(_)));
        assert!(ed.public_file.is_some() && ed.private_file.is_some());
        assert_eq!(ed.details, Details::Public(parse_public_key(ED25519).unwrap()));
        assert!(ed.created.is_some(), "every file system keeps at least a modification time");
        assert_eq!(rsa.details, Details::PrivateOnly);
        assert!(rsa.public_file.is_none());
        assert!(work.private_file.is_none());
        let Details::Public(work_details) = &work.details else { panic!("work.pub is a readable key") };
        assert_eq!(work_details.algorithm, KeyAlgorithm::Rsa { bits: 3072 });
    }

    #[test]
    fn a_missing_folder_has_no_keys() {
        let dir = TempDir::new("missing");
        let scan = scan_dir(&dir.0.join("nope")).unwrap();
        assert!(scan.keys.is_empty());
    }

    #[test]
    fn oversized_public_keys_are_reported() {
        let dir = TempDir::new("oversized");
        dir.write("huge.pub", &"A".repeat(usize::try_from(MAX_PUBLIC_KEY_BYTES).unwrap() + 10));
        let scan = scan_dir(&dir.0).unwrap();
        assert!(matches!(&scan.keys[0].details, Details::Unreadable(reason) if reason.contains("too large")));
    }
}
