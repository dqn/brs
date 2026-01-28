//! Beatoraja skin renderer
//!
//! Handles rendering of skin elements using loaded textures.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use super::scene::play::{NoteType as SkinNoteType, PlaySkinConfig};
use super::types::{BeatorajaSkin, ImageDef};
use crate::skin::assets::{ImageRegion, TextureCache, TextureId};

/// Skin asset cache for rendering
pub struct SkinAssets {
    /// Texture cache
    pub textures: TextureCache,
    /// Mapping from source ID to texture ID
    source_to_texture: HashMap<i32, TextureId>,
    /// Image definitions (for region lookup)
    image_defs: HashMap<i32, ImageDef>,
    /// Source texture dimensions
    source_dimensions: HashMap<i32, (u32, u32)>,
}

impl SkinAssets {
    /// Create a new skin asset cache
    pub fn new(base_path: &Path) -> Self {
        Self {
            textures: TextureCache::new(base_path.to_path_buf()),
            source_to_texture: HashMap::new(),
            image_defs: HashMap::new(),
            source_dimensions: HashMap::new(),
        }
    }

    /// Load all sources from a skin
    pub async fn load_skin(&mut self, skin: &BeatorajaSkin) -> Result<()> {
        // Store image definitions
        for img_def in &skin.image {
            self.image_defs.insert(img_def.id, img_def.clone());
        }

        // Load source textures
        for source in &skin.source {
            match self.textures.load(&source.path).await {
                Ok(tex_id) => {
                    self.source_to_texture.insert(source.id, tex_id);

                    // Store dimensions
                    if let Some(entry) = self.textures.peek(tex_id) {
                        self.source_dimensions
                            .insert(source.id, (entry.width, entry.height));
                    }
                }
                Err(e) => {
                    // Log warning but continue loading
                    eprintln!("Warning: Failed to load skin source {}: {}", source.path, e);
                }
            }
        }

        Ok(())
    }

    /// Get texture for an image definition ID
    pub fn get_texture_for_image(&self, image_id: i32) -> Option<(Texture2D, ImageRegion)> {
        let img_def = self.image_defs.get(&image_id)?;
        let tex_id = self.source_to_texture.get(&img_def.src)?;
        let entry = self.textures.peek(*tex_id)?;

        // Calculate region
        let (src_w, src_h) = self
            .source_dimensions
            .get(&img_def.src)
            .copied()
            .unwrap_or((entry.width, entry.height));

        let region = if img_def.w == 0 && img_def.h == 0 {
            // Full texture
            ImageRegion::full(src_w, src_h)
        } else {
            ImageRegion::new(
                img_def.x as u32,
                img_def.y as u32,
                img_def.w as u32,
                img_def.h as u32,
            )
        };

        Some((entry.texture.clone(), region))
    }

    /// Get texture for an image definition with frame animation
    pub fn get_texture_for_image_frame(
        &self,
        image_id: i32,
        frame: u32,
    ) -> Option<(Texture2D, ImageRegion)> {
        let img_def = self.image_defs.get(&image_id)?;
        let tex_id = self.source_to_texture.get(&img_def.src)?;
        let entry = self.textures.peek(*tex_id)?;

        let (src_w, src_h) = self
            .source_dimensions
            .get(&img_def.src)
            .copied()
            .unwrap_or((entry.width, entry.height));

        let base_region = if img_def.w == 0 && img_def.h == 0 {
            ImageRegion::full(src_w, src_h)
        } else {
            ImageRegion::new(
                img_def.x as u32,
                img_def.y as u32,
                img_def.w as u32,
                img_def.h as u32,
            )
        };

        // Apply frame division
        let frame_region = base_region.get_frame(frame, img_def.divx as u32, img_def.divy as u32);

        Some((entry.texture.clone(), frame_region))
    }

    /// Check if a source is loaded
    pub fn has_source(&self, source_id: i32) -> bool {
        self.source_to_texture.contains_key(&source_id)
    }

    /// Check if an image definition exists
    pub fn has_image(&self, image_id: i32) -> bool {
        self.image_defs.contains_key(&image_id)
    }
}

/// Skin note renderer
pub struct SkinRenderer {
    /// Loaded skin assets
    assets: SkinAssets,
    /// Play skin configuration
    play_config: Option<PlaySkinConfig>,
    /// Skin scale factors
    scale_x: f32,
    scale_y: f32,
}

impl SkinRenderer {
    /// Create a new skin renderer
    pub fn new(assets: SkinAssets) -> Self {
        Self {
            assets,
            play_config: None,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Load a skin and extract play configuration
    pub async fn load_skin(base_path: &Path, skin: &BeatorajaSkin) -> Result<Self> {
        let mut assets = SkinAssets::new(base_path);
        assets.load_skin(skin).await?;

        let play_config = PlaySkinConfig::from_skin(skin);

        Ok(Self {
            assets,
            play_config,
            scale_x: 1.0,
            scale_y: 1.0,
        })
    }

    /// Set scale factors for different screen resolutions
    pub fn set_scale(&mut self, screen_w: f32, screen_h: f32, skin_w: f32, skin_h: f32) {
        self.scale_x = screen_w / skin_w;
        self.scale_y = screen_h / skin_h;
    }

    /// Get play skin configuration
    pub fn play_config(&self) -> Option<&PlaySkinConfig> {
        self.play_config.as_ref()
    }

    /// Draw a note using skin textures
    pub fn draw_note(
        &self,
        lane: usize,
        note_type: SkinNoteType,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let Some(config) = &self.play_config else {
            return;
        };

        let image_id = match super::scene::play::get_note_image_id(config, lane, note_type) {
            Some(id) => id,
            None => return,
        };

        if let Some((texture, region)) = self.assets.get_texture_for_image(image_id) {
            self.draw_texture_region(texture, &region, x, y, width, height);
        }
    }

    /// Draw a note with frame animation
    #[allow(clippy::too_many_arguments)]
    pub fn draw_note_animated(
        &self,
        lane: usize,
        note_type: SkinNoteType,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        frame: u32,
    ) {
        let Some(config) = &self.play_config else {
            return;
        };

        let image_id = match super::scene::play::get_note_image_id(config, lane, note_type) {
            Some(id) => id,
            None => return,
        };

        if let Some((texture, region)) = self.assets.get_texture_for_image_frame(image_id, frame) {
            self.draw_texture_region(texture, &region, x, y, width, height);
        }
    }

    /// Draw LN body (tiled or stretched)
    pub fn draw_ln_body(&self, lane: usize, x: f32, y: f32, width: f32, height: f32, active: bool) {
        let Some(config) = &self.play_config else {
            return;
        };

        let note_type = if active {
            SkinNoteType::LnActive
        } else {
            SkinNoteType::LnBody
        };

        let image_id = match super::scene::play::get_note_image_id(config, lane, note_type) {
            Some(id) => id,
            None => return,
        };

        if let Some((texture, region)) = self.assets.get_texture_for_image(image_id) {
            if config.ln_body_tile {
                // Tile the texture vertically
                self.draw_texture_tiled(texture, &region, x, y, width, height);
            } else {
                // Stretch to fit
                self.draw_texture_region(texture, &region, x, y, width, height);
            }
        }
    }

    /// Draw a texture region at specified position and size
    fn draw_texture_region(
        &self,
        texture: Texture2D,
        region: &ImageRegion,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let source_rect = region.to_rect();

        draw_texture_ex(
            &texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                source: Some(source_rect),
                ..Default::default()
            },
        );
    }

    /// Draw a texture region tiled vertically
    fn draw_texture_tiled(
        &self,
        texture: Texture2D,
        region: &ImageRegion,
        x: f32,
        mut y: f32,
        width: f32,
        total_height: f32,
    ) {
        let source_rect = region.to_rect();
        let tile_height = region.height as f32 * (width / region.width as f32);

        let mut remaining = total_height;
        while remaining > 0.0 {
            let draw_height = tile_height.min(remaining);
            let source_height = if draw_height < tile_height {
                region.height as f32 * (draw_height / tile_height)
            } else {
                region.height as f32
            };

            let partial_source =
                Rect::new(source_rect.x, source_rect.y, source_rect.w, source_height);

            draw_texture_ex(
                &texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(width, draw_height)),
                    source: Some(partial_source),
                    ..Default::default()
                },
            );

            y += draw_height;
            remaining -= draw_height;
        }
    }

    /// Check if skin has textures for notes
    pub fn has_note_textures(&self) -> bool {
        self.play_config.is_some()
    }

    /// Get lane count from skin config
    pub fn lane_count(&self) -> Option<usize> {
        self.play_config.as_ref().map(|c| c.lane_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_skin_assets_new() {
        let dir = tempdir().unwrap();
        let assets = SkinAssets::new(dir.path());
        assert!(assets.source_to_texture.is_empty());
        assert!(assets.image_defs.is_empty());
    }

    #[test]
    fn test_skin_renderer_scale() {
        let dir = tempdir().unwrap();
        let assets = SkinAssets::new(dir.path());
        let mut renderer = SkinRenderer::new(assets);

        renderer.set_scale(1920.0, 1080.0, 1280.0, 720.0);
        assert!((renderer.scale_x - 1.5).abs() < 0.001);
        assert!((renderer.scale_y - 1.5).abs() < 0.001);
    }
}
