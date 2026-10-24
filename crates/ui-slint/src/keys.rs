//! Key Management's list: the keys in `~/.ssh`, read off the UI thread so the window opens at
//! once, then shown through the `KeyList` global (`ui/keys.slint`). `found` holds every key,
//! `keys` the ones matching the search box.

use std::thread;

use cloudnode_core::time::iso_date;
use cloudnode_keys::{Details, LocalKey, Scan, ScanError, tilde};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{KeyEntry, KeyList, KeyState, MainWindow};

/// Wires up Key Management's menu and the search box, then starts reading the keys.
pub fn load(window: &MainWindow) {
    let list = window.global::<KeyList>();

    let weak = window.as_weak();
    list.on_refresh(move || {
        if let Some(window) = weak.upgrade() {
            read(&window);
        }
    });

    let weak = window.as_weak();
    list.on_search(move |query| {
        if let Some(window) = weak.upgrade() {
            let list = window.global::<KeyList>();
            list.set_query(query);
            filter(&list);
        }
    });

    read(window);
}

/// Reads `~/.ssh` in the background; the list fills in when that is done.
fn read(window: &MainWindow) {
    let list = window.global::<KeyList>();
    if let Some(dir) = cloudnode_keys::home_ssh_dir() {
        list.set_folder(tilde(&dir).into());
    }
    list.set_loading(true);

    let weak = window.as_weak();
    let job = move || {
        let result = cloudnode_keys::scan_home();
        if let Err(e) = weak.upgrade_in_event_loop(move |window| show(&window, result)) {
            log::warn!("could not show the keys: {e}");
        }
    };
    if let Err(e) = thread::Builder::new().name("cloudnode-keys".into()).spawn(job) {
        log::warn!("could not read the keys in the background ({e}); reading them now");
        show(window, cloudnode_keys::scan_home());
    }
}

/// Puts a scan's result in the list, or its error in place of the list.
fn show(window: &MainWindow, result: Result<Scan, ScanError>) {
    let list = window.global::<KeyList>();
    match result {
        Ok(scan) => {
            let folder = tilde(&scan.dir);
            log::info!("{} keys in {folder}", scan.keys.len());
            list.set_folder(folder.into());
            list.set_error(SharedString::new());
            let entries: Vec<KeyEntry> = scan.keys.iter().map(entry).collect();
            list.set_found(ModelRc::new(VecModel::from(entries)));
        }
        Err(e) => {
            log::warn!("could not read the keys: {e}");
            list.set_error(e.to_string().into());
            list.set_found(ModelRc::default());
        }
    }
    filter(&list);
    list.set_loading(false);
}

/// Shows the keys that match the search box.
fn filter(list: &KeyList<'_>) {
    let query = list.get_query().trim().to_lowercase();
    let shown: Vec<KeyEntry> = list.get_found().iter().filter(|entry| matches(entry, &query)).collect();
    list.set_keys(ModelRc::new(VecModel::from(shown)));
}

/// Whether `query` (lower case) is in the key's name, type, comment or fingerprint. An empty
/// query matches every key.
fn matches(entry: &KeyEntry, query: &str) -> bool {
    query.is_empty() || [&entry.name, &entry.kind, &entry.comment, &entry.fingerprint].iter().any(|field| field.to_lowercase().contains(query))
}

/// One key as the list shows it.
fn entry(key: &LocalKey) -> KeyEntry {
    let name = SharedString::from(key.name.as_str());
    let created: SharedString = key.created.and_then(iso_date).unwrap_or_default().into();
    let path: SharedString = key.private_file.as_deref().or(key.public_file.as_deref()).map(tilde).unwrap_or_default().into();
    match &key.details {
        Details::Public(public) => KeyEntry {
            name,
            state: KeyState::Readable,
            kind: public.algorithm.to_string().into(),
            fingerprint: public.fingerprint.as_str().into(),
            comment: public.comment.as_str().into(),
            created,
            path,
            ..Default::default()
        },
        Details::PrivateOnly => KeyEntry { name, state: KeyState::PrivateOnly, created, path, ..Default::default() },
        Details::Unreadable(reason) => KeyEntry { name, state: KeyState::Unreadable, problem: reason.as_str().into(), created, path, ..Default::default() },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use cloudnode_core::KeyAlgorithm;
    use cloudnode_keys::PublicDetails;

    use super::*;

    fn key(details: Details) -> LocalKey {
        LocalKey { name: "id_ed25519".into(), public_file: None, private_file: None, details, created: None }
    }

    fn rsa() -> LocalKey {
        let details = PublicDetails { algorithm: KeyAlgorithm::Rsa { bits: 4096 }, fingerprint: "SHA256:AbC".into(), comment: "me@host".into() };
        LocalKey {
            name: "id_rsa_prod".into(),
            public_file: Some(PathBuf::from("/nowhere/id_rsa_prod.pub")),
            private_file: None,
            details: Details::Public(details),
            created: Some(UNIX_EPOCH + Duration::from_secs(19_742 * 86_400)),
        }
    }

    #[test]
    fn public_keys_show_their_type_date_and_fingerprint() {
        let row = entry(&rsa());
        assert_eq!(row.name, "id_rsa_prod");
        assert_eq!(row.state, KeyState::Readable);
        assert_eq!(row.kind, "RSA 4096");
        assert_eq!(row.fingerprint, "SHA256:AbC");
        assert_eq!(row.comment, "me@host");
        assert_eq!(row.created, "2024-01-20");
        assert_eq!(row.path, "/nowhere/id_rsa_prod.pub");
    }

    #[test]
    fn other_keys_carry_their_state() {
        assert_eq!(entry(&key(Details::PrivateOnly)).state, KeyState::PrivateOnly);
        let row = entry(&key(Details::Unreadable("the file is empty".into())));
        assert_eq!((row.state, row.problem.as_str()), (KeyState::Unreadable, "the file is empty"));
        assert_eq!(row.created, "");
    }

    #[test]
    fn search_looks_at_name_type_comment_and_fingerprint() {
        let row = entry(&rsa());
        for query in ["", "prod", "rsa 4096", "me@", "sha256:abc"] {
            assert!(matches(&row, query), "{query}");
        }
        assert!(!matches(&row, "ed25519"));
    }
}
