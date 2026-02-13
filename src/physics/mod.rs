//! Physics: colliders, rigid bodies, collision detection.
//!
//! Uses SIMD (wide crate) in hot path for slide_velocity: v minus (v·n)*n.

mod collider;
mod rigidbody;

pub use collider::{Collider, ColliderShape};
pub use rigidbody::RigidBody;

use crate::ecs::{Entity, World};
use crate::math::Vec3;
use wide::f32x4;

/// Convert Vec3 to f32x4 (x, y, z, 0) for SIMD.
#[inline(always)]
fn vec3_to_simd(v: Vec3) -> f32x4 {
    f32x4::from([v.x, v.y, v.z, 0.0])
}

/// Convert f32x4 (x, y, z, unused) back to Vec3.
#[inline(always)]
fn simd_to_vec3(s: f32x4) -> Vec3 {
    let a: [f32; 4] = bytemuck::cast(s);
    Vec3::new(a[0], a[1], a[2])
}

/// Removes the velocity component into a surface (normal points outward).
/// v_new = v minus (v·n)*n. SIMD: one horizontal add and mul/sub.
#[inline]
fn slide_velocity(velocity: Vec3, normal: Vec3) -> Vec3 {
    let v = vec3_to_simd(velocity);
    let n = vec3_to_simd(normal);
    let n_len_sq = (n * n).reduce_add();
    if n_len_sq <= 1e-12 {
        return velocity;
    }
    let n_len = n_len_sq.sqrt();
    let n = n / f32x4::from([n_len, n_len, n_len, n_len]);
    let dot = (v * n).reduce_add();
    let out = v - n * f32x4::from([dot, dot, dot, dot]);
    simd_to_vec3(out)
}

/// Collision event.
#[derive(Debug, Clone, Copy)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub penetration: f32,
}

/// Physics world.
pub struct Physics {
    pub gravity: Vec3,
    pub collision_events: Vec<CollisionEvent>,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            collision_events: Vec::new(),
        }
    }

    /// Update physics simulation.
    pub fn update(&mut self, world: &mut World, delta: f32) {
        self.collision_events.clear();

        // Collect all entities with RigidBody and Collider.
        let entities: Vec<Entity> = world.query::<(&RigidBody, &Collider)>().iter().collect();
        if entities.is_empty() {
            return;
        }

        // Apply gravity and update velocities.
        for &entity in &entities {
            if let Some(rb) = world.get_mut::<RigidBody>(entity) {
                if !rb.is_kinematic {
                    rb.velocity += self.gravity * delta;
                }
            }
        }

        // Update positions.
        for &entity in &entities {
            let Some(rb) = world.get::<RigidBody>(entity) else {
                continue;
            };
            let velocity = rb.velocity;
            if let Some(transform) = world.get_mut::<crate::math::Transform>(entity) {
                transform.position += velocity * delta;
            }
        }

        // Collision detection (O(n²), simple pairwise approach). Use refs only to compute
        // collision result, then drop them before get_mut for resolution (no Collider clone).
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let entity_a = entities[i];
                let entity_b = entities[j];

                let pos_a = world
                    .get::<crate::math::Transform>(entity_a)
                    .map(|t| t.position);
                let pos_b = world
                    .get::<crate::math::Transform>(entity_b)
                    .map(|t| t.position);
                let collision_result =
                    match (
                        pos_a,
                        pos_b,
                        world.get::<Collider>(entity_a),
                        world.get::<Collider>(entity_b),
                    ) {
                        (Some(pa), Some(pb), Some(ca), Some(cb)) => {
                            ca.collision_info(cb, pa, pb)
                        }
                        _ => None,
                    };

                if let Some((point, normal, penetration)) = collision_result {
                    // Normal points from B toward A (direction to push A out of collision).
                    self.collision_events.push(CollisionEvent {
                        entity_a,
                        entity_b,
                        point,
                        normal,
                        penetration,
                    });

                    // Triggers: no physical response (walk through). Game code can still react via collision_events.
                    let trigger_a = world
                        .get::<Collider>(entity_a)
                        .map(|c| c.is_trigger)
                        .unwrap_or(false);
                    let trigger_b = world
                        .get::<Collider>(entity_b)
                        .map(|c| c.is_trigger)
                        .unwrap_or(false);
                    if trigger_a || trigger_b {
                        continue;
                    }

                    let rb_a_kinematic = world
                        .get::<RigidBody>(entity_a)
                        .map(|r| r.is_kinematic)
                        .unwrap_or(true);
                    let rb_b_kinematic = world
                        .get::<RigidBody>(entity_b)
                        .map(|r| r.is_kinematic)
                        .unwrap_or(true);

                    // 1) Penetration resolution: push entities out along normal so they do not overlap.
                    if !rb_a_kinematic && !rb_b_kinematic {
                        let half_pen = penetration / 2.0;
                        if let Some(t) = world.get_mut::<crate::math::Transform>(entity_a) {
                            t.position += normal * half_pen;
                        }
                        if let Some(t) = world.get_mut::<crate::math::Transform>(entity_b) {
                            t.position -= normal * half_pen;
                        }
                    } else if !rb_a_kinematic {
                        if let Some(t) = world.get_mut::<crate::math::Transform>(entity_a) {
                            t.position += normal * penetration;
                        }
                    } else if !rb_b_kinematic {
                        if let Some(t) = world.get_mut::<crate::math::Transform>(entity_b) {
                            t.position -= normal * penetration;
                        }
                    }

                    // 2) Velocity projection (sliding): remove velocity into the wall so we slide along it.
                    // v_new = v minus (v·n)*n. For A outward normal is n, for B it is minus n.
                    if let Some(rb) = world.get_mut::<RigidBody>(entity_a) {
                        if !rb.is_kinematic {
                            rb.velocity = slide_velocity(rb.velocity, normal);
                        }
                    }
                    if let Some(rb) = world.get_mut::<RigidBody>(entity_b) {
                        if !rb.is_kinematic {
                            rb.velocity = slide_velocity(rb.velocity, -normal);
                        }
                    }
                }
            }
        }
    }

    /// Get collisions for an entity.
    pub fn get_collisions(&self, entity: Entity) -> Vec<&CollisionEvent> {
        self.collision_events
            .iter()
            .filter(|e| e.entity_a == entity || e.entity_b == entity)
            .collect()
    }

    /// Check if entity is colliding with anything.
    pub fn is_colliding(&self, entity: Entity) -> bool {
        self.collision_events
            .iter()
            .any(|e| e.entity_a == entity || e.entity_b == entity)
    }

    /// Raycast into the world. Returns (Entity, hit point, distance, normal).
    pub fn raycast(
        &self,
        world: &World,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(Entity, Vec3, f32, Vec3)> {
        let direction = direction.normalized();
        let mut closest: Option<(Entity, Vec3, f32, Vec3)> = None;

        for entity in world.query::<(&crate::math::Transform, &Collider)>().iter() {
            let transform = world.get::<crate::math::Transform>(entity)?;
            let collider = world.get::<Collider>(entity)?;

            if let Some((hit_point, distance, normal)) =
                collider.raycast(origin, direction, transform.position)
            {
                if distance <= max_distance {
                    let is_closer = match &closest {
                        None => true,
                        Some((_, _, d, _)) => distance < *d,
                    };
                    if is_closer {
                        closest = Some((entity, hit_point, distance, normal));
                    }
                }
            }
        }

        closest
    }
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}
