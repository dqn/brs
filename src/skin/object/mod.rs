mod destination;
mod image_object;
mod skin_object;

pub use destination::{InterpolatedDest, interpolate_destinations};
pub use image_object::ImageObject;
pub use skin_object::{SkinObject, check_option_visibility, get_timer_elapsed, is_timer_active};
