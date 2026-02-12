//! Material: surface properties.

use crate::math::Color;

/// Material component.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Color,
    pub emission_strength: f32,
}

impl Material {
    pub fn new() -> Self {
        Self {
            color: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emission: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Create solid color material.
    pub fn color(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Create metallic material.
    pub fn metallic(color: Color, metallic: f32) -> Self {
        Self {
            color,
            metallic,
            roughness: 0.2,
            ..Default::default()
        }
    }

    /// Create emissive material.
    pub fn emissive(color: Color, strength: f32) -> Self {
        Self {
            color: Color::BLACK,
            emission: color,
            emission_strength: strength,
            ..Default::default()
        }
    }

    /// Set color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set metallic.
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// Set roughness.
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Set emission.
    pub fn with_emission(mut self, color: Color, strength: f32) -> Self {
        self.emission = color;
        self.emission_strength = strength;
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

/// Common materials.
impl Material {
    pub fn red() -> Self { Self::color(Color::RED) }
    pub fn green() -> Self { Self::color(Color::GREEN) }
    pub fn blue() -> Self { Self::color(Color::BLUE) }
    pub fn yellow() -> Self { Self::color(Color::YELLOW) }
    pub fn cyan() -> Self { Self::color(Color::CYAN) }
    pub fn magenta() -> Self { Self::color(Color::MAGENTA) }
    pub fn white() -> Self { Self::color(Color::WHITE) }
    pub fn black() -> Self { Self::color(Color::BLACK) }
    pub fn gray() -> Self { Self::color(Color::GRAY) }
    pub fn orange() -> Self { Self::color(Color::ORANGE) }
    pub fn purple() -> Self { Self::color(Color::PURPLE) }
}
