// Rendering types — re-exported from beatoraja-render (Phase 13c-2).
// All types previously defined here are now backed by real wgpu implementations.

// Re-export all public types from beatoraja-render
pub use crate::render::blend::BlendMode;
pub use crate::render::blend::gl11;
pub use crate::render::blend::gl20;
pub use crate::render::color::{Color, Matrix4, Rectangle};
pub use crate::render::font::{
    BitmapFont, BitmapFontData, FreeTypeFontGenerator, FreeTypeFontParameter, GlyphLayout,
};
pub use crate::render::pixmap::{BlitRect, Pixmap, PixmapFormat};
pub use crate::render::shader::ShaderProgram;
pub use crate::render::sprite_batch::SpriteBatch;
pub use crate::render::texture::{Texture, TextureFilter, TextureRegion};
pub use crate::render::{FileHandle, Gdx};
