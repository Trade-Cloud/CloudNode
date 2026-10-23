//! Fills the Slint `Theme` global (`ui/theme.slint`) from [`Tokens`], so the window uses exactly the
//! colours the TUI and the terminal renderer use.

use cloudnode_core::{Rgba, Tokens};
use slint::{Color, ComponentHandle};

use crate::{MainWindow, Theme};

/// A token as a Slint colour.
pub fn color(c: Rgba) -> Color {
    Color::from_argb_u8(c.a, c.r, c.g, c.b)
}

/// Sets every property of the `Theme` global. Call it before the window is shown: until then the
/// global's colours are transparent.
pub fn apply(window: &MainWindow, tokens: &Tokens) {
    let theme = window.global::<Theme>();
    theme.set_window_bg(color(tokens.window_bg));
    theme.set_sidebar_bg(color(tokens.sidebar_bg));
    theme.set_surface(color(tokens.surface));
    theme.set_field_bg(color(tokens.field_bg));
    theme.set_hover(color(tokens.hover));
    theme.set_ink(color(tokens.ink));
    theme.set_ink_strong(color(tokens.ink_strong));
    theme.set_muted(color(tokens.muted));
    theme.set_faint(color(tokens.faint));
    theme.set_line(color(tokens.line));
    theme.set_line_strong(color(tokens.line_strong));
    theme.set_accent(color(tokens.accent));
    theme.set_accent_hover(color(tokens.accent_hover));
    theme.set_accent_ink(color(tokens.accent_ink));
    theme.set_on_accent(color(tokens.on_accent));
    theme.set_selection(color(tokens.selection));
    theme.set_good(color(tokens.good));
    theme.set_warn(color(tokens.warn));
    theme.set_bad(color(tokens.bad));
    theme.set_terminal_bg(color(tokens.terminal.background));
    theme.set_terminal_fg(color(tokens.terminal.foreground));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_keep_every_channel() {
        let c = color(Rgba::hex_alpha(0x123456, 0x78));
        assert_eq!((c.red(), c.green(), c.blue(), c.alpha()), (0x12, 0x34, 0x56, 0x78));
    }
}
