// Translated from MusicDecideSkin.java
// Skin for the music decide screen.

use crate::skin::skin::Skin;
use crate::skin::skin_header::SkinHeader;

/// MusicDecideSkin - skin for the music decide screen
///
/// Translated from MusicDecideSkin.java
/// In Java, MusicDecideSkin extends Skin. In Rust, we use composition.
pub struct MusicDecideSkin {
    pub skin: Skin,
}

impl MusicDecideSkin {
    pub fn new(header: SkinHeader) -> Self {
        Self {
            skin: Skin::new(header),
        }
    }
}
