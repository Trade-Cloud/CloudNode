//! CloudNode's terminal interface, in Ratatui on Crossterm.
//!
//! The same layout as the window (hosts and keys on the left, sessions on the right) in the same
//! colours: [`cloudnode_core::Tokens`] blended to opaque RGB, since a console can't draw
//! translucency. The app shell (`apps/cloudnode-tui`) owns start-up and exit.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod palette;
mod view;

use std::io;

use cloudnode_core::Tokens;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Takes over the console, runs until the user quits, and gives the console back, also when
/// drawing or reading input failed, so the user's shell is never left in raw mode.
pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &Tokens::DARK);
    let restored = ratatui::try_restore();
    result.and(restored)
}

fn event_loop(terminal: &mut DefaultTerminal, tokens: &Tokens) -> io::Result<()> {
    let colors = palette::Palette::new(tokens);
    loop {
        terminal.draw(|frame| view::draw(frame, &colors))?;
        // Anything but a key (a resize, a focus change) just redraws.
        let Event::Key(key) = event::read()? else { continue };
        if quits(key) {
            return Ok(());
        }
    }
}

/// `q`, Esc and Ctrl+C quit. Only presses count: Windows also reports releases.
fn quits(key: KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_esc_and_ctrl_c_quit() {
        assert!(quits(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)));
        assert!(quits(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(quits(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!quits(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)));
        assert!(!quits(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
    }

    #[test]
    fn releases_are_ignored() {
        let mut key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert!(!quits(key));
    }
}
