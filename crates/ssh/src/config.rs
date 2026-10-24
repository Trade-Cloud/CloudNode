//! The user's SSH client config, `~/.ssh/config`: the servers it names, and adding one.
//!
//! Until CloudNode's own vault holds servers (M1 in `ROADMAP.md`), Home lists the hosts this file
//! names, and a server the user creates is added to it as a `Host` block at the end, so `ssh
//! <name>` works for it too. Reading is forgiving and bounded; adding appends one block through an
//! atomic write and keeps everything already in the file byte for byte.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use cloudnode_core::host::{DEFAULT_PORT, validate_address, validate_user};

/// Config files bigger than this are not read (real ones are a few kilobytes).
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
/// Hosts listed from one file.
const MAX_HOSTS: usize = 1000;
/// Longest name accepted for a new server, in characters.
const MAX_ALIAS_LEN: usize = 128;
/// Characters a new server's name can't contain: they mean something else in a `Host` line.
const NOT_IN_ALIAS: &[char] = &['*', '?', '!', '#', '"', '\'', ',', '='];

/// A server the config names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigHost {
    /// The name after `Host`, as typed in `ssh <alias>`.
    pub alias: String,
    /// `HostName`, or the alias when the block has none.
    pub hostname: String,
    pub user: Option<String>,
    pub port: u16,
}

impl ConfigHost {
    /// Where the host is, as `user@host:port`, leaving out what the config doesn't say (no user)
    /// and the default port. An IPv6 address followed by a port is bracketed so the port stays
    /// readable.
    pub fn destination(&self) -> String {
        let mut destination = self.user.as_ref().map(|user| format!("{user}@")).unwrap_or_default();
        if self.port == DEFAULT_PORT {
            destination.push_str(&self.hostname);
        } else if self.hostname.contains(':') {
            destination.push_str(&format!("[{}]:{}", self.hostname, self.port));
        } else {
            destination.push_str(&format!("{}:{}", self.hostname, self.port));
        }
        destination
    }
}

/// What one config file names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostsFile {
    pub path: PathBuf,
    /// In file order, each name once.
    pub hosts: Vec<ConfigHost>,
}

/// Why the config couldn't be read or added to.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("this system has no home folder")]
    NoHome,
    #[error("could not read {}: {source}", .path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("{} is too large to be an SSH config", .path.display())]
    TooLarge { path: PathBuf },
    #[error("could not write {}: {source}", .path.display())]
    Write { path: PathBuf, source: io::Error },
    #[error("a server named {alias} is already in the SSH config")]
    Exists { alias: String },
}

/// The user's `~/.ssh/config`.
pub fn home_config_path() -> Option<PathBuf> {
    cloudnode_fs::home_ssh_dir().map(|dir| dir.join("config"))
}

/// The hosts in the user's `~/.ssh/config`. A missing file names no hosts.
pub fn read_home() -> Result<HostsFile, ConfigError> {
    let path = home_config_path().ok_or(ConfigError::NoHome)?;
    read(&path)
}

/// The hosts in the config at `path`. A missing file names no hosts.
pub fn read(path: &Path) -> Result<HostsFile, ConfigError> {
    let text = read_bytes(path)?.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()).unwrap_or_default();
    Ok(HostsFile { path: path.to_path_buf(), hosts: parse(&text) })
}

/// The file's bytes, or `None` when there is no file.
fn read_bytes(path: &Path) -> Result<Option<Vec<u8>>, ConfigError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ConfigError::Read { path: path.to_path_buf(), source }),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes).map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(ConfigError::TooLarge { path: path.to_path_buf() });
    }
    Ok(Some(bytes))
}

/// One `Host` (or `Match`) block while parsing: its concrete names and the first value of each
/// setting CloudNode shows (OpenSSH, too, takes the first value it finds).
#[derive(Default)]
struct Block {
    aliases: Vec<String>,
    hostname: Option<String>,
    user: Option<String>,
    port: Option<u16>,
}

/// The concrete hosts named in config text, in file order, each name once. `Host` patterns with
/// wildcards (`*`, `?`) or negations (`!`) are rules rather than servers, and `Match` blocks are
/// skipped. Each host shows what its own block says: settings that would reach it from `Host *`
/// and the like are not merged in. `Include` is not followed yet.
pub fn parse(text: &str) -> Vec<ConfigHost> {
    let mut blocks: Vec<Block> = Vec::new();
    for line in text.lines() {
        let Some((keyword, args)) = split_line(line) else { continue };
        let keyword = keyword.to_ascii_lowercase();
        if keyword == "host" {
            let aliases = words(args).into_iter().filter(|word| !is_pattern(word)).collect();
            blocks.push(Block { aliases, ..Block::default() });
            continue;
        }
        if keyword == "match" {
            blocks.push(Block::default());
            continue;
        }
        // Settings before the first block apply to every host; CloudNode shows per-host ones.
        let Some(block) = blocks.last_mut() else { continue };
        let Some(value) = words(args).into_iter().next() else { continue };
        match keyword.as_str() {
            "hostname" if block.hostname.is_none() => block.hostname = Some(value),
            "user" if block.user.is_none() => block.user = Some(value),
            "port" if block.port.is_none() => block.port = value.parse::<u16>().ok().filter(|&port| port != 0),
            _ => {}
        }
    }

    let mut hosts: Vec<ConfigHost> = Vec::new();
    for block in blocks {
        for alias in block.aliases {
            if hosts.len() >= MAX_HOSTS {
                return hosts;
            }
            if hosts.iter().any(|host| host.alias == alias) {
                continue;
            }
            let hostname = block.hostname.clone().unwrap_or_else(|| alias.clone());
            hosts.push(ConfigHost { alias, hostname, user: block.user.clone(), port: block.port.unwrap_or(DEFAULT_PORT) });
        }
    }
    hosts
}

/// A config line's keyword and the rest, for `Keyword value`, `Keyword=value` and
/// `Keyword = value`; `None` for blank and comment lines.
fn split_line(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let end = line.find(|c: char| c.is_whitespace() || c == '=').unwrap_or(line.len());
    let (keyword, rest) = line.split_at(end);
    let rest = rest.trim_start();
    Some((keyword, rest.strip_prefix('=').unwrap_or(rest).trim()))
}

/// A line's arguments, split on spaces; double quotes keep spaces inside one argument.
fn words(args: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in args.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Whether a `Host` word is a pattern (`*`, `?`) or a negation (`!`) rather than one server.
fn is_pattern(word: &str) -> bool {
    word.contains(['*', '?']) || word.starts_with('!')
}

/// A server to add to the config, already checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: u16,
}

impl NewHost {
    /// Checks what the user typed in the New Server form. Blank user and port mean "OpenSSH's
    /// default" (the local user name, port 22). The error says what to fix.
    pub fn from_form(name: &str, address: &str, user: &str, port: &str) -> Result<Self, String> {
        let alias = name.trim();
        validate_alias(alias)?;

        let address = address.trim();
        validate_address(address).map_err(|e| e.to_string())?;
        // OpenSSH's HostName takes an IPv6 address without brackets.
        let hostname = address.strip_prefix('[').and_then(|a| a.strip_suffix(']')).unwrap_or(address).to_owned();

        let user = user.trim();
        let user = if user.is_empty() {
            None
        } else {
            validate_user(user).map_err(|e| e.to_string())?;
            Some(user.to_owned())
        };

        let port = port.trim();
        let port = if port.is_empty() {
            DEFAULT_PORT
        } else {
            port.parse::<u16>().ok().filter(|&p| p != 0).ok_or_else(|| format!("{port} is not a port number (1–65535)"))?
        };

        Ok(Self { alias: alias.to_owned(), hostname, user, port })
    }

    /// The `Host` block written for this server. `added` is the date it was added, for the
    /// comment above it.
    pub fn block(&self, added: Option<&str>) -> String {
        let mut block = match added {
            Some(date) => format!("# Added by CloudNode on {date}\n"),
            None => "# Added by CloudNode\n".to_owned(),
        };
        block.push_str(&format!("Host {}\n    HostName {}\n", self.alias, self.hostname));
        if let Some(user) = &self.user {
            block.push_str(&format!("    User {user}\n"));
        }
        if self.port != DEFAULT_PORT {
            block.push_str(&format!("    Port {}\n", self.port));
        }
        block
    }
}

/// A new server's name: not empty, at most [`MAX_ALIAS_LEN`] characters, no spaces and none of
/// the characters that make a `Host` word a pattern or a comment.
fn validate_alias(alias: &str) -> Result<(), String> {
    if alias.is_empty() {
        return Err("the server needs a name".into());
    }
    if alias.chars().count() > MAX_ALIAS_LEN {
        return Err("the name is too long".into());
    }
    if alias.chars().any(|c| c.is_whitespace() || c.is_control() || NOT_IN_ALIAS.contains(&c)) {
        return Err("the name can't contain spaces or any of * ? ! # \" ' , =".into());
    }
    Ok(())
}

/// Adds `host` as a new `Host` block at the end of the config at `path`, after a blank line,
/// making the file (and its folder, owner-only) when there is none. Everything already in the file
/// is kept byte for byte; a name it already has is refused.
pub fn append(path: &Path, host: &NewHost, added: Option<&str>) -> Result<(), ConfigError> {
    let mut bytes = read_bytes(path)?.unwrap_or_default();
    if parse(&String::from_utf8_lossy(&bytes)).iter().any(|existing| existing.alias == host.alias) {
        return Err(ConfigError::Exists { alias: host.alias.clone() });
    }
    if !bytes.is_empty() {
        if !bytes.ends_with(b"\n") {
            bytes.push(b'\n');
        }
        bytes.push(b'\n');
    }
    bytes.extend_from_slice(host.block(added).as_bytes());

    let write_error = |source| ConfigError::Write { path: path.to_path_buf(), source };
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        cloudnode_fs::ensure_private_dir(dir).map_err(write_error)?;
    }
    cloudnode_fs::atomic::write(path, &bytes).map_err(write_error)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const CONFIG: &str = "\
# My servers
Include ~/.ssh/work.conf
User everyone

Host web-prod-01 web1
    HostName 203.0.113.10
    User deploy
    Port 2222

Host db-replica-2
  hostname=db.example.com
  USER = postgres

Host *.internal !bastion jump?
    User nobody

Match host example.org
    User matched

Host api-staging
    HostName \"api staging.example.com\"
    HostName second-ignored.example.com
    Port not-a-port

Host web1
    HostName duplicate.example.com
";

    /// A fresh folder for one test, removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("cloudnode-ssh-{}-{name}", std::process::id()));
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

    fn host(alias: &str, hostname: &str, user: Option<&str>, port: u16) -> ConfigHost {
        ConfigHost { alias: alias.into(), hostname: hostname.into(), user: user.map(Into::into), port }
    }

    #[test]
    fn lists_the_concrete_hosts_in_order() {
        assert_eq!(
            parse(CONFIG),
            [
                host("web-prod-01", "203.0.113.10", Some("deploy"), 2222),
                host("web1", "203.0.113.10", Some("deploy"), 2222),
                host("db-replica-2", "db.example.com", Some("postgres"), 22),
                host("api-staging", "api staging.example.com", None, 22),
            ]
        );
    }

    #[test]
    fn destinations_leave_out_what_is_default() {
        assert_eq!(host("a", "203.0.113.10", Some("deploy"), 22).destination(), "deploy@203.0.113.10");
        assert_eq!(host("a", "db.example.com", None, 2222).destination(), "db.example.com:2222");
        assert_eq!(host("a", "2001:db8::1", Some("root"), 2222).destination(), "root@[2001:db8::1]:2222");
        assert_eq!(host("a", "2001:db8::1", None, 22).destination(), "2001:db8::1");
    }

    #[test]
    fn junk_is_skipped_not_fatal() {
        for text in ["", "\n\n# only comments\n", "Host", "Port 22", "HostName x\nUser y", "Host \"unterminated", "\u{0}\u{1}Host a"] {
            let _ = parse(text);
        }
        assert_eq!(parse("Host \"unterminated"), [host("unterminated", "unterminated", None, 22)]);
    }

    #[test]
    fn a_missing_file_names_no_hosts() {
        let dir = TempDir::new("missing");
        assert!(read(&dir.0.join("config")).unwrap().hosts.is_empty());
    }

    #[test]
    fn the_form_is_checked() {
        let ok = NewHost::from_form(" web-prod-01 ", "[2001:db8::1]", "", "").unwrap();
        assert_eq!(ok, NewHost { alias: "web-prod-01".into(), hostname: "2001:db8::1".into(), user: None, port: 22 });
        assert!(NewHost::from_form("", "example.com", "", "").is_err());
        assert!(NewHost::from_form("two words", "example.com", "", "").is_err());
        assert!(NewHost::from_form("web*", "example.com", "", "").is_err());
        assert!(NewHost::from_form("web", "not an address", "", "").is_err());
        assert!(NewHost::from_form("web", "example.com", "a b", "").is_err());
        assert!(NewHost::from_form("web", "example.com", "", "0").is_err());
        assert!(NewHost::from_form("web", "example.com", "", "70000").is_err());
        assert_eq!(NewHost::from_form("web", "example.com", "root", "2222").unwrap().port, 2222);
    }

    #[test]
    fn writes_only_what_differs_from_the_defaults() {
        let plain = NewHost { alias: "a".into(), hostname: "a.example.com".into(), user: None, port: 22 };
        assert_eq!(plain.block(None), "# Added by CloudNode\nHost a\n    HostName a.example.com\n");
        let full = NewHost { user: Some("root".into()), port: 2222, ..plain };
        assert_eq!(
            full.block(Some("2026-10-24")),
            "# Added by CloudNode on 2026-10-24\nHost a\n    HostName a.example.com\n    User root\n    Port 2222\n"
        );
    }

    #[test]
    fn appends_after_what_is_there_and_keeps_it() {
        let dir = TempDir::new("append");
        let path = dir.0.join("config");
        let before: &[u8] = b"Host old\n    HostName old.example.com\n    # \xff not UTF-8, kept as is";
        fs::write(&path, before).unwrap();

        let new = NewHost::from_form("new", "new.example.com", "me", "").unwrap();
        append(&path, &new, None).unwrap();

        let after = fs::read(&path).unwrap();
        assert!(after.starts_with(before));
        assert!(after.ends_with(b"\n\n# Added by CloudNode\nHost new\n    HostName new.example.com\n    User me\n"));
        let names: Vec<String> = read(&path).unwrap().hosts.into_iter().map(|h| h.alias).collect();
        assert_eq!(names, ["old", "new"]);
    }

    #[test]
    fn makes_the_file_and_folder_when_missing() {
        let dir = TempDir::new("fresh");
        let path = dir.0.join(".ssh").join("config");
        append(&path, &NewHost::from_form("box", "10.0.0.5", "", "").unwrap(), None).unwrap();
        assert_eq!(read(&path).unwrap().hosts, [host("box", "10.0.0.5", None, 22)]);
    }

    #[test]
    fn refuses_a_name_that_is_already_there() {
        let dir = TempDir::new("exists");
        let path = dir.0.join("config");
        fs::write(&path, "Host web\n").unwrap();
        let result = append(&path, &NewHost::from_form("web", "example.com", "", "").unwrap(), None);
        assert!(matches!(result, Err(ConfigError::Exists { .. })));
        assert_eq!(fs::read_to_string(&path).unwrap(), "Host web\n");
    }
}
