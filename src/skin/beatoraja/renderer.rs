//! Beatoraja skin renderer
//!
//! Handles rendering of skin elements using loaded textures.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use super::scene::play::{NoteType as SkinNoteType, PlaySkinConfig};
use super::types::{BeatorajaSkin, ImageDef, ValueDef};
use crate::game::JudgeResult;
use crate::skin::assets::{ImageRegion, TextureCache, TextureId};

/// Skin asset cache for rendering
pub struct SkinAssets {
    /// Texture cache
    pub textures: TextureCache,
    /// Mapping from source ID to texture ID
    source_to_texture: HashMap<i32, TextureId>,
    /// Image definitions (for region lookup)
    image_defs: HashMap<i32, ImageDef>,
    /// Value definitions (for number rendering)
    value_defs: HashMap<i32, ValueDef>,
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
            value_defs: HashMap::new(),
            source_dimensions: HashMap::new(),
        }
    }

    /// Load all sources from a skin
    pub async fn load_skin(&mut self, skin: &BeatorajaSkin) -> Result<()> {
        // Store image definitions
        for img_def in &skin.image {
            self.image_defs.insert(img_def.id, img_def.clone());
        }

        // Store value definitions
        for val_def in &skin.value {
            self.value_defs.insert(val_def.id, val_def.clone());
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

    /// Get value definition by ID
    pub fn get_value_def(&self, value_id: i32) -> Option<&ValueDef> {
        self.value_defs.get(&value_id)
    }

    /// Get texture for a digit from a value definition
    /// Returns texture and region for the specified digit (0-9)
    pub fn get_digit_texture(&self, value_id: i32, digit: u8) -> Option<(Texture2D, ImageRegion)> {
        let val_def = self.value_defs.get(&value_id)?;
        let tex_id = self.source_to_texture.get(&val_def.src)?;
        let entry = self.textures.peek(*tex_id)?;

        // Calculate digit position in the sprite sheet
        let digit_width = val_def.w as u32;
        let digit_height = val_def.h as u32;
        let divx = val_def.divx.max(1) as u32;
        let divy = val_def.divy.max(1) as u32;

        let digit_idx = digit as u32 % (divx * divy);
        let col = digit_idx % divx;
        let row = digit_idx / divx;

        let region = ImageRegion::new(
            val_def.x as u32 + col * digit_width,
            val_def.y as u32 + row * digit_height,
            digit_width,
            digit_height,
        );

        Some((entry.texture.clone(), region))
    }
}

/// Judge image configuration
#[derive(Debug, Clone, Default)]
pub struct JudgeImageConfig {
    /// Image IDs for each judge result (PGREAT, GREAT, GOOD, BAD, POOR)
    pub images: Vec<i32>,
}

/// Gauge rendering configuration
#[derive(Debug, Clone, Default)]
pub struct GaugeConfig {
    /// Background image ID
    pub background_id: Option<i32>,
    /// Filled gauge image ID
    pub fill_id: Option<i32>,
    /// Gauge direction (0: right, 1: down, 2: left, 3: up)
    pub direction: i32,
}

/// Skin note renderer
pub struct SkinRenderer {
    /// Loaded skin assets
    assets: SkinAssets,
    /// Play skin configuration
    play_config: Option<PlaySkinConfig>,
    /// Judge image configuration
    judge_config: Option<JudgeImageConfig>,
    /// Gauge configuration
    gauge_config: Option<GaugeConfig>,
    /// Combo number value definition ID
    combo_value_id: Option<i32>,
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
            judge_config: None,
            gauge_config: None,
            combo_value_id: None,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }

    /// Load a skin and extract configurations
    pub async fn load_skin(base_path: &Path, skin: &BeatorajaSkin) -> Result<Self> {
        let mut assets = SkinAssets::new(base_path);
        assets.load_skin(skin).await?;

        let play_config = PlaySkinConfig::from_skin(skin);

        // Extract judge configuration
        let judge_config = skin.judge.first().map(|j| JudgeImageConfig {
            images: j.images.clone(),
        });

        // Extract gauge configuration from slider elements
        let gauge_config = skin
            .slider
            .iter()
            .find(|s| s.base.refer == 0)
            .map(|s| GaugeConfig {
                background_id: None,
                fill_id: Some(s.id),
                direction: s.direction,
            });

        // Find combo number value definition (typically refer=10 for combo)
        let combo_value_id = skin.number.iter().find(|n| n.value == 10).map(|n| n.id);

        Ok(Self {
            assets,
            play_config,
            judge_config,
            gauge_config,
            combo_value_id,
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

    /// Draw a number using skin digit textures
    ///
    /// # Arguments
    /// * `value_id` - Value definition ID to use for digit textures
    /// * `number` - The number to display
    /// * `x` - X position (right-aligned)
    /// * `y` - Y position
    /// * `digit_width` - Width of each digit
    /// * `digit_height` - Height of each digit
    /// * `max_digits` - Maximum number of digits to display
    /// * `zero_pad` - Whether to pad with leading zeros
    #[allow(clippy::too_many_arguments)]
    pub fn draw_number(
        &self,
        value_id: i32,
        number: u32,
        x: f32,
        y: f32,
        digit_width: f32,
        digit_height: f32,
        max_digits: usize,
        zero_pad: bool,
    ) {
        let digits: Vec<u8> = if number == 0 {
            vec![0]
        } else {
            let mut n = number;
            let mut d = Vec::new();
            while n > 0 {
                d.push((n % 10) as u8);
                n /= 10;
            }
            d.reverse();
            d
        };

        // Limit to max digits
        let digits = if digits.len() > max_digits {
            digits[digits.len() - max_digits..].to_vec()
        } else {
            digits
        };

        // Calculate starting position (right-aligned)
        let total_digits = if zero_pad { max_digits } else { digits.len() };
        let start_x = x - (total_digits as f32 * digit_width);

        // Draw leading zeros if padding
        let leading_zeros = if zero_pad {
            max_digits.saturating_sub(digits.len())
        } else {
            0
        };

        for i in 0..leading_zeros {
            if let Some((texture, region)) = self.assets.get_digit_texture(value_id, 0) {
                let dx = start_x + (i as f32 * digit_width);
                self.draw_texture_region(texture, &region, dx, y, digit_width, digit_height);
            }
        }

        // Draw actual digits
        for (i, &digit) in digits.iter().enumerate() {
            if let Some((texture, region)) = self.assets.get_digit_texture(value_id, digit) {
                let dx = start_x + ((leading_zeros + i) as f32 * digit_width);
                self.draw_texture_region(texture, &region, dx, y, digit_width, digit_height);
            }
        }
    }

    /// Draw combo number if skin has combo value definition
    pub fn draw_combo(&self, combo: u32, x: f32, y: f32, scale: f32) {
        let Some(value_id) = self.combo_value_id else {
            return;
        };

        let Some(val_def) = self.assets.get_value_def(value_id) else {
            return;
        };

        let digit_width = val_def.w as f32 * scale;
        let digit_height = val_def.h as f32 * scale;

        self.draw_number(value_id, combo, x, y, digit_width, digit_height, 4, false);
    }

    /// Draw judge image
    ///
    /// # Arguments
    /// * `result` - Judge result
    /// * `x` - X position (center)
    /// * `y` - Y position (center)
    /// * `scale` - Scale factor
    /// * `alpha` - Alpha value (0.0-1.0)
    pub fn draw_judge(&self, result: JudgeResult, x: f32, y: f32, scale: f32, alpha: f32) {
        let Some(config) = &self.judge_config else {
            return;
        };

        // Map JudgeResult to image index
        let idx = match result {
            JudgeResult::PGreat => 0,
            JudgeResult::Great => 1,
            JudgeResult::Good => 2,
            JudgeResult::Bad => 3,
            JudgeResult::Poor => 4,
        };

        let Some(&image_id) = config.images.get(idx) else {
            return;
        };

        if let Some((texture, region)) = self.assets.get_texture_for_image(image_id) {
            let width = region.width as f32 * scale;
            let height = region.height as f32 * scale;
            let draw_x = x - width / 2.0;
            let draw_y = y - height / 2.0;

            let color = Color::new(1.0, 1.0, 1.0, alpha);
            let source_rect = region.to_rect();

            draw_texture_ex(
                &texture,
                draw_x,
                draw_y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    source: Some(source_rect),
                    ..Default::default()
                },
            );
        }
    }

    /// Draw gauge (horizontal or vertical bar)
    ///
    /// # Arguments
    /// * `value` - Gauge value (0.0-1.0)
    /// * `x` - X position
    /// * `y` - Y position
    /// * `width` - Total width
    /// * `height` - Total height
    pub fn draw_gauge(&self, value: f32, x: f32, y: f32, width: f32, height: f32) {
        let Some(config) = &self.gauge_config else {
            return;
        };

        let Some(fill_id) = config.fill_id else {
            return;
        };

        let Some((texture, region)) = self.assets.get_texture_for_image(fill_id) else {
            return;
        };

        let value = value.clamp(0.0, 1.0);

        // Calculate filled portion based on direction
        let (draw_x, draw_y, draw_w, draw_h, src_w, src_h) = match config.direction {
            0 => {
                // Right (horizontal, fill from left)
                let w = width * value;
                let sw = region.width as f32 * value;
                (x, y, w, height, sw, region.height as f32)
            }
            1 => {
                // Down (vertical, fill from top)
                let h = height * value;
                let sh = region.height as f32 * value;
                (x, y, width, h, region.width as f32, sh)
            }
            2 => {
                // Left (horizontal, fill from right)
                let w = width * value;
                let sw = region.width as f32 * value;
                (x + width - w, y, w, height, sw, region.height as f32)
            }
            3 => {
                // Up (vertical, fill from bottom)
                let h = height * value;
                let sh = region.height as f32 * value;
                (x, y + height - h, width, h, region.width as f32, sh)
            }
            _ => (
                x,
                y,
                width * value,
                height,
                region.width as f32 * value,
                region.height as f32,
            ),
        };

        if draw_w <= 0.0 || draw_h <= 0.0 {
            return;
        }

        let source_rect = match config.direction {
            2 => Rect::new(
                region.x as f32 + region.width as f32 - src_w,
                region.y as f32,
                src_w,
                src_h,
            ),
            3 => Rect::new(
                region.x as f32,
                region.y as f32 + region.height as f32 - src_h,
                src_w,
                src_h,
            ),
            _ => Rect::new(region.x as f32, region.y as f32, src_w, src_h),
        };

        draw_texture_ex(
            &texture,
            draw_x,
            draw_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                source: Some(source_rect),
                ..Default::default()
            },
        );
    }

    /// Draw an image element by ID
    pub fn draw_image(&self, image_id: i32, x: f32, y: f32, width: f32, height: f32) {
        if let Some((texture, region)) = self.assets.get_texture_for_image(image_id) {
            self.draw_texture_region(texture, &region, x, y, width, height);
        }
    }

    /// Draw an image element with alpha
    pub fn draw_image_with_alpha(
        &self,
        image_id: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        alpha: f32,
    ) {
        if let Some((texture, region)) = self.assets.get_texture_for_image(image_id) {
            let source_rect = region.to_rect();
            let color = Color::new(1.0, 1.0, 1.0, alpha);

            draw_texture_ex(
                &texture,
                x,
                y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(width, height)),
                    source: Some(source_rect),
                    ..Default::default()
                },
            );
        }
    }

    /// Check if skin has textures for notes
    pub fn has_note_textures(&self) -> bool {
        self.play_config.is_some()
    }

    /// Check if skin has judge images
    pub fn has_judge_images(&self) -> bool {
        self.judge_config
            .as_ref()
            .is_some_and(|c| !c.images.is_empty())
    }

    /// Check if skin has combo number definition
    pub fn has_combo_numbers(&self) -> bool {
        self.combo_value_id.is_some()
    }

    /// Check if skin has gauge configuration
    pub fn has_gauge(&self) -> bool {
        self.gauge_config.is_some()
    }

    /// Get lane count from skin config
    pub fn lane_count(&self) -> Option<usize> {
        self.play_config.as_ref().map(|c| c.lane_count)
    }

    /// Get skin assets reference
    pub fn assets(&self) -> &SkinAssets {
        &self.assets
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
