//! World: container for all entities and components.
//!
//! Component storage uses Vec<Option<>> indexed by entity id (sparse set style)
//! for cache friendly iteration and O(1) get/set by id.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::{Entity, Component};

/// Storage for a single component type. Indexed by entity id (no hash, cache friendly).
pub(crate) struct ComponentStorage {
    data: Vec<Option<Box<dyn Any + Send + Sync>>>,
}

impl ComponentStorage {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn insert<T: Component>(&mut self, entity_id: u32, component: T) {
        let i = entity_id as usize;
        while self.data.len() <= i {
            self.data.push(None);
        }
        self.data[i] = Some(Box::new(component));
    }

    fn get<T: Component>(&self, entity_id: u32) -> Option<&T> {
        self.data.get(entity_id as usize)?.as_ref()?.downcast_ref()
    }

    fn get_mut<T: Component>(&mut self, entity_id: u32) -> Option<&mut T> {
        self.data.get_mut(entity_id as usize)?.as_mut()?.downcast_mut()
    }

    fn remove(&mut self, entity_id: u32) {
        if let Some(slot) = self.data.get_mut(entity_id as usize) {
            *slot = None;
        }
    }

    fn contains(&self, entity_id: u32) -> bool {
        self.data
            .get(entity_id as usize)
            .map(|o| o.is_some())
            .unwrap_or(false)
    }

    /// Number of entities that have this component.
    fn len(&self) -> usize {
        self.data.iter().filter(|o| o.is_some()).count()
    }

    fn entity_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, o)| if o.is_some() { Some(i as u32) } else { None })
    }
}

/// Entity slot for tracking alive or dead entities.
struct EntitySlot {
    generation: u32,
    alive: bool,
}

/// World: the ECS container.
pub struct World {
    entities: Vec<EntitySlot>,
    free_indices: Vec<u32>,
    components: HashMap<TypeId, ComponentStorage>,
    next_id: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_indices: Vec::new(),
            components: HashMap::new(),
            next_id: 0,
        }
    }

    /// Spawn a new entity.
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let (id, generation) = if let Some(id) = self.free_indices.pop() {
            let slot = &mut self.entities[id as usize];
            slot.generation += 1;
            slot.alive = true;
            (id, slot.generation)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.entities.push(EntitySlot {
                generation: 0,
                alive: true,
            });
            (id, 0)
        };

        EntityBuilder {
            world: self,
            entity: Entity::new(id, generation),
        }
    }

    /// Despawn an entity.
    pub fn despawn(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }

        // Mark as dead.
        self.entities[entity.id as usize].alive = false;
        self.free_indices.push(entity.id);

        // Remove all components.
        for storage in self.components.values_mut() {
            storage.remove(entity.id);
        }
    }

    /// Check if entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        if let Some(slot) = self.entities.get(entity.id as usize) {
            slot.alive && slot.generation == entity.generation
        } else {
            false
        }
    }

    /// Add a component to an entity.
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.is_alive(entity) {
            return;
        }

        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(ComponentStorage::new);
        storage.insert(entity.id, component);
    }

    /// Remove a component from an entity.
    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        if !self.is_alive(entity) {
            return;
        }

        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.remove(entity.id);
        }
    }

    /// Get a component.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)?.get(entity.id)
    }

    /// Get a mutable component.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_alive(entity) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)?.get_mut(entity.id)
    }

    /// Check if entity has a component.
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .map(|s| s.contains(entity.id))
            .unwrap_or(false)
    }

    /// Query entities with specific components.
    pub fn query<Q: QueryTuple>(&self) -> Query<'_, Q> {
        Query::new(self)
    }

    /// Get all alive entities.
    pub fn entities(&self) -> Vec<Entity> {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.alive)
            .map(|(id, slot)| Entity::new(id as u32, slot.generation))
            .collect()
    }

    /// Get entity count.
    pub fn entity_count(&self) -> usize {
        self.entities.iter().filter(|s| s.alive).count()
    }

    /// Clear all entities.
    pub fn clear(&mut self) {
        self.entities.clear();
        self.free_indices.clear();
        self.components.clear();
        self.next_id = 0;
    }

    // Internal: get storage for type.
    pub(crate) fn get_storage<T: Component>(&self) -> Option<&ComponentStorage> {
        self.components.get(&TypeId::of::<T>())
    }

    #[allow(dead_code)]
    pub(crate) fn get_storage_mut<T: Component>(&mut self) -> Option<&mut ComponentStorage> {
        self.components.get_mut(&TypeId::of::<T>())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for spawning entities with components.
pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    /// Add a component
    pub fn with<T: Component>(self, component: T) -> Self {
        self.world.add_component(self.entity, component);
        self
    }

    /// Finish building and return the entity.
    pub fn build(self) -> Entity {
        self.entity
    }
}

/// Query for entities with specific components.
pub struct Query<'a, Q: QueryTuple> {
    world: &'a World,
    _marker: std::marker::PhantomData<Q>,
}

impl<'a, Q: QueryTuple> Query<'a, Q> {
    fn new(world: &'a World) -> Self {
        Self {
            world,
            _marker: std::marker::PhantomData,
        }
    }

    /// Count matching entities.
    pub fn count(&self) -> usize {
        Q::count(self.world)
    }

    /// Iterate over matching entities.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + 'a {
        Q::iter(self.world)
    }
}

/// Trait for query tuples.
pub trait QueryTuple {
    fn count(world: &World) -> usize;
    fn iter(world: &World) -> Box<dyn Iterator<Item = Entity> + '_>;
}

// Implement for single component.
impl<A: Component> QueryTuple for (&A,) {
    fn count(world: &World) -> usize {
        world.get_storage::<A>().map(|s| s.len()).unwrap_or(0)
    }

    fn iter(world: &World) -> Box<dyn Iterator<Item = Entity> + '_> {
        if let Some(storage) = world.get_storage::<A>() {
            Box::new(storage.entity_ids().filter_map(move |id| {
                let slot = world.entities.get(id as usize)?;
                if slot.alive {
                    Some(Entity::new(id, slot.generation))
                } else {
                    None
                }
            }))
        } else {
            Box::new(std::iter::empty())
        }
    }
}

// Implement for two components.
impl<A: Component, B: Component> QueryTuple for (&A, &B) {
    fn count(world: &World) -> usize {
        let storage_a = world.get_storage::<A>();
        let storage_b = world.get_storage::<B>();

        match (storage_a, storage_b) {
            (Some(a), Some(b)) => {
                a.entity_ids()
                    .filter(|id| b.contains(*id))
                    .filter(|id| {
                        world.entities.get(*id as usize)
                            .map(|s| s.alive)
                            .unwrap_or(false)
                    })
                    .count()
            }
            _ => 0,
        }
    }

    fn iter(world: &World) -> Box<dyn Iterator<Item = Entity> + '_> {
        let storage_a = world.get_storage::<A>();
        let storage_b = world.get_storage::<B>();

        match (storage_a, storage_b) {
            (Some(a), Some(b)) => {
                Box::new(a.entity_ids().filter_map(move |id| {
                    if !b.contains(id) {
                        return None;
                    }
                    let slot = world.entities.get(id as usize)?;
                    if slot.alive {
                        Some(Entity::new(id, slot.generation))
                    } else {
                        None
                    }
                }))
            }
            _ => Box::new(std::iter::empty()),
        }
    }
}

// Implement for three components.
impl<A: Component, B: Component, C: Component> QueryTuple for (&A, &B, &C) {
    fn count(world: &World) -> usize {
        let storage_a = world.get_storage::<A>();
        let storage_b = world.get_storage::<B>();
        let storage_c = world.get_storage::<C>();

        match (storage_a, storage_b, storage_c) {
            (Some(a), Some(b), Some(c)) => {
                a.entity_ids()
                    .filter(|id| b.contains(*id) && c.contains(*id))
                    .filter(|id| {
                        world.entities.get(*id as usize)
                            .map(|s| s.alive)
                            .unwrap_or(false)
                    })
                    .count()
            }
            _ => 0,
        }
    }

    fn iter(world: &World) -> Box<dyn Iterator<Item = Entity> + '_> {
        let storage_a = world.get_storage::<A>();
        let storage_b = world.get_storage::<B>();
        let storage_c = world.get_storage::<C>();

        match (storage_a, storage_b, storage_c) {
            (Some(a), Some(b), Some(c)) => {
                Box::new(a.entity_ids().filter_map(move |id| {
                    if !b.contains(id) || !c.contains(id) {
                        return None;
                    }
                    let slot = world.entities.get(id as usize)?;
                    if slot.alive {
                        Some(Entity::new(id, slot.generation))
                    } else {
                        None
                    }
                }))
            }
            _ => Box::new(std::iter::empty()),
        }
    }
}
