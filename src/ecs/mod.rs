//! Entity Component System
//!
//! Simple but effective ECS for game objects.

mod component;
mod entity;
mod world;

pub use component::Component;
pub use entity::Entity;
pub use world::{EntityBuilder, Query, World};
