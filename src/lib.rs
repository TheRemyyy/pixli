//! # Pixli
//!
//! A simple yet powerful 3D game engine written in Rust.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use pixli::prelude::*;
//!
//! fn main() -> pixli::Result<()> {
//!     App::new()
//!         .with_title("My Game")
//!         .with_size(1280, 720)
//!         .add_startup_system(setup)
//!         .add_system(update)
//!         .run()
//! }
//!
//! fn setup(world: &mut World, renderer: &mut Renderer) {
//!     // Spawn a cube
//!     world.spawn()
//!         .with(Transform::from_position(Vec3::ZERO))
//!         .with(Mesh::cube(1.0))
//!         .with(Material::color(Color::RED));
//!
//!     // Setup camera
//!     renderer.camera.position = Vec3::new(0.0, 2.0, 5.0);
//! }
//!
//! fn update(state: &mut GameState) {
//!     // Use state.world, state.input, state.time, state.renderer, etc.
//! }
//! ```

pub mod app;
pub mod audio;
pub mod ecs;
pub mod error;
pub mod input;
pub mod math;
pub mod physics;
pub mod renderer;
pub mod time;
pub mod window;

pub use error::{Error, Result};

/// Prelude: import common types with `use pixli::prelude::*`.
pub mod prelude {
    pub use crate::app::{App, GameState, System, StartupSystem};
    pub use crate::error::{Error, Result};
    pub use crate::audio::{Audio, AudioSource, Sound};
    pub use crate::ecs::{Component, Entity, World, EntityBuilder, Query};
    pub use crate::input::{Input, KeyCode, MouseButton};
    pub use crate::math::{Vec2, Vec3, Vec4, Mat4, Quat, Transform, Color};
    pub use crate::physics::{Physics, Collider, RigidBody, CollisionEvent};
    pub use crate::renderer::{Renderer, Camera, Mesh, Material, Texture, Light, LightType, UnlitVertex, UnlitMesh, UnlitMeshRef};
    pub use crate::time::Time;
    pub use crate::window::Window;
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let c = a + b;
        assert_eq!(c.x, 5.0);
        assert_eq!(c.y, 7.0);
        assert_eq!(c.z, 9.0);
    }

    #[test]
    fn test_vec3_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(a.dot(b), 0.0);
    }

    #[test]
    fn test_vec3_cross() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let c = a.cross(b);
        assert_eq!(c.z, 1.0);
    }

    #[test]
    fn test_vec3_normalize() {
        let a = Vec3::new(3.0, 0.0, 4.0);
        let n = a.normalized();
        assert!((n.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_transform() {
        let mut t = Transform::new();
        t.position = Vec3::new(1.0, 2.0, 3.0);
        t.scale = Vec3::splat(2.0);
        let matrix = t.matrix();
        assert!(matrix.w.x == 1.0);
    }

    #[test]
    fn test_color() {
        let c = Color::rgb(255, 128, 0);
        assert_eq!(c.r, 1.0);
        assert!((c.g - 0.5).abs() < 0.01);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_entity_spawn() {
        let mut world = World::new();
        let e = world.spawn().with(Transform::new()).build();
        assert!(world.has::<Transform>(e));
    }

    #[test]
    fn test_entity_despawn() {
        let mut world = World::new();
        let e = world.spawn().with(Transform::new()).build();
        world.despawn(e);
        assert!(!world.is_alive(e));
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        world.spawn().with(Transform::new()).with(Mesh::cube(1.0)).build();
        world.spawn().with(Transform::new()).build();
        
        let query = world.query::<(&Transform, &Mesh)>();
        assert_eq!(query.count(), 1);
    }

    #[test]
    fn test_physics_aabb_collision() {
        let a = Collider::box_collider(Vec3::splat(1.0));
        let b = Collider::box_collider(Vec3::splat(1.0));
        
        let pos_a = Vec3::ZERO;
        let pos_b = Vec3::new(0.5, 0.0, 0.0);
        
        assert!(a.intersects(&b, pos_a, pos_b));
    }

    #[test]
    fn test_physics_sphere_collision() {
        let a = Collider::sphere(1.0);
        let b = Collider::sphere(1.0);
        
        let pos_a = Vec3::ZERO;
        let pos_b = Vec3::new(1.5, 0.0, 0.0);
        
        assert!(a.intersects(&b, pos_a, pos_b));
    }

    #[test]
    fn test_mesh_vertex_count() {
        let cube = Mesh::cube(1.0);
        assert_eq!(cube.vertices.len(), 36); // 6 faces * 2 triangles * 3 vertices
    }

    #[test]
    fn test_time() {
        let mut time = Time::new();
        time.update(0.016);
        assert!((time.delta() - 0.016).abs() < 0.0001);
        assert_eq!(time.frame_count(), 1);
    }
}
