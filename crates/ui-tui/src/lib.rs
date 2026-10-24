//! CloudNode's terminal interface, in Ratatui on Crossterm.
//!
//! The same layout as the window (servers or keys on the left, sessions on the right) in the same
//! colours: [`cloudnode_core::Tokens`] blended to opaque RGB, since a console can't draw
//! translucency. Tab switches the sidebar between Servers and Keys, as the window's rail does.
//! The app shell (`apps/cloudnode-tui`) owns start-up and exit.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod palette;
mod view;

use std::io;

use cloudnode_core::Tokens;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::view::{Data, Pane};

/// Takes over the console, runs until the user quits, and gives the console back, also when
/// drawing or reading input failed, so the user's shell is never left in raw mode.
pub fn run() -> io::Result<()> {
    // Read before taking over the console: ~/.ssh holds a handful of small files, and a read
    // that fails is shown in the list rather than stopping the app.
    let data = Data { hosts: cloudnode_ssh::config::read_home(), keys: cloudnode_keys::scan_home() };
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &Tokens::DARK, &data);
    let restored = ratatui::try_restore();
    result.and(restored)
}

fn event_loop(terminal: &mut DefaultTerminal, tokens: &Tokens, data: &Data) -> io::Result<()> {
    let colors = palette::Palette::new(tokens);
    let mut pane = Pane::Servers;
    loop {
        terminal.draw(|frame| view::draw(frame, &colors, data, pane))?;
        // Anything but a key (a resize, a focus change) just redraws.
        let Event::Key(key) = event::read()? else { continue };
        match action(key) {
            Some(Action::Quit) => return Ok(()),
            Some(Action::SwitchPane) => pane = pane.other(),
            None => {}
        }
    }
}

/// What a key press asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Quit,
    SwitchPane,
}

/// `q`, Esc and Ctrl+C quit; Tab and Shift+Tab switch between Servers and Keys. Only presses
/// count: Windows also reports releases.
fn action(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        KeyCode::Tab | KeyCode::BackTab => Some(Action::SwitchPane),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        action(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn q_esc_and_ctrl_c_quit() {
        assert_eq!(press(KeyCode::Char('q'), KeyModifiers::NONE), Some(Action::Quit));
        assert_eq!(press(KeyCode::Esc, KeyModifiers::NONE), Some(Action::Quit));
        assert_eq!(press(KeyCode::Char('c'), KeyModifiers::CONTROL), Some(Action::Quit));
        assert_eq!(press(KeyCode::Char('c'), KeyModifiers::NONE), None);
        assert_eq!(press(KeyCode::Enter, KeyModifiers::NONE), None);
    }

    #[test]
    fn tab_switches_the_list() {
        assert_eq!(press(KeyCode::Tab, KeyModifiers::NONE), Some(Action::SwitchPane));
        assert_eq!(press(KeyCode::BackTab, KeyModifiers::SHIFT), Some(Action::SwitchPane));
    }

    #[test]
    fn releases_are_ignored() {
        let mut key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert_eq!(action(key), None);
    }
}
