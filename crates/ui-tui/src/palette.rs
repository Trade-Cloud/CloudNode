//! The colour tokens as console colours.

use cloudnode_core::{Rgba, Tokens};
use ratatui::style::Color;

/// What the TUI draws with. A console has no alpha, so every translucent token is blended over
/// the surface it sits on.
pub struct Palette {
    pub window_bg: Color,
    pub sidebar_bg: Color,
    pub terminal_bg: Color,
    pub ink: Color,
    pub ink_strong: Color,
    pub muted: Color,
    pub faint: Color,
    /// Box-drawing borders. The window's hairline is too faint for a one-cell line, so the TUI
    /// uses the stronger rule.
    pub line: Color,
    pub accent: Color,
    pub on_accent: Color,
}

impl Palette {
    pub fn new(t: &Tokens) -> Self {
        let on_sidebar = |c: Rgba| rgb(c.over(t.sidebar_bg));
        Self {
            window_bg: rgb(t.window_bg),
            sidebar_bg: rgb(t.sidebar_bg),
            terminal_bg: rgb(t.terminal.background),
            ink: on_sidebar(t.ink),
            ink_strong: on_sidebar(t.ink_strong),
            muted: on_sidebar(t.muted),
            faint: on_sidebar(t.faint),
            line: on_sidebar(t.line_strong),
            accent: rgb(t.accent),
            on_accent: rgb(t.on_accent),
        }
    }
}

/// An opaque token as a 24-bit console colour.
fn rgb(c: Rgba) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translucent_tokens_become_opaque_blends() {
        let p = Palette::new(&Tokens::DARK);
        let expected = Tokens::DARK.muted.over(Tokens::DARK.sidebar_bg);
        assert_eq!(p.muted, Color::Rgb(expected.r, expected.g, expected.b));
        assert_eq!(p.accent, rgb(Tokens::DARK.accent));
    }
}
