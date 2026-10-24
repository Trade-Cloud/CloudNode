//! Colour tokens, shared by every CloudNode surface: the Slint window, the Ratatui interface and
//! the terminal renderer read the same values, so CloudNode looks the same in a window and in a
//! console. Nothing outside this module writes a colour literal.
//!
//! The look is layered dark blue-grey: the [`BACKGROUND`] behind everything (the left panel and
//! the middle of the window), the [`SURFACE`] in front of it, [`ELEMENT`]s on the surface, text
//! and icons in [`TEXT`], and one accent, [`ACCENT`], for the selected icon (the open panel),
//! focus and the active session.

/// The back of the window: the left panel and the middle, behind the session panel.
pub const BACKGROUND: Rgba = Rgba::hex(0x171e28);
/// The main colour, in front of the background: the session panel, menus, popovers.
pub const SURFACE: Rgba = Rgba::hex(0x232b36);
/// Elements in the foreground: text fields, buttons, the selected row, hairlines.
pub const ELEMENT: Rgba = Rgba::hex(0x2a3845);
/// Selected icons (the panel that is open), focus rings, the active session.
pub const ACCENT: Rgba = Rgba::hex(0x3bbae3);
/// Icons and text.
pub const TEXT: Rgba = Rgba::hex(0x818898);

/// A colour in sRGB with straight (not premultiplied) alpha.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    /// An opaque colour from `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self { r, g, b, a: 255 }
    }

    /// A colour from `0xRRGGBB` and an alpha.
    pub const fn hex_alpha(rgb: u32, a: u8) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self { r, g, b, a }
    }

    /// The same colour with another alpha.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// This colour drawn over an opaque `background`, for surfaces that can't blend (a console
    /// takes opaque RGB only). The result is opaque.
    pub const fn over(self, background: Self) -> Self {
        Self { r: mix(self.r, background.r, self.a), g: mix(self.g, background.g, self.a), b: mix(self.b, background.b, self.a), a: 255 }
    }
}

/// `fg` at `alpha` over `bg`, rounded to nearest. Never exceeds 255: the weights sum to 255.
const fn mix(fg: u8, bg: u8, alpha: u8) -> u8 {
    let a = alpha as u32;
    ((fg as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8
}

/// The terminal's own colours: what programs on the server ask for by number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalPalette {
    pub foreground: Rgba,
    pub background: Rgba,
    pub cursor: Rgba,
    pub selection: Rgba,
    /// ANSI colours 0–15: black, red, green, yellow, blue, magenta, cyan and white, then their
    /// bright forms in the same order.
    pub ansi: [Rgba; 16],
}

/// Colour tokens for one appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokens {
    /// The icon rail at the window's left edge.
    pub rail_bg: Rgba,
    /// The left panel (sidebar) the rail opens.
    pub sidebar_bg: Rgba,
    /// The middle of the window, behind the session panel.
    pub content_bg: Rgba,
    /// The main surface in front of the backgrounds: the session panel, menus, popovers, the
    /// active tab.
    pub surface: Rgba,
    /// Elements in the foreground: text fields, buttons, the selected row.
    pub element: Rgba,
    /// An element under the pointer.
    pub element_hover: Rgba,
    /// Text and icons.
    pub ink: Rgba,
    /// Headings, typed text and the active tab's title.
    pub ink_strong: Rgba,
    /// Placeholders, empty states and disabled text.
    pub faint: Rgba,
    /// Hairline separators.
    pub line: Rgba,
    /// Field borders and stronger separators (the console's box lines).
    pub line_strong: Rgba,
    /// Selected icons (the panel that is open), focus rings, primary buttons, the active session.
    pub accent: Rgba,
    /// The accent under the pointer.
    pub accent_hover: Rgba,
    /// Text on an accent surface.
    pub on_accent: Rgba,
    /// Selected text and rows.
    pub selection: Rgba,
    /// Dims the window behind a dialog.
    pub scrim: Rgba,
    pub good: Rgba,
    pub warn: Rgba,
    pub bad: Rgba,
    pub terminal: TerminalPalette,
}

impl Tokens {
    /// The dark appearance, CloudNode's default. The six base colours are TradeCloud's; the
    /// hover, strong-text and strong-line tints are derived from them.
    pub const DARK: Self = Self {
        rail_bg: BACKGROUND,
        sidebar_bg: BACKGROUND,
        content_bg: BACKGROUND,
        surface: SURFACE,
        element: ELEMENT,
        element_hover: Rgba::hex(0x334457),
        ink: TEXT,
        ink_strong: Rgba::hex(0xd0d5dd),
        faint: TEXT.with_alpha(153),
        line: ELEMENT,
        line_strong: Rgba::hex(0x3a4a5c),
        accent: ACCENT,
        accent_hover: Rgba::hex(0x62c9ea),
        on_accent: BACKGROUND,
        selection: ACCENT.with_alpha(56),
        scrim: BACKGROUND.with_alpha(184),
        good: Rgba::hex(0x32d74b),
        warn: Rgba::hex(0xffd60a),
        bad: Rgba::hex(0xff453a),
        terminal: TerminalPalette {
            // Sessions open on the surface, so the terminal shares its colour.
            background: SURFACE,
            foreground: Rgba::hex(0xd0d5dd),
            cursor: ACCENT,
            selection: ACCENT.with_alpha(77),
            ansi: [
                ELEMENT,
                Rgba::hex(0xff5f57),
                Rgba::hex(0x5fd38d),
                Rgba::hex(0xf5c451),
                Rgba::hex(0x5aa9ff),
                Rgba::hex(0xc792ea),
                ACCENT,
                Rgba::hex(0xc3c8d0),
                Rgba::hex(0x5b6577),
                Rgba::hex(0xff7b73),
                Rgba::hex(0x7ee2a6),
                Rgba::hex(0xffd479),
                Rgba::hex(0x82bfff),
                Rgba::hex(0xd9aef5),
                Rgba::hex(0x6fd0ee),
                Rgba::hex(0xffffff),
            ],
        },
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_unpacks_rgb() {
        assert_eq!(Rgba::hex(0x123456), Rgba { r: 0x12, g: 0x34, b: 0x56, a: 255 });
        assert_eq!(Rgba::hex_alpha(0xabcdef, 7), Rgba { r: 0xab, g: 0xcd, b: 0xef, a: 7 });
    }

    #[test]
    fn over_blends_by_alpha() {
        let bg = Rgba::hex(0x000000);
        assert_eq!(Rgba::hex(0xffffff).over(bg), Rgba::hex(0xffffff));
        assert_eq!(Rgba::hex_alpha(0xffffff, 0).over(bg), bg);
        assert_eq!(Rgba::hex_alpha(0xffffff, 128).over(bg), Rgba::hex(0x808080));
        assert_eq!(Rgba::hex_alpha(0xffffff, 255).over(Rgba::hex(0xffffff)), Rgba::hex(0xffffff));
    }

    #[test]
    fn dark_tokens_put_each_base_colour_in_its_role() {
        let t = Tokens::DARK;
        assert_eq!(t.rail_bg, Rgba::hex(0x171e28));
        assert_eq!(t.sidebar_bg, Rgba::hex(0x171e28));
        assert_eq!(t.content_bg, Rgba::hex(0x171e28));
        assert_eq!(t.surface, Rgba::hex(0x232b36));
        assert_eq!(t.element, Rgba::hex(0x2a3845));
        assert_eq!(t.accent, Rgba::hex(0x3bbae3));
        assert_eq!(t.ink, Rgba::hex(0x818898));
        assert_eq!(t.terminal.cursor, ACCENT);
        assert_eq!(t.terminal.background, SURFACE);
    }
}
