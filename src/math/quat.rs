//! Quaternion for 3D rotations.

use std::ops::Mul;

use super::{Mat4, Vec3, Vec4};

/// Quaternion for rotations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Create quaternion from axis and angle (radians).
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();
        let axis = axis.normalized();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: half_angle.cos(),
        }
    }

    /// Create quaternion from Euler angles (pitch, yaw, roll) in radians.
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        let (sp, cp) = (pitch * 0.5).sin_cos();
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sr, cr) = (roll * 0.5).sin_cos();

        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    /// Convert to Euler angles (pitch, yaw, roll).
    pub fn to_euler(&self) -> (f32, f32, f32) {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (pitch, yaw, roll)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            Self::IDENTITY
        }
    }

    #[inline]
    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        self.conjugate().normalized()
    }

    /// Rotate a vector by this quaternion.
    pub fn rotate(&self, v: Vec3) -> Vec3 {
        let q_vec = Vec3::new(self.x, self.y, self.z);
        let uv = q_vec.cross(v);
        let uuv = q_vec.cross(uv);
        v + (uv * self.w + uuv) * 2.0
    }

    /// Spherical linear interpolation.
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        let other = if dot < 0.0 {
            dot = -dot;
            Self::new(-other.x, -other.y, -other.z, -other.w)
        } else {
            other
        };

        if dot > 0.9995 {
            Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            }
            .normalized()
        } else {
            let theta_0 = dot.acos();
            let theta = theta_0 * t;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
            let s1 = sin_theta / sin_theta_0;

            Self {
                x: self.x * s0 + other.x * s1,
                y: self.y * s0 + other.y * s1,
                z: self.z * s0 + other.z * s1,
                w: self.w * s0 + other.w * s1,
            }
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.rotate(Vec3::FORWARD)
    }

    pub fn right(&self) -> Vec3 {
        self.rotate(Vec3::RIGHT)
    }

    pub fn up(&self) -> Vec3 {
        self.rotate(Vec3::UP)
    }

    /// Build quaternion from camera axes: right, up, forward.
    pub fn from_rotation_axes(right: Vec3, up: Vec3, forward: Vec3) -> Self {
        let r = right.normalized();
        let u = up.normalized();
        let f = forward.normalized();
        let (m00, m01, m02) = (r.x, u.x, -f.x);
        let (m10, m11, m12) = (r.y, u.y, -f.y);
        let (m20, m21, m22) = (r.z, u.z, -f.z);
        let trace = m00 + m11 + m22;
        if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            Self::new((m21 - m12) * s, (m02 - m20) * s, (m10 - m01) * s, 0.25 / s)
        } else if m00 > m11 && m00 > m22 {
            let s = 2.0 * (1.0 + m00 - m11 - m22).sqrt();
            Self::new(0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
        } else if m11 > m22 {
            let s = 2.0 * (1.0 - m00 + m11 - m22).sqrt();
            Self::new((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
        } else {
            let s = 2.0 * (1.0 - m00 - m11 + m22).sqrt();
            Self::new((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
        }
    }

    pub fn to_mat4(&self) -> Mat4 {
        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;

        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        Mat4 {
            x: Vec4::new(1.0 - (yy + zz), xy + wz, xz - wy, 0.0),
            y: Vec4::new(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0),
            z: Vec4::new(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl Mul for Quat {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}
