//! 4x4 matrix (column major).

use std::ops::Mul;

use super::{Vec3, Vec4};

/// 4x4 matrix (column major).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        x: Vec4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
        y: Vec4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
        z: Vec4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
        w: Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
    };

    pub const ZERO: Self = Self {
        x: Vec4::ZERO,
        y: Vec4::ZERO,
        z: Vec4::ZERO,
        w: Vec4::ZERO,
    };

    pub fn translation(v: Vec3) -> Self {
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(v.x, v.y, v.z, 1.0),
        }
    }

    pub fn scale(v: Vec3) -> Self {
        Self {
            x: Vec4::new(v.x, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, v.y, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, v.z, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: Vec4::new(1.0, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, c, s, 0.0),
            z: Vec4::new(0.0, -s, c, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: Vec4::new(c, 0.0, -s, 0.0),
            y: Vec4::new(0.0, 1.0, 0.0, 0.0),
            z: Vec4::new(s, 0.0, c, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: Vec4::new(c, s, 0.0, 0.0),
            y: Vec4::new(-s, c, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, 1.0, 0.0),
            w: Vec4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);
        Self {
            x: Vec4::new(f / aspect, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, f, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, (far + near) * nf, -1.0),
            w: Vec4::new(0.0, 0.0, 2.0 * far * near * nf, 0.0),
        }
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;
        Self {
            x: Vec4::new(2.0 / rml, 0.0, 0.0, 0.0),
            y: Vec4::new(0.0, 2.0 / tmb, 0.0, 0.0),
            z: Vec4::new(0.0, 0.0, -2.0 / fmn, 0.0),
            w: Vec4::new(-(right + left) / rml, -(top + bottom) / tmb, -(far + near) / fmn, 1.0),
        }
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalized();
        let s = f.cross(up).normalized();
        let u = s.cross(f);
        Self {
            x: Vec4::new(s.x, u.x, -f.x, 0.0),
            y: Vec4::new(s.y, u.y, -f.y, 0.0),
            z: Vec4::new(s.z, u.z, -f.z, 0.0),
            w: Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        }
    }

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let v = self.transform_vec4(Vec4::from_vec3(p, 1.0));
        Vec3::new(v.x / v.w, v.y / v.w, v.z / v.w)
    }

    pub fn transform_direction(&self, d: Vec3) -> Vec3 {
        self.transform_vec4(Vec4::from_vec3(d, 0.0)).xyz()
    }

    pub fn transform_vec4(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.x.x * v.x + self.y.x * v.y + self.z.x * v.z + self.w.x * v.w,
            self.x.y * v.x + self.y.y * v.y + self.z.y * v.z + self.w.y * v.w,
            self.x.z * v.x + self.y.z * v.y + self.z.z * v.z + self.w.z * v.w,
            self.x.w * v.x + self.y.w * v.y + self.z.w * v.z + self.w.w * v.w,
        )
    }

    pub fn inverse(&self) -> Self {
        let m = self;
        let s0 = m.x.x * m.y.y - m.y.x * m.x.y;
        let s1 = m.x.x * m.y.z - m.y.x * m.x.z;
        let s2 = m.x.x * m.y.w - m.y.x * m.x.w;
        let s3 = m.x.y * m.y.z - m.y.y * m.x.z;
        let s4 = m.x.y * m.y.w - m.y.y * m.x.w;
        let s5 = m.x.z * m.y.w - m.y.z * m.x.w;
        let c5 = m.z.z * m.w.w - m.w.z * m.z.w;
        let c4 = m.z.y * m.w.w - m.w.y * m.z.w;
        let c3 = m.z.y * m.w.z - m.w.y * m.z.z;
        let c2 = m.z.x * m.w.w - m.w.x * m.z.w;
        let c1 = m.z.x * m.w.z - m.w.x * m.z.z;
        let c0 = m.z.x * m.w.y - m.w.x * m.z.y;
        let det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;
        let inv_det = 1.0 / det;
        Self {
            x: Vec4::new(
                (m.y.y * c5 - m.y.z * c4 + m.y.w * c3) * inv_det,
                (-m.x.y * c5 + m.x.z * c4 - m.x.w * c3) * inv_det,
                (m.w.y * s5 - m.w.z * s4 + m.w.w * s3) * inv_det,
                (-m.z.y * s5 + m.z.z * s4 - m.z.w * s3) * inv_det,
            ),
            y: Vec4::new(
                (-m.y.x * c5 + m.y.z * c2 - m.y.w * c1) * inv_det,
                (m.x.x * c5 - m.x.z * c2 + m.x.w * c1) * inv_det,
                (-m.w.x * s5 + m.w.z * s2 - m.w.w * s1) * inv_det,
                (m.z.x * s5 - m.z.z * s2 + m.z.w * s1) * inv_det,
            ),
            z: Vec4::new(
                (m.y.x * c4 - m.y.y * c2 + m.y.w * c0) * inv_det,
                (-m.x.x * c4 + m.x.y * c2 - m.x.w * c0) * inv_det,
                (m.w.x * s4 - m.w.y * s2 + m.w.w * s0) * inv_det,
                (-m.z.x * s4 + m.z.y * s2 - m.z.w * s0) * inv_det,
            ),
            w: Vec4::new(
                (-m.y.x * c3 + m.y.y * c1 - m.y.z * c0) * inv_det,
                (m.x.x * c3 - m.x.y * c1 + m.x.z * c0) * inv_det,
                (-m.w.x * s3 + m.w.y * s1 - m.w.z * s0) * inv_det,
                (m.z.x * s3 - m.z.y * s1 + m.z.z * s0) * inv_det,
            ),
        }
    }

    pub fn transpose(&self) -> Self {
        Self {
            x: Vec4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            y: Vec4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            z: Vec4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            w: Vec4::new(self.x.w, self.y.w, self.z.w, self.w.w),
        }
    }

    pub fn to_cols_array(&self) -> [[f32; 4]; 4] {
        [
            self.x.to_array(),
            self.y.to_array(),
            self.z.to_array(),
            self.w.to_array(),
        ]
    }
}

impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.transform_vec4(rhs.x),
            y: self.transform_vec4(rhs.y),
            z: self.transform_vec4(rhs.z),
            w: self.transform_vec4(rhs.w),
        }
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        self.transform_vec4(rhs)
    }
}
