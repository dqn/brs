use crate::skin::destination::DestinationSet;
use crate::traits::render::TextureId;

/// Skin judge object for judge display (PG/GR/GD/BD/PR/MS animations).
#[derive(Debug, Clone, Default)]
pub struct JudgeObject {
    pub id: String,
    /// Player index (0=1P, 1=2P, 2=3P).
    pub player: i32,
    /// Texture entries per judge type (0-5: PG, GR, GD, BD, PR, MS).
    pub textures: Vec<JudgeTextureEntry>,
    /// Number texture for combo display.
    pub number_textures: Vec<JudgeTextureEntry>,
    /// Fast/slow indicator index.
    pub shift: bool,
    pub dst: DestinationSet,
    /// Separate destination for the number display.
    pub number_dst: DestinationSet,
}

/// Texture entry for a judge image.
#[derive(Debug, Clone)]
pub struct JudgeTextureEntry {
    pub src: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    pub div_x: i32,
    pub div_y: i32,
    pub cycle: i32,
    pub texture: Option<TextureId>,
}
