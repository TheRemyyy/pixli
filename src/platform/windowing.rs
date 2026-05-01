//! Native window integration helpers.

use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowAttributes};

use crate::error::{Error, Result};
use crate::window::WindowConfig;

/// Apply cross-desktop window metadata and platform-specific hints.
pub fn apply_platform_window_attributes(
    attributes: WindowAttributes,
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
) -> WindowAttributes {
    let attributes = attributes
        .with_decorations(config.decorated)
        .with_fullscreen(config.fullscreen.then(|| {
            let monitor = event_loop.primary_monitor();
            Fullscreen::Borderless(monitor)
        }));

    #[cfg(target_os = "linux")]
    {
        let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(
            attributes,
            config.app_id.clone(),
            config.app_id.clone(),
        );
        winit::platform::x11::WindowAttributesExtX11::with_name(
            attributes,
            config.app_id.clone(),
            config.app_id.clone(),
        )
    }

    #[cfg(not(target_os = "linux"))]
    {
        attributes
    }
}

/// Release any active pointer grab and make the cursor visible.
pub fn release_cursor(window: &Window) -> Result<()> {
    window
        .set_cursor_grab(CursorGrabMode::None)
        .map_err(|err| Error::Window(format!("release cursor grab: {err}")))?;
    window.set_cursor_visible(true);
    Ok(())
}

/// Capture the cursor for FPS-style controls.
pub fn capture_cursor(window: &Window) -> Result<()> {
    let grab_result = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));

    match grab_result {
        Ok(()) => {
            window.set_cursor_visible(false);
            Ok(())
        }
        Err(err) => Err(Error::Window(format!("capture cursor: {err}"))),
    }
}
