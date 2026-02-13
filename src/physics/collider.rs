//! Collider shapes.

use crate::math::Vec3;

/// Collider shape type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColliderShape {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
}

/// Collider component.
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec3,
    pub is_trigger: bool, // Triggers do not cause physical response.
}

impl Collider {
    /// Create box collider.
    pub fn box_collider(size: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box {
                half_extents: size * 0.5,
            },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    /// Create sphere collider.
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    /// Create capsule collider.
    pub fn capsule(radius: f32, height: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule { radius, height },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    /// Set as trigger.
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }

    /// Set offset.
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Check intersection with another collider.
    pub fn intersects(&self, other: &Collider, pos_a: Vec3, pos_b: Vec3) -> bool {
        let pos_a = pos_a + self.offset;
        let pos_b = pos_b + other.offset;

        match (&self.shape, &other.shape) {
            (ColliderShape::Box { half_extents: a }, ColliderShape::Box { half_extents: b }) => {
                Self::aabb_aabb(pos_a, *a, pos_b, *b)
            }
            (ColliderShape::Sphere { radius: a }, ColliderShape::Sphere { radius: b }) => {
                Self::sphere_sphere(pos_a, *a, pos_b, *b)
            }
            (ColliderShape::Box { half_extents }, ColliderShape::Sphere { radius }) => {
                Self::aabb_sphere(pos_a, *half_extents, pos_b, *radius)
            }
            (ColliderShape::Sphere { radius }, ColliderShape::Box { half_extents }) => {
                Self::aabb_sphere(pos_b, *half_extents, pos_a, *radius)
            }
            _ => false, // TODO: Capsule collisions
        }
    }

    /// Get collision info (point, normal, penetration).
    /// Normal points from `other` (B) toward `self` (A), direction to push A out of collision.
    pub fn collision_info(
        &self,
        other: &Collider,
        pos_a: Vec3,
        pos_b: Vec3,
    ) -> Option<(Vec3, Vec3, f32)> {
        let pos_a = pos_a + self.offset;
        let pos_b = pos_b + other.offset;

        match (&self.shape, &other.shape) {
            (ColliderShape::Box { half_extents: a }, ColliderShape::Box { half_extents: b }) => {
                Self::aabb_aabb_info(pos_a, *a, pos_b, *b)
            }
            (ColliderShape::Sphere { radius: a }, ColliderShape::Sphere { radius: b }) => {
                Self::sphere_sphere_info(pos_a, *a, pos_b, *b)
            }
            (ColliderShape::Box { half_extents }, ColliderShape::Sphere { radius }) => {
                Self::aabb_sphere_info(pos_a, *half_extents, pos_b, *radius)
            }
            (ColliderShape::Sphere { radius }, ColliderShape::Box { half_extents }) => {
                Self::aabb_sphere_info(pos_b, *half_extents, pos_a, *radius)
                    .map(|(p, n, d)| (p, -n, d))
            }
            _ => None,
        }
    }

    /// Raycast against this collider. Returns (hit point, distance, normal).
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        collider_pos: Vec3,
    ) -> Option<(Vec3, f32, Vec3)> {
        let pos = collider_pos + self.offset;

        match &self.shape {
            ColliderShape::Sphere { radius } => Self::ray_sphere(origin, direction, pos, *radius),
            ColliderShape::Box { half_extents } => {
                Self::ray_aabb(origin, direction, pos, *half_extents)
            }
            _ => None,
        }
    }

    // Collision helper functions.

    fn aabb_aabb(pos_a: Vec3, half_a: Vec3, pos_b: Vec3, half_b: Vec3) -> bool {
        (pos_a.x - half_a.x <= pos_b.x + half_b.x && pos_a.x + half_a.x >= pos_b.x - half_b.x)
            && (pos_a.y - half_a.y <= pos_b.y + half_b.y
                && pos_a.y + half_a.y >= pos_b.y - half_b.y)
            && (pos_a.z - half_a.z <= pos_b.z + half_b.z
                && pos_a.z + half_a.z >= pos_b.z - half_b.z)
    }

    fn sphere_sphere(pos_a: Vec3, radius_a: f32, pos_b: Vec3, radius_b: f32) -> bool {
        let dist_sq = (pos_b - pos_a).length_squared();
        let radius_sum = radius_a + radius_b;
        dist_sq <= radius_sum * radius_sum
    }

    fn aabb_sphere(box_pos: Vec3, half_extents: Vec3, sphere_pos: Vec3, radius: f32) -> bool {
        let closest = Vec3::new(
            sphere_pos
                .x
                .clamp(box_pos.x - half_extents.x, box_pos.x + half_extents.x),
            sphere_pos
                .y
                .clamp(box_pos.y - half_extents.y, box_pos.y + half_extents.y),
            sphere_pos
                .z
                .clamp(box_pos.z - half_extents.z, box_pos.z + half_extents.z),
        );
        (closest - sphere_pos).length_squared() <= radius * radius
    }

    fn aabb_aabb_info(
        pos_a: Vec3,
        half_a: Vec3,
        pos_b: Vec3,
        half_b: Vec3,
    ) -> Option<(Vec3, Vec3, f32)> {
        let overlap_x = (half_a.x + half_b.x) - (pos_b.x - pos_a.x).abs();
        let overlap_y = (half_a.y + half_b.y) - (pos_b.y - pos_a.y).abs();
        let overlap_z = (half_a.z + half_b.z) - (pos_b.z - pos_a.z).abs();

        if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
            return None;
        }

        let (normal, penetration) = if overlap_x < overlap_y && overlap_x < overlap_z {
            (
                Vec3::new(if pos_a.x < pos_b.x { -1.0 } else { 1.0 }, 0.0, 0.0),
                overlap_x,
            )
        } else if overlap_y < overlap_z {
            (
                Vec3::new(0.0, if pos_a.y < pos_b.y { -1.0 } else { 1.0 }, 0.0),
                overlap_y,
            )
        } else {
            (
                Vec3::new(0.0, 0.0, if pos_a.z < pos_b.z { -1.0 } else { 1.0 }),
                overlap_z,
            )
        };

        let point = (pos_a + pos_b) * 0.5;
        Some((point, normal, penetration))
    }

    fn sphere_sphere_info(
        pos_a: Vec3,
        radius_a: f32,
        pos_b: Vec3,
        radius_b: f32,
    ) -> Option<(Vec3, Vec3, f32)> {
        let diff = pos_b - pos_a;
        let dist = diff.length();
        let radius_sum = radius_a + radius_b;

        if dist >= radius_sum {
            return None;
        }

        let normal = if dist > 0.0001 { diff / dist } else { Vec3::UP };
        let penetration = radius_sum - dist;
        let point = pos_a + normal * radius_a;

        Some((point, normal, penetration))
    }

    fn aabb_sphere_info(
        box_pos: Vec3,
        half_extents: Vec3,
        sphere_pos: Vec3,
        radius: f32,
    ) -> Option<(Vec3, Vec3, f32)> {
        let closest = Vec3::new(
            sphere_pos
                .x
                .clamp(box_pos.x - half_extents.x, box_pos.x + half_extents.x),
            sphere_pos
                .y
                .clamp(box_pos.y - half_extents.y, box_pos.y + half_extents.y),
            sphere_pos
                .z
                .clamp(box_pos.z - half_extents.z, box_pos.z + half_extents.z),
        );

        let diff = sphere_pos - closest;
        let dist_sq = diff.length_squared();

        if dist_sq > radius * radius {
            return None;
        }

        let dist = dist_sq.sqrt();
        let normal = if dist > 0.0001 { diff / dist } else { Vec3::UP };
        let penetration = radius - dist;

        Some((closest, normal, penetration))
    }

    fn ray_sphere(
        origin: Vec3,
        direction: Vec3,
        center: Vec3,
        radius: f32,
    ) -> Option<(Vec3, f32, Vec3)> {
        let oc = origin - center;
        let a = direction.dot(direction);
        let b = 2.0 * oc.dot(direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }

        let point = origin + direction * t;
        let normal = (point - center).normalized();
        Some((point, t, normal))
    }

    fn ray_aabb(
        origin: Vec3,
        direction: Vec3,
        center: Vec3,
        half_extents: Vec3,
    ) -> Option<(Vec3, f32, Vec3)> {
        let min = center - half_extents;
        let max = center + half_extents;

        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;
        let mut normal = Vec3::ZERO;

        for i in 0..3 {
            let (o, d, mn, mx) = match i {
                0 => (origin.x, direction.x, min.x, max.x),
                1 => (origin.y, direction.y, min.y, max.y),
                _ => (origin.z, direction.z, min.z, max.z),
            };

            if d.abs() < 0.0001 {
                if o < mn || o > mx {
                    return None;
                }
            } else {
                let t1 = (mn - o) / d;
                let t2 = (mx - o) / d;
                let (t_enter, t_exit) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                if t_enter > tmin {
                    tmin = t_enter;
                    normal = match i {
                        0 => Vec3::new(if d < 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0),
                        1 => Vec3::new(0.0, if d < 0.0 { 1.0 } else { -1.0 }, 0.0),
                        _ => Vec3::new(0.0, 0.0, if d < 0.0 { 1.0 } else { -1.0 }),
                    };
                }
                tmax = tmax.min(t_exit);
                if tmin > tmax {
                    return None;
                }
            }
        }

        if tmin < 0.0 {
            return None;
        }

        let point = origin + direction * tmin;
        Some((point, tmin, normal))
    }
}

impl Default for Collider {
    fn default() -> Self {
        Self::box_collider(Vec3::ONE)
    }
}
