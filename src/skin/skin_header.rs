use super::SkinType;

/// Skin header containing metadata.
#[derive(Debug, Clone)]
pub struct SkinHeader {
    /// Skin name.
    pub name: String,
    /// Skin author.
    pub author: String,
    /// Skin type.
    pub skin_type: SkinType,
    /// Resolution width.
    pub width: u32,
    /// Resolution height.
    pub height: u32,
    /// Time when loading ends (ms).
    pub loadend: i32,
    /// Time when play starts (ms).
    pub playstart: i32,
    /// Scene duration (ms).
    pub scene: i32,
    /// Input delay (ms).
    pub input: i32,
    /// Close time (ms).
    pub close: i32,
    /// Fadeout time (ms).
    pub fadeout: i32,
}

impl Default for SkinHeader {
    fn default() -> Self {
        Self {
            name: String::new(),
            author: String::new(),
            skin_type: SkinType::Play7,
            width: 1920,
            height: 1080,
            loadend: 0,
            playstart: 0,
            scene: 0,
            input: 0,
            close: 0,
            fadeout: 0,
        }
    }
}
