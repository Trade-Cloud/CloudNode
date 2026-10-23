//! Saved hosts, the groups they sit in, and how they sign in.
//!
//! Everything here comes from the user or from a vault file on disk, so [`Host::validate`] checks
//! it before the SSH layer ever sees it.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::key::KeyId;

/// The standard SSH port.
pub const DEFAULT_PORT: u16 = 22;
/// Longest host label, in characters.
pub const MAX_LABEL_LEN: usize = 128;
/// Longest host name, in bytes (the DNS limit).
pub const MAX_ADDRESS_LEN: usize = 253;
/// Longest DNS label (the part between two dots), in bytes.
const MAX_DNS_LABEL_LEN: usize = 63;
/// Longest user name, in characters.
pub const MAX_USER_LEN: usize = 64;

/// A host's id, unique within one vault. Ids are never reused, so a stale id can't point at
/// another host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HostId(pub u64);

/// A group's id, unique within one vault.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GroupId(pub u64);

/// A saved server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    pub id: HostId,
    /// The name shown in the sidebar and on the session tab.
    pub label: String,
    /// Host name or IP address. An IPv6 address may come in brackets (`[::1]`).
    pub address: String,
    pub port: u16,
    pub user: String,
    pub auth: Auth,
    /// The sidebar group, or none for the top level.
    pub group: Option<GroupId>,
    /// Listed under Favorites.
    pub favorite: bool,
}

/// How CloudNode signs in to a host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    /// The password saved in the vault, or asked for when connecting.
    Password,
    /// A key from Key Management.
    Key(KeyId),
    /// The keys the system's SSH agent offers.
    Agent,
}

/// A sidebar group of hosts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
}

/// Why a host can't be saved or connected to.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HostError {
    #[error("the host needs a name")]
    EmptyLabel,
    #[error("the host name is too long")]
    LabelTooLong,
    #[error("the host needs an address")]
    EmptyAddress,
    #[error("the address is not a valid host name or IP address")]
    InvalidAddress,
    #[error("port 0 is not a valid SSH port")]
    InvalidPort,
    #[error("the user name is empty, too long or contains spaces")]
    InvalidUser,
}

impl Host {
    /// Checks every field, so a hand-edited or damaged vault can't hand the SSH layer an address
    /// or user name it would misread.
    pub fn validate(&self) -> Result<(), HostError> {
        let label = self.label.trim();
        if label.is_empty() {
            return Err(HostError::EmptyLabel);
        }
        if label.chars().count() > MAX_LABEL_LEN {
            return Err(HostError::LabelTooLong);
        }
        validate_address(&self.address)?;
        if self.port == 0 {
            return Err(HostError::InvalidPort);
        }
        validate_user(&self.user)
    }
}

/// An IP address (IPv6 with or without brackets) or an ASCII host name: dot-separated labels of
/// letters, digits, `-` and `_`, none starting or ending with `-`. A trailing dot is allowed.
/// International names are entered in their `xn--` form.
pub fn validate_address(address: &str) -> Result<(), HostError> {
    if address.is_empty() {
        return Err(HostError::EmptyAddress);
    }
    let bare = address.strip_prefix('[').and_then(|a| a.strip_suffix(']')).unwrap_or(address);
    if bare.parse::<IpAddr>().is_ok() {
        return Ok(());
    }
    if address.len() > MAX_ADDRESS_LEN {
        return Err(HostError::InvalidAddress);
    }
    let name = address.strip_suffix('.').unwrap_or(address);
    let valid = name.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= MAX_DNS_LABEL_LEN
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    });
    if valid { Ok(()) } else { Err(HostError::InvalidAddress) }
}

/// A user name: not empty, at most [`MAX_USER_LEN`] characters, no spaces or control characters.
/// `@` is allowed (`user@corp.example` on some directory servers).
pub fn validate_user(user: &str) -> Result<(), HostError> {
    if user.is_empty() || user.chars().count() > MAX_USER_LEN || user.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(HostError::InvalidUser);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host() -> Host {
        Host {
            id: HostId(1),
            label: "Build server".into(),
            address: "build.example.com".into(),
            port: DEFAULT_PORT,
            user: "deploy".into(),
            auth: Auth::Agent,
            group: None,
            favorite: false,
        }
    }

    #[test]
    fn a_complete_host_is_valid() {
        assert_eq!(host().validate(), Ok(()));
    }

    #[test]
    fn blank_label_is_rejected() {
        let h = Host { label: "   ".into(), ..host() };
        assert_eq!(h.validate(), Err(HostError::EmptyLabel));
    }

    #[test]
    fn port_zero_is_rejected() {
        let h = Host { port: 0, ..host() };
        assert_eq!(h.validate(), Err(HostError::InvalidPort));
    }

    #[test]
    fn ip_addresses_are_accepted() {
        for address in ["10.0.0.1", "::1", "[::1]", "[fe80::1]", "2001:db8::42"] {
            assert_eq!(validate_address(address), Ok(()), "{address}");
        }
    }

    #[test]
    fn host_names_are_checked_label_by_label() {
        for address in ["example.com", "example.com.", "my_box.lan", "xn--80ak6aa92e.com", "a-b.c"] {
            assert_eq!(validate_address(address), Ok(()), "{address}");
        }
        for address in ["", "exa mple.com", "-bad.com", "bad-.com", "two..dots", "ünïcode.com", "a;b", "[not-ip]"] {
            assert!(validate_address(address).is_err(), "{address}");
        }
    }

    #[test]
    fn overlong_names_are_rejected() {
        let long = format!("{}.com", "a".repeat(MAX_DNS_LABEL_LEN + 1));
        assert_eq!(validate_address(&long), Err(HostError::InvalidAddress));
        let longer = ["abc"; 80].join(".");
        assert_eq!(validate_address(&longer), Err(HostError::InvalidAddress));
    }

    #[test]
    fn user_names_without_spaces_are_accepted() {
        assert_eq!(validate_user("root"), Ok(()));
        assert_eq!(validate_user("admin@corp.example"), Ok(()));
        for user in ["", "two words", "tab\there", "new\nline"] {
            assert_eq!(validate_user(user), Err(HostError::InvalidUser), "{user:?}");
        }
        assert_eq!(validate_user(&"u".repeat(MAX_USER_LEN + 1)), Err(HostError::InvalidUser));
    }
}
