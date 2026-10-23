//! Colour tokens, shared by every CloudNode surface: the Slint window, the Ratatui interface and
//! the terminal renderer read the same values, so CloudNode looks the same in a window and in a
//! console. Nothing outside this module writes a colour literal.
//!
//! The look follows Apple's dark appearance: monochrome dark greys, hairline separators instead
//! of shadows, and one accent, TradeCloud orange, saved for focus, selection and the active
//! session.

/// TradeCloud orange.
///
/// Placeholder until the brand's exact value is supplied: change it here and every surface follows.
pub const TRADECLOUD_ORANGE: Rgba = Rgba::hex(0xff7a1a);

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
    /// Behind everything: the title bar and the tab strip.
    pub window_bg: Rgba,
    /// The sidebar.
    pub sidebar_bg: Rgba,
    /// Raised surfaces: menus, popovers, the active tab.
    pub surface: Rgba,
    /// Text fields and wells.
    pub field_bg: Rgba,
    /// A row or button under the pointer.
    pub hover: Rgba,
    /// Body text.
    pub ink: Rgba,
    /// Headings and the active tab's title.
    pub ink_strong: Rgba,
    /// Secondary text.
    pub muted: Rgba,
    /// Section headers, placeholders and disabled text.
    pub faint: Rgba,
    /// Hairline separators.
    pub line: Rgba,
    /// Field borders and stronger separators.
    pub line_strong: Rgba,
    /// TradeCloud orange: focus rings, primary buttons, the active session.
    pub accent: Rgba,
    /// The accent under the pointer.
    pub accent_hover: Rgba,
    /// Text and icons in the accent colour on dark surfaces.
    pub accent_ink: Rgba,
    /// Text on an accent surface.
    pub on_accent: Rgba,
    /// Selected rows.
    pub selection: Rgba,
    pub good: Rgba,
    pub warn: Rgba,
    pub bad: Rgba,
    pub terminal: TerminalPalette,
}

impl Tokens {
    /// The dark appearance, CloudNode's default.
    pub const DARK: Self = Self {
        window_bg: Rgba::hex(0x18181a),
        sidebar_bg: Rgba::hex(0x1f1f22),
        surface: Rgba::hex(0x2a2a2e),
        field_bg: Rgba::hex(0x141416),
        hover: Rgba::hex_alpha(0xffffff, 14),
        ink: Rgba::hex(0xe6e6e8),
        ink_strong: Rgba::hex(0xffffff),
        muted: Rgba::hex_alpha(0xebebf5, 153),
        faint: Rgba::hex_alpha(0xebebf5, 77),
        line: Rgba::hex_alpha(0xffffff, 20),
        line_strong: Rgba::hex_alpha(0xffffff, 41),
        accent: TRADECLOUD_ORANGE,
        accent_hover: Rgba::hex(0xff8f3d),
        accent_ink: Rgba::hex(0xff9a52),
        on_accent: Rgba::hex(0x1a1a1c),
        selection: TRADECLOUD_ORANGE.with_alpha(56),
        good: Rgba::hex(0x32d74b),
        warn: Rgba::hex(0xffd60a),
        bad: Rgba::hex(0xff453a),
        terminal: TerminalPalette {
            foreground: Rgba::hex(0xe6e6e8),
            background: Rgba::hex(0x111113),
            cursor: TRADECLOUD_ORANGE,
            selection: TRADECLOUD_ORANGE.with_alpha(77),
            ansi: [
                Rgba::hex(0x1c1c1f),
                Rgba::hex(0xff5f57),
                Rgba::hex(0x5fd38d),
                Rgba::hex(0xf5c451),
                Rgba::hex(0x5aa9ff),
                Rgba::hex(0xc792ea),
                Rgba::hex(0x5ccfe6),
                Rgba::hex(0xc8c8cc),
                Rgba::hex(0x5a5a60),
                Rgba::hex(0xff7b73),
                Rgba::hex(0x7ee2a6),
                Rgba::hex(0xffd479),
                Rgba::hex(0x82bfff),
                Rgba::hex(0xd9aef5),
                Rgba::hex(0x86ddf0),
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
    fn dark_tokens_use_the_brand_accent() {
        assert_eq!(Tokens::DARK.accent, TRADECLOUD_ORANGE);
        assert_eq!(Tokens::DARK.terminal.cursor, TRADECLOUD_ORANGE);
    }
}
