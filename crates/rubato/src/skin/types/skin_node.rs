use std::any::Any;

use crate::skin::reexports::MainState;
use crate::skin::types::skin_object::{SkinObjectData, SkinObjectRenderer};

/// Trait for polymorphic skin object dispatch, replacing the SkinObject enum.
pub trait SkinNode: Send {
    /// Access the common object data.
    fn data(&self) -> &SkinObjectData;

    /// Mutable access to the common object data.
    fn data_mut(&mut self) -> &mut SkinObjectData;

    /// Validate the object after loading. Returns true if valid.
    fn validate(&mut self) -> bool {
        true
    }

    /// Update state for the current frame.
    fn prepare(&mut self, time: i64, state: &dyn MainState);

    /// Draw the object.
    fn draw(&mut self, sprite: &mut SkinObjectRenderer, state: &dyn MainState);

    /// Release resources.
    fn dispose(&mut self) {}

    /// Type name for debugging.
    fn type_name(&self) -> &'static str;

    /// Returns true if this object is a slider (used for mouse drag filtering).
    fn is_slider(&self) -> bool {
        false
    }

    /// Downcast support.
    fn as_any(&self) -> &dyn Any;

    /// Mutable downcast support.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Owned downcast support for consuming Box<dyn SkinNode>.
    fn into_any_box(self: Box<Self>) -> Box<dyn Any>;
}
