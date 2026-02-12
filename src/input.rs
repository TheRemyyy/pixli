//! Input handling: keyboard, mouse, gamepad.

use std::collections::HashSet;
use crate::math::Vec2;

/// Re-export of winit key codes.
pub use winit::keyboard::KeyCode;

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Back,
            winit::event::MouseButton::Forward => MouseButton::Forward,
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

/// Input state.
pub struct Input {
    // Keyboard
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,

    // Mouse
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    scroll_delta: Vec2,

    // Mouse capture state
    mouse_captured: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            mouse_captured: false,
        }
    }

    /// Call at the start of each frame to clear "just" states
    pub fn update(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
    }

    // ========== Keyboard ==========

    /// Check if a key is currently held down
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    /// Check if any of the given keys is pressed
    pub fn any_key_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|k| self.key_pressed(*k))
    }

    /// Check if all of the given keys are pressed
    pub fn all_keys_pressed(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|k| self.key_pressed(*k))
    }

    /// Get horizontal axis (-1, 0, 1) from A/D or Left/Right
    pub fn axis_horizontal(&self) -> f32 {
        let mut axis = 0.0;
        if self.key_pressed(KeyCode::KeyA) || self.key_pressed(KeyCode::ArrowLeft) {
            axis -= 1.0;
        }
        if self.key_pressed(KeyCode::KeyD) || self.key_pressed(KeyCode::ArrowRight) {
            axis += 1.0;
        }
        axis
    }

    /// Get vertical axis (-1, 0, 1) from W/S or Up/Down
    pub fn axis_vertical(&self) -> f32 {
        let mut axis = 0.0;
        if self.key_pressed(KeyCode::KeyS) || self.key_pressed(KeyCode::ArrowDown) {
            axis -= 1.0;
        }
        if self.key_pressed(KeyCode::KeyW) || self.key_pressed(KeyCode::ArrowUp) {
            axis += 1.0;
        }
        axis
    }

    /// Get movement vector from WASD/Arrow keys
    pub fn movement_vector(&self) -> Vec2 {
        Vec2::new(self.axis_horizontal(), self.axis_vertical())
    }

    /// Get normalized movement vector
    pub fn movement_vector_normalized(&self) -> Vec2 {
        let v = self.movement_vector();
        if v.length_squared() > 0.0 {
            v.normalized()
        } else {
            Vec2::ZERO
        }
    }

    // ========== Mouse ==========

    /// Check if a mouse button is currently held down
    pub fn mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Check if a mouse button was just pressed this frame
    pub fn mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    /// Check if a mouse button was just released this frame
    pub fn mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    /// Get mouse position in window coordinates
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Get mouse movement delta since last frame
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Get scroll wheel delta
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    /// Check if mouse is captured (for FPS-style controls)
    pub fn is_mouse_captured(&self) -> bool {
        self.mouse_captured
    }

    // ========== Internal event handlers ==========

    pub(crate) fn on_key_pressed(&mut self, key: KeyCode) {
        if !self.keys_pressed.contains(&key) {
            self.keys_just_pressed.insert(key);
        }
        self.keys_pressed.insert(key);
    }

    pub(crate) fn on_key_released(&mut self, key: KeyCode) {
        self.keys_pressed.remove(&key);
        self.keys_just_released.insert(key);
    }

    pub(crate) fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        if !self.mouse_buttons_pressed.contains(&button) {
            self.mouse_buttons_just_pressed.insert(button);
        }
        self.mouse_buttons_pressed.insert(button);
    }

    pub(crate) fn on_mouse_button_released(&mut self, button: MouseButton) {
        self.mouse_buttons_pressed.remove(&button);
        self.mouse_buttons_just_released.insert(button);
    }

    pub(crate) fn on_mouse_moved(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    pub(crate) fn on_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta = Vec2::new(dx, dy);
    }

    pub(crate) fn on_scroll(&mut self, dx: f32, dy: f32) {
        self.scroll_delta = Vec2::new(dx, dy);
    }

    pub(crate) fn set_mouse_captured(&mut self, captured: bool) {
        self.mouse_captured = captured;
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
