//! Rigid body component.

use crate::math::Vec3;

/// Rigid body component.
#[derive(Debug, Clone, Copy)]
pub struct RigidBody {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub is_kinematic: bool, // Kinematic bodies are not affected by physics.
    pub use_gravity: bool,
    pub freeze_rotation: bool,
}

impl RigidBody {
    pub fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            drag: 0.0,
            angular_drag: 0.05,
            is_kinematic: false,
            use_gravity: true,
            freeze_rotation: false,
        }
    }

    /// Create dynamic rigid body.
    pub fn dynamic() -> Self {
        Self::new()
    }

    /// Create kinematic rigid body (controlled by code, not by physics).
    pub fn kinematic() -> Self {
        Self {
            is_kinematic: true,
            use_gravity: false,
            ..Default::default()
        }
    }

    /// Create static rigid body (does not move).
    pub fn statik() -> Self {
        Self {
            is_kinematic: true,
            use_gravity: false,
            mass: f32::INFINITY,
            ..Default::default()
        }
    }

    /// Set mass.
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass.max(0.001);
        self
    }

    /// Set drag.
    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag = drag.max(0.0);
        self
    }

    /// Set initial velocity.
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Disable gravity.
    pub fn without_gravity(mut self) -> Self {
        self.use_gravity = false;
        self
    }

    /// Freeze rotation.
    pub fn freeze_rotation(mut self) -> Self {
        self.freeze_rotation = true;
        self
    }

    /// Apply force (continuous).
    pub fn add_force(&mut self, force: Vec3) {
        if !self.is_kinematic {
            self.velocity += force / self.mass;
        }
    }

    /// Apply impulse (instant).
    pub fn add_impulse(&mut self, impulse: Vec3) {
        if !self.is_kinematic {
            self.velocity += impulse / self.mass;
        }
    }

    /// Apply force at position (causes rotation).
    pub fn add_force_at_position(&mut self, force: Vec3, position: Vec3, center_of_mass: Vec3) {
        if !self.is_kinematic {
            self.velocity += force / self.mass;

            if !self.freeze_rotation {
                let torque = (position - center_of_mass).cross(force);
                self.angular_velocity += torque / self.mass;
            }
        }
    }

    /// Apply torque.
    pub fn add_torque(&mut self, torque: Vec3) {
        if !self.is_kinematic && !self.freeze_rotation {
            self.angular_velocity += torque / self.mass;
        }
    }

    /// Apply drag.
    pub fn apply_drag(&mut self, delta: f32) {
        if self.drag > 0.0 {
            let drag_factor = 1.0 - self.drag * delta;
            self.velocity *= drag_factor.max(0.0);
        }
        if self.angular_drag > 0.0 {
            let drag_factor = 1.0 - self.angular_drag * delta;
            self.angular_velocity *= drag_factor.max(0.0);
        }
    }

    /// Stop all movement.
    pub fn stop(&mut self) {
        self.velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
    }

    /// Get speed (magnitude of velocity).
    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }

    /// Get kinetic energy.
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    /// Get momentum.
    pub fn momentum(&self) -> Vec3 {
        self.velocity * self.mass
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new()
    }
}
