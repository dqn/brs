mod destination;
mod image_object;
mod number_object;
mod skin_object;
mod text_object;

pub use destination::{InterpolatedDest, interpolate_destinations};
pub use image_object::ImageObject;
pub use number_object::NumberObject;
pub use skin_object::{SkinObject, check_option_visibility, get_timer_elapsed, is_timer_active};
pub use text_object::TextObject;
