//! 2D vector.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0 };
    pub const DOWN: Self = Self { x: 0.0, y: -1.0 };
    pub const LEFT: Self = Self { x: -1.0, y: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            *self / len
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    #[inline]
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    #[inline]
    pub fn from_angle(angle: f32) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    #[inline]
    pub fn perpendicular(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn to_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self * rhs.x, self * rhs.y)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
