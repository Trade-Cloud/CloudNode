//! The colour tokens as console colours.

use cloudnode_core::{Rgba, Tokens};
use ratatui::style::Color;

/// What the TUI draws with. A console has no alpha, so every translucent token is blended over
/// the surface it sits on.
pub struct Palette {
    pub sidebar_bg: Color,
    /// The bottom row with the key hints.
    pub content_bg: Color,
    /// The session panel.
    pub surface: Color,
    pub ink: Color,
    pub ink_strong: Color,
    /// Hints in the sidebar, blended over it.
    pub faint: Color,
    /// Box-drawing borders. The window's hairline is too close to the backgrounds for a one-cell
    /// line, so the TUI uses the stronger rule.
    pub line: Color,
    pub accent: Color,
    pub on_accent: Color,
    pub bad: Color,
}

impl Palette {
    pub fn new(t: &Tokens) -> Self {
        Self {
            sidebar_bg: rgb(t.sidebar_bg),
            content_bg: rgb(t.content_bg),
            surface: rgb(t.surface),
            ink: rgb(t.ink),
            ink_strong: rgb(t.ink_strong),
            faint: rgb(t.faint.over(t.sidebar_bg)),
            line: rgb(t.line_strong),
            accent: rgb(t.accent),
            on_accent: rgb(t.on_accent),
            bad: rgb(t.bad),
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
        let expected = Tokens::DARK.faint.over(Tokens::DARK.sidebar_bg);
        assert_eq!(p.faint, Color::Rgb(expected.r, expected.g, expected.b));
        assert_eq!(p.accent, rgb(Tokens::DARK.accent));
    }
}
