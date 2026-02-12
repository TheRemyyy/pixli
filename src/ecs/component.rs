//! Component trait.

use std::any::Any;

/// Marker trait for components.
pub trait Component: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Blanket implementation for all types that meet the requirements.
impl<T: Any + Send + Sync + 'static> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
