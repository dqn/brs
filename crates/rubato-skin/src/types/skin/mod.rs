// Skin.java -> skin.rs
// Mechanical line-by-line translation.

use std::collections::HashMap;

use crate::core::custom_event::CustomEvent;
use crate::core::custom_timer::CustomTimer;
use crate::objects::skin_image::SkinImage;
use crate::objects::skin_number::SkinNumber;
use crate::property::boolean_property::BooleanProperty;
use crate::property::timer_property::TimerPropertyEnum;
use crate::property::timer_property_factory;
use crate::reexports::{MainState, SkinConfigOffset, SkinOffset, TextureRegion};
use crate::types::skin_header::SkinHeader;
use crate::types::skin_node::SkinNode;
use crate::types::skin_object::{DestinationParams, SkinObjectRenderer};

use log::debug;

include!("skin_impl.rs");
include!("skin_drawable.rs");

#[cfg(test)]
mod tests;
