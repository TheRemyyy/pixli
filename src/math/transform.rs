//! Transform: position, rotation, scale.

use super::{Mat4, Quat, Vec3};

/// Transform component with position, rotation, and scale.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            scale: Vec3::ONE,
        }
    }

    pub fn from_position_scale(position: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    pub fn from_position_rotation_scale(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        let t = Mat4::translation(self.position);
        let r = self.rotation.to_mat4();
        let s = Mat4::scale(self.scale);
        t * r * s
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation.forward()
    }

    pub fn right(&self) -> Vec3 {
        self.rotation.right()
    }

    pub fn up(&self) -> Vec3 {
        self.rotation.up()
    }

    pub fn translate(&mut self, offset: Vec3) {
        self.position += offset;
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalized();
        let right_cand = up.cross(forward);
        let right = if right_cand.length_squared() < 1e-10 {
            let ref_vec = if forward.x.abs() <= forward.y.abs() && forward.x.abs() <= forward.z.abs() {
                Vec3::RIGHT
            } else if forward.y.abs() <= forward.z.abs() {
                Vec3::UP
            } else {
                Vec3::FORWARD
            };
            forward.cross(ref_vec).normalized()
        } else {
            right_cand.normalized()
        };
        let up_actual = forward.cross(right).normalized();

        let trace = right.x + up_actual.y + forward.z;
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            self.rotation = Quat::new(
                (up_actual.z - forward.y) * s,
                (forward.x - right.z) * s,
                (right.y - up_actual.x) * s,
                0.25 / s,
            );
        } else {
            let max_diag = (right.x, up_actual.y, forward.z);
            if max_diag.0 >= max_diag.1 && max_diag.0 >= max_diag.2 {
                let s = (1.0 + right.x - up_actual.y - forward.z).sqrt() * 2.0;
                if s > 1e-6 {
                    self.rotation = Quat::new(
                        0.25 * s,
                        (right.y + up_actual.x) / s,
                        (right.z + forward.x) / s,
                        (up_actual.z - forward.y) / s,
                    );
                }
            } else if max_diag.1 >= max_diag.2 {
                let s = (1.0 - right.x + up_actual.y - forward.z).sqrt() * 2.0;
                if s > 1e-6 {
                    self.rotation = Quat::new(
                        (right.y + up_actual.x) / s,
                        0.25 * s,
                        (up_actual.z + forward.y) / s,
                        (right.z - forward.x) / s,
                    );
                }
            } else {
                let s = (1.0 - right.x - up_actual.y + forward.z).sqrt() * 2.0;
                if s > 1e-6 {
                    self.rotation = Quat::new(
                        (right.z + forward.x) / s,
                        (up_actual.z + forward.y) / s,
                        0.25 * s,
                        (right.y - up_actual.x) / s,
                    );
                }
            }
        }
    }

    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}
