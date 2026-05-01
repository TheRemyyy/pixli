//! Platform-specific runtime integration.

pub mod constants;
pub mod graphics;
pub mod windowing;

pub use graphics::{graphics_backends, is_supported_backend, select_present_mode};
pub use windowing::{apply_platform_window_attributes, capture_cursor, release_cursor};
