//! The window's frame. On macOS the title bar is transparent, its title hidden, and the content
//! runs under it, so the window buttons sit over the top of the sidebar as in TradeCloud's design.
//! The sidebar and the session area keep `TITLEBAR_HEIGHT` free at the top for the buttons and
//! for dragging the window (`Chrome.titlebar-inset` in `ui/theme.slint`). Windows and Linux keep
//! their native title bars.

use slint::PlatformError;

use crate::MainWindow;

/// Room kept at the top of the window for macOS's title bar (28 pt) and a little air under the
/// window buttons.
#[cfg(target_os = "macos")]
const TITLEBAR_HEIGHT: f32 = 32.0;

/// Chooses Slint's backend. On macOS that is winit, asked for a transparent title bar over
/// full-size content; elsewhere Slint picks as usual. Must run before the first window is made.
pub fn select_backend() -> Result<(), PlatformError> {
    #[cfg(target_os = "macos")]
    {
        use slint::winit_030::winit::platform::macos::WindowAttributesExtMacOS;

        slint::BackendSelector::new()
            .backend_name("winit".into())
            .with_winit_window_attributes_hook(|attributes| {
                attributes.with_titlebar_transparent(true).with_title_hidden(true).with_fullsize_content_view(true)
            })
            .select()?;
    }
    Ok(())
}

/// Tells the layout how much room the title bar needs where it lies over the content.
pub fn apply(window: &MainWindow) {
    #[cfg(target_os = "macos")]
    {
        use slint::ComponentHandle;

        window.global::<crate::Chrome>().set_titlebar_inset(TITLEBAR_HEIGHT);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = window;
}
