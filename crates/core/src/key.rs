//! What Key Management lists about each key. The private key itself never leaves the vault crate;
//! these types carry only what is safe to show.

use serde::{Deserialize, Serialize};

/// A key's id, unique within one vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyId(pub u64);

/// A key's algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    Ed25519,
    /// RSA with this modulus size in bits.
    Rsa {
        bits: u16,
    },
    EcdsaP256,
}

impl KeyAlgorithm {
    /// The algorithm name OpenSSH writes in `authorized_keys` and public key files.
    pub fn ssh_name(self) -> &'static str {
        match self {
            Self::Ed25519 => "ssh-ed25519",
            Self::Rsa { .. } => "ssh-rsa",
            Self::EcdsaP256 => "ecdsa-sha2-nistp256",
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
    }
}
