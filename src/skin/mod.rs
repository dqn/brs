// Skin system supporting LR2/JSON/Lua formats.

pub mod loader;
pub mod lua;
pub mod object;
mod renderer;
mod skin_data;
mod skin_header;
pub mod skin_property;
mod skin_type;
pub mod source;

pub use loader::LuaSkinLoader;
pub use lua::{JudgeType, LastJudge, MainState, MainStateTimers};
pub use object::{
    ImageObject, InterpolatedDest, SkinObject, check_option_visibility, get_timer_elapsed,
    interpolate_destinations, is_timer_active,
};
pub use renderer::SkinRenderer;
pub use skin_data::{
    Destination, ImageDef, ImageSetDef, Skin, SkinObjectData, SkinObjectType, SkinSource,
};
pub use skin_header::SkinHeader;
pub use skin_type::SkinType;
pub use source::{DrawParams, LoadedTexture, SkinSourceManager, draw_texture_params};
