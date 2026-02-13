//! Lights: directional, point, spot.

use crate::math::{Color, Vec3};

/// Light type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Light component.
#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub color: Color,
    pub intensity: f32,
    pub position: Vec3,
    pub direction: Vec3,
    pub range: f32,      // For point and spot lights.
    pub spot_angle: f32, // For spot lights (radians).
}

impl Light {
    /// Create directional light.
    pub fn directional(direction: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            color,
            intensity,
            position: Vec3::ZERO,
            direction: direction.normalized(),
            range: 0.0,
            spot_angle: 0.0,
        }
    }

    /// Create point light.
    pub fn point(position: Vec3, color: Color, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            color,
            intensity,
            position,
            direction: Vec3::DOWN,
            range,
            spot_angle: 0.0,
        }
    }

    /// Create spot light.
    pub fn spot(
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        angle_degrees: f32,
    ) -> Self {
        Self {
            light_type: LightType::Spot,
            color,
            intensity,
            position,
            direction: direction.normalized(),
            range,
            spot_angle: angle_degrees.to_radians(),
        }
    }

    /// Create sun light (directional pointing down).
    pub fn sun(color: Color, intensity: f32) -> Self {
        Self::directional(Vec3::new(-0.5, -1.0, -0.3), color, intensity)
    }

    /// Set color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    /// Calculate attenuation at distance (for point and spot lights).
    pub fn attenuation(&self, distance: f32) -> f32 {
        if self.light_type == LightType::Directional {
            return 1.0;
        }

        let d = distance / self.range;
        (1.0 - d * d).max(0.0)
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::sun(Color::WHITE, 1.0)
    }
}
