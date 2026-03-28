mod mapping;
mod property_types;
#[cfg(test)]
mod tests;

use super::boolean_property::BooleanProperty;
use crate::skin::skin_property::*;

use mapping::{get_boolean_property0, get_boolean_type_property};
use property_types::{NegatedBooleanProperty, StaticWithoutMusicSelectProperty};

const ID_LENGTH: usize = 65536;

/// Factory for creating BooleanProperty instances from option IDs.
pub struct BooleanPropertyFactory;

impl BooleanPropertyFactory {
    /// Returns a BooleanProperty for the given option ID.
    /// Negative IDs produce a negated property.
    pub fn boolean_property(optionid: i32) -> Option<Box<dyn BooleanProperty>> {
        boolean_property(optionid)
    }
}

/// Returns a BooleanProperty for the given option ID.
/// Negative IDs produce a negated property.
pub fn boolean_property(optionid: i32) -> Option<Box<dyn BooleanProperty>> {
    let id = optionid.unsigned_abs() as usize;
    if id >= ID_LENGTH {
        return None;
    }

    // Due to the complexity of caching with trait objects in Rust,
    // we create properties on each call. The Java version uses static caches,
    // but the property creation is cheap enough.
    let result = get_boolean_property_by_id(id as i32);

    match result {
        Some(prop) => {
            if optionid < 0 {
                // Negate the property
                Some(Box::new(NegatedBooleanProperty { inner: prop }))
            } else {
                Some(prop)
            }
        }
        None => None,
    }
}

fn get_boolean_property_by_id(id: i32) -> Option<Box<dyn BooleanProperty>> {
    // Check BooleanType enum first (known IDs with proper staticness)
    if let Some(prop) = get_boolean_type_property(id) {
        return Some(prop);
    }

    // Course stage properties (OPTION_COURSE_STAGE1 .. OPTION_COURSE_STAGE4, OPTION_COURSE_STAGE_FINAL)
    if (OPTION_COURSE_STAGE1..=OPTION_COURSE_STAGE4).contains(&id) {
        return Some(Box::new(StaticWithoutMusicSelectProperty { id }));
    }
    if id == OPTION_COURSE_STAGE_FINAL {
        return Some(Box::new(StaticWithoutMusicSelectProperty { id }));
    }

    // Fallback to getBooleanProperty0
    if let Some(prop) = get_boolean_property0(id) {
        return Some(prop);
    }

    None
}
