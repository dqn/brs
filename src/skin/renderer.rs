use std::path::Path;

use anyhow::Result;
use tracing::warn;

use crate::skin::{
    ImageObject, LuaSkinLoader, MainState, NumberObject, Skin, SkinObject, SkinObjectType,
    SkinSourceManager, TextObject,
};

/// Runtime skin renderer that manages loaded skin objects.
pub struct SkinRenderer {
    /// Loaded skin data.
    skin: Skin,
    /// Texture source manager.
    sources: SkinSourceManager,
    /// Prepared skin objects.
    objects: Vec<Box<dyn SkinObject>>,
}

impl SkinRenderer {
    /// Load a skin from a .luaskin file.
    pub async fn load(skin_path: &Path) -> Result<Self> {
        let loader = LuaSkinLoader::new()?;
        let skin = loader.load(skin_path, &std::collections::HashMap::new())?;

        let base_dir = skin_path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut sources = SkinSourceManager::new(base_dir);

        // Load all texture sources
        for (id, source) in &skin.sources {
            if let Err(e) = sources.load_source(*id, &source.path).await {
                warn!("Failed to load source {}: {}", id, e);
            }
        }

        // Load all fonts
        for (id, font_def) in &skin.fonts {
            if let Err(e) = sources.load_font(*id, &font_def.path).await {
                warn!("Failed to load font {}: {}", id, e);
            }
        }

        // Create skin objects
        let mut objects: Vec<Box<dyn SkinObject>> = Vec::new();
        for obj_data in &skin.objects {
            let obj: Box<dyn SkinObject> = match obj_data.object_type {
                SkinObjectType::Number => {
                    let number_def = skin.numbers.get(&obj_data.id).cloned();
                    Box::new(NumberObject::new(obj_data.clone(), number_def))
                }
                SkinObjectType::Text => {
                    let text_def = skin.texts.get(&obj_data.id).cloned();
                    Box::new(TextObject::new(obj_data.clone(), text_def))
                }
                SkinObjectType::Image | SkinObjectType::ImageSet => {
                    let image_def = skin.images.get(&obj_data.id).cloned();
                    Box::new(ImageObject::new(obj_data.clone(), image_def))
                }
                _ => {
                    let image_def = skin.images.get(&obj_data.id).cloned();
                    Box::new(ImageObject::new(obj_data.clone(), image_def))
                }
            };
            objects.push(obj);
        }

        // Prepare all objects
        for obj in &mut objects {
            obj.prepare(&sources);
        }

        Ok(Self {
            skin,
            sources,
            objects,
        })
    }

    /// Draw all skin objects.
    pub fn draw(&self, state: &MainState, now_time_us: i64) {
        for obj in &self.objects {
            obj.draw(state, &self.sources, now_time_us);
        }
    }

    /// Get the skin resolution.
    pub fn resolution(&self) -> (u32, u32) {
        (self.skin.header.width, self.skin.header.height)
    }

    /// Get the skin name.
    pub fn name(&self) -> &str {
        &self.skin.header.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_renderer_types() {
        // Just verify the types compile correctly
        fn _accepts_skin_renderer(_: SkinRenderer) {}
    }
}
