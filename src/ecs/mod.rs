//! Entity Component System
//!
//! Simple but effective ECS for game objects.

mod entity;
mod component;
mod world;

pub use entity::Entity;
pub use component::Component;
pub use world::{World, EntityBuilder, Query};
