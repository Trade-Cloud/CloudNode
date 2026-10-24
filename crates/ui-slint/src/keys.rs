//! Key Management's list: the keys in `~/.ssh`, read off the UI thread so the window opens at
//! once, then shown through the `KeyList` global (`ui/keys.slint`).

use std::thread;

use cloudnode_keys::{Details, LocalKey, Scan, ScanError};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{KeyEntry, KeyList, KeyState, MainWindow};

/// Starts reading the keys in the background; the list fills in when that is done.
pub fn load(window: &MainWindow) {
    let list = window.global::<KeyList>();
    if let Some(dir) = cloudnode_keys::home_ssh_dir() {
        list.set_folder(cloudnode_keys::tilde(&dir).into());
    }
    list.set_loading(true);

    let weak = window.as_weak();
    let read = move || {
        let result = cloudnode_keys::scan_home();
        if let Err(e) = weak.upgrade_in_event_loop(move |window| show(&window, result)) {
            log::warn!("could not show the keys: {e}");
        }
    };
    if let Err(e) = thread::Builder::new().name("cloudnode-keys".into()).spawn(read) {
        log::warn!("could not read the keys in the background ({e}); reading them now");
        show(window, cloudnode_keys::scan_home());
    }
}

/// Puts a scan's result in the list, or its error in place of the list.
fn show(window: &MainWindow, result: Result<Scan, ScanError>) {
    let list = window.global::<KeyList>();
    match result {
        Ok(scan) => {
            let folder = cloudnode_keys::tilde(&scan.dir);
            log::info!("{} keys in {folder}", scan.keys.len());
            list.set_folder(folder.into());
            list.set_error(SharedString::new());
            let entries: Vec<KeyEntry> = scan.keys.iter().map(entry).collect();
            list.set_keys(ModelRc::new(VecModel::from(entries)));
        }
        Err(e) => {
            log::warn!("could not read the keys: {e}");
            list.set_error(e.to_string().into());
        }
    }
    list.set_loading(false);
}

/// One key as the list shows it.
fn entry(key: &LocalKey) -> KeyEntry {
    let name = SharedString::from(key.name.as_str());
    match &key.details {
        Details::Public(public) => KeyEntry {
            name,
            state: KeyState::Readable,
            kind: public.algorithm.to_string().into(),
            fingerprint: public.fingerprint.as_str().into(),
            ..Default::default()
        },
        Details::PrivateOnly => KeyEntry { name, state: KeyState::PrivateOnly, ..Default::default() },
        Details::Unreadable(reason) => KeyEntry { name, state: KeyState::Unreadable, problem: reason.as_str().into(), ..Default::default() },
    }
}

#[cfg(test)]
mod tests {
    use cloudnode_core::KeyAlgorithm;
    use cloudnode_keys::PublicDetails;

    use super::*;

    fn key(details: Details) -> LocalKey {
        LocalKey { name: "id_ed25519".into(), public_file: None, private_file: None, details }
    }

    #[test]
    fn public_keys_show_their_type_and_fingerprint() {
        let details = PublicDetails { algorithm: KeyAlgorithm::Rsa { bits: 4096 }, fingerprint: "SHA256:abc".into(), comment: "me@host".into() };
        let row = entry(&key(Details::Public(details)));
        assert_eq!(row.name, "id_ed25519");
        assert_eq!(row.state, KeyState::Readable);
        assert_eq!(row.kind, "RSA 4096");
        assert_eq!(row.fingerprint, "SHA256:abc");
    }

    #[test]
    fn other_keys_carry_their_state() {
        assert_eq!(entry(&key(Details::PrivateOnly)).state, KeyState::PrivateOnly);
        let row = entry(&key(Details::Unreadable("the file is empty".into())));
        assert_eq!((row.state, row.problem.as_str()), (KeyState::Unreadable, "the file is empty"));
    }
}
