//! Home's server list: the hosts `~/.ssh/config` names, read off the UI thread so the window opens
//! at once and shown through the `HostList` global (`ui/hosts.slint`), and the New Server form,
//! which adds a server to that config. `found` holds every server, `hosts` the ones matching the
//! search box.

use std::thread;
use std::time::SystemTime;

use cloudnode_core::time::iso_date;
use cloudnode_fs::tilde;
use cloudnode_ssh::config::{self, ConfigError, ConfigHost, HostsFile, NewHost};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{HostEntry, HostList, MainWindow};

/// Wires up the server list's menu, search box and form, then starts reading the config.
pub fn load(window: &MainWindow) {
    let list = window.global::<HostList>();

    let weak = window.as_weak();
    list.on_refresh(move || {
        if let Some(window) = weak.upgrade() {
            read(&window);
        }
    });

    let weak = window.as_weak();
    list.on_search(move |query| {
        if let Some(window) = weak.upgrade() {
            let list = window.global::<HostList>();
            list.set_query(query);
            filter(&list);
        }
    });

    let weak = window.as_weak();
    list.on_add(move |name, address, user, port| {
        if let Some(window) = weak.upgrade() {
            add(&window, &name, &address, &user, &port);
        }
    });

    read(window);
}

/// Reads `~/.ssh/config` in the background; the list fills in when that is done.
fn read(window: &MainWindow) {
    let list = window.global::<HostList>();
    if let Some(path) = config::home_config_path() {
        list.set_source(tilde(&path).into());
    }
    list.set_loading(true);

    let weak = window.as_weak();
    let job = move || {
        let result = config::read_home();
        if let Err(e) = weak.upgrade_in_event_loop(move |window| show(&window, result)) {
            log::warn!("could not show the servers: {e}");
        }
    };
    if let Err(e) = thread::Builder::new().name("cloudnode-hosts".into()).spawn(job) {
        log::warn!("could not read the servers in the background ({e}); reading them now");
        show(window, config::read_home());
    }
}

/// Puts what the config names in the list, or why it couldn't be read in place of the list.
fn show(window: &MainWindow, result: Result<HostsFile, ConfigError>) {
    let list = window.global::<HostList>();
    match result {
        Ok(file) => {
            let source = tilde(&file.path);
            log::info!("{} servers in {source}", file.hosts.len());
            list.set_source(source.into());
            list.set_error(SharedString::new());
            let entries: Vec<HostEntry> = file.hosts.iter().map(entry).collect();
            list.set_found(ModelRc::new(VecModel::from(entries)));
        }
        Err(e) => {
            log::warn!("could not read the servers: {e}");
            list.set_error(sentence(&e.to_string()).into());
            list.set_found(ModelRc::default());
        }
    }
    filter(&list);
    list.set_loading(false);
}

/// The New Server form's "Add Server": checks what was typed, adds the server to the config,
/// closes the form and selects the new server; or leaves the form open saying what to fix.
fn add(window: &MainWindow, name: &str, address: &str, user: &str, port: &str) {
    let list = window.global::<HostList>();
    let host = match NewHost::from_form(name, address, user, port) {
        Ok(host) => host,
        Err(problem) => {
            list.set_form_error(sentence(&problem).into());
            return;
        }
    };
    let Some(path) = config::home_config_path() else {
        list.set_form_error(sentence(&ConfigError::NoHome.to_string()).into());
        return;
    };
    let today = iso_date(SystemTime::now());
    match config::append(&path, &host, today.as_deref()) {
        Ok(()) => {
            log::info!("added {} to {}", host.alias, tilde(&path));
            list.set_form_open(false);
            list.set_form_error(SharedString::new());
            list.set_selected(host.alias.as_str().into());
            read(window);
        }
        Err(e) => {
            log::warn!("could not add {}: {e}", host.alias);
            list.set_form_error(sentence(&e.to_string()).into());
        }
    }
}

/// Shows the servers that match the search box.
fn filter(list: &HostList<'_>) {
    let query = list.get_query().trim().to_lowercase();
    let shown: Vec<HostEntry> = list.get_found().iter().filter(|entry| matches(entry, &query)).collect();
    list.set_hosts(ModelRc::new(VecModel::from(shown)));
}

/// Whether `query` (lower case) is in the server's name or `user@host:port`. An empty query
/// matches every server.
fn matches(entry: &HostEntry, query: &str) -> bool {
    query.is_empty() || entry.alias.to_lowercase().contains(query) || entry.detail.to_lowercase().contains(query)
}

/// One server as the list shows it.
fn entry(host: &ConfigHost) -> HostEntry {
    HostEntry { alias: host.alias.as_str().into(), detail: host.destination().into() }
}

/// `text` with a capital first letter, as a message in the window reads.
fn sentence(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_show_the_name_and_destination() {
        let host = ConfigHost { alias: "web".into(), hostname: "203.0.113.10".into(), user: Some("deploy".into()), port: 22 };
        let row = entry(&host);
        assert_eq!((row.alias.as_str(), row.detail.as_str()), ("web", "deploy@203.0.113.10"));
    }

    #[test]
    fn search_looks_at_name_and_address() {
        let host = ConfigHost { alias: "web".into(), hostname: "203.0.113.10".into(), user: Some("deploy".into()), port: 22 };
        let row = entry(&host);
        for query in ["", "we", "deploy@", "113.10"] {
            assert!(matches(&row, query), "{query}");
        }
        assert!(!matches(&row, "staging"));
    }

    #[test]
    fn messages_start_with_a_capital() {
        assert_eq!(sentence("the server needs a name"), "The server needs a name");
        assert_eq!(sentence(""), "");
    }
}
