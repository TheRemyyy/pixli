//! Window management.

use crate::math::Vec2;

/// Window configuration.
#[derive(Clone)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub vsync: bool,
    pub decorated: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Pixel Engine".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            fullscreen: false,
            vsync: false,
            decorated: true,
        }
    }
}

impl WindowConfig {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }
}

/// Window state.
pub struct Window {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
    pub focused: bool,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
            focused: true,
        }
    }

    /// Get window size as Vec2.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Get aspect ratio.
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Get center position.
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.width as f32 / 2.0, self.height as f32 / 2.0)
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new(1280, 720)
    }
}
