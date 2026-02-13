//! Entity: unique identifier for game objects.

/// Entity identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32,
}

impl Entity {
    pub(crate) fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    /// Get the entity unique ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the entity generation (for validity checking).
    pub fn generation(&self) -> u32 {
        self.generation
    }
}
