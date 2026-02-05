use anyhow::Result;

/// Texture handle for referencing loaded textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

/// Font handle for referencing loaded fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub u64);

/// Blend mode for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Alpha,
    Additive,
}

/// Color with RGBA components (0.0..=1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Source rectangle within a texture (pixel coordinates).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SrcRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Destination rectangle on screen (pixel coordinates).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DstRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Abstraction over rendering backends.
/// Implementations: WgpuRenderer (production), CommandRecorder (testing).
pub trait RenderBackend {
    fn begin_frame(&mut self) -> Result<()>;
    fn end_frame(&mut self) -> Result<()>;

    fn load_texture(&mut self, path: &std::path::Path) -> Result<TextureId>;
    fn load_texture_from_memory(&mut self, data: &[u8]) -> Result<TextureId>;
    fn texture_size(&self, id: TextureId) -> Option<(u32, u32)>;

    fn draw_sprite(
        &mut self,
        texture: TextureId,
        src: SrcRect,
        dst: DstRect,
        color: Color,
        angle: f32,
        blend: BlendMode,
    ) -> Result<()>;

    fn load_font(&mut self, path: &std::path::Path) -> Result<FontId>;
    fn draw_text(
        &mut self,
        font: FontId,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> Result<()>;

    fn screen_size(&self) -> (u32, u32);
    fn set_render_target(&mut self, texture: Option<TextureId>) -> Result<()>;
    fn clear(&mut self, color: Color) -> Result<()>;
}
