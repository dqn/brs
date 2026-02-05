// Skin system supporting LR2/JSON/Lua formats.

pub mod font;
pub mod loader;
pub mod lua;
pub mod object;
pub mod path;
mod renderer;
mod skin_data;
mod skin_header;
pub mod skin_property;
mod skin_type;
pub mod source;

pub use font::{FontInfo, GlyphInfo};
pub use loader::{Lr2SkinLoader, LuaSkinLoader};
pub use lua::{JudgeType, LastJudge, MainState, MainStateTimers};
pub use object::{
    ImageObject, ImageSetObject, InterpolatedDest, NumberObject, SkinObject, SliderObject,
    TextObject, check_option_visibility, get_timer_elapsed, interpolate_destinations,
    is_timer_active,
};
pub use renderer::SkinRenderer;
pub use skin_data::{
    Destination, FontDef, ImageDef, ImageSetDef, NumberDef, Skin, SkinObjectData, SkinObjectType,
    SkinOffset, SkinSource, SliderDef, TextDef,
};
pub use skin_header::SkinHeader;
pub use skin_type::SkinType;
pub use source::{DrawParams, LoadedTexture, SkinSourceManager, draw_texture_params};
