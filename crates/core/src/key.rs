//! What Key Management lists about each key. The private key itself never leaves the vault crate;
//! these types carry only what is safe to show.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A key's id, unique within one vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyId(pub u64);

/// A key's algorithm: every kind OpenSSH writes, so any key in `~/.ssh` can be shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Ed25519,
    /// RSA with this modulus size in bits.
    Rsa {
        bits: u16,
    },
    EcdsaP256,
    EcdsaP384,
    EcdsaP521,
    /// Ed25519 on a FIDO security key.
    SkEd25519,
    /// ECDSA P-256 on a FIDO security key.
    SkEcdsaP256,
    /// DSA: obsolete and refused by current OpenSSH. Listed, never generated.
    Dsa,
}

impl KeyAlgorithm {
    /// The algorithm name OpenSSH writes in `authorized_keys` and public key files.
    pub fn ssh_name(self) -> &'static str {
        match self {
            Self::Ed25519 => "ssh-ed25519",
            Self::Rsa { .. } => "ssh-rsa",
            Self::EcdsaP256 => "ecdsa-sha2-nistp256",
            Self::EcdsaP384 => "ecdsa-sha2-nistp384",
            Self::EcdsaP521 => "ecdsa-sha2-nistp521",
            Self::SkEd25519 => "sk-ssh-ed25519@openssh.com",
            Self::SkEcdsaP256 => "sk-ecdsa-sha2-nistp256@openssh.com",
            Self::Dsa => "ssh-dss",
        }
    }
}

/// The short name Key Management shows: `Ed25519`, `RSA 4096`, `ECDSA P-256`, ….
impl fmt::Display for KeyAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 => f.write_str("Ed25519"),
            Self::Rsa { bits } => write!(f, "RSA {bits}"),
            Self::EcdsaP256 => f.write_str("ECDSA P-256"),
            Self::EcdsaP384 => f.write_str("ECDSA P-384"),
            Self::EcdsaP521 => f.write_str("ECDSA P-521"),
            Self::SkEd25519 => f.write_str("Ed25519-SK"),
            Self::SkEcdsaP256 => f.write_str("ECDSA-SK P-256"),
            Self::Dsa => f.write_str("DSA"),
        }
    }
}

/// A key as Key Management shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyInfo {
    pub id: KeyId,
    /// The name the user gave it.
    pub name: String,
    pub algorithm: KeyAlgorithm,
    /// The SHA-256 fingerprint as OpenSSH prints it (`SHA256:…`).
    pub fingerprint: String,
    /// The public key's comment (often `user@machine`).
    pub comment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_names_match_openssh() {
        assert_eq!(KeyAlgorithm::Ed25519.ssh_name(), "ssh-ed25519");
        assert_eq!(KeyAlgorithm::Rsa { bits: 4096 }.ssh_name(), "ssh-rsa");
        assert_eq!(KeyAlgorithm::EcdsaP256.ssh_name(), "ecdsa-sha2-nistp256");
        assert_eq!(KeyAlgorithm::SkEd25519.ssh_name(), "sk-ssh-ed25519@openssh.com");
        assert_eq!(KeyAlgorithm::Dsa.ssh_name(), "ssh-dss");
    }

    #[test]
    fn display_names_are_short() {
        assert_eq!(KeyAlgorithm::Ed25519.to_string(), "Ed25519");
        assert_eq!(KeyAlgorithm::Rsa { bits: 3072 }.to_string(), "RSA 3072");
        assert_eq!(KeyAlgorithm::EcdsaP384.to_string(), "ECDSA P-384");
        assert_eq!(KeyAlgorithm::SkEcdsaP256.to_string(), "ECDSA-SK P-256");
    }
}
