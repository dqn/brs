// Skin.java -> skin.rs
// Mechanical line-by-line translation.

use std::collections::HashMap;

use crate::skin::core::custom_event::CustomEvent;
use crate::skin::core::custom_timer::CustomTimer;
use crate::skin::objects::skin_image::SkinImage;
use crate::skin::objects::skin_number::SkinNumber;
use crate::skin::property::boolean_property::BooleanProperty;
use crate::skin::property::timer_property::TimerPropertyEnum;
use crate::skin::property::timer_property_factory;
use crate::skin::reexports::{MainState, SkinConfigOffset, SkinOffset, TextureRegion};
use crate::skin::types::skin_header::SkinHeader;
use crate::skin::types::skin_node::SkinNode;
use crate::skin::types::skin_object::{DestinationParams, SkinObjectRenderer};

use log::debug;

include!("skin_impl.rs");
include!("skin_drawable.rs");

#[cfg(test)]
mod tests;
