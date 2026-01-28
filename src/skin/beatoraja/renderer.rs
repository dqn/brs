//! Beatoraja skin renderer
//!
//! Handles rendering of skin elements using loaded textures.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use macroquad::prelude::*;

use super::conditions::{
    GameState as SkinGameState, GaugeType, JudgeType, TimingType, evaluate_conditions,
};
use super::properties::PropertyManager;
use super::scene::play::{NoteType as SkinNoteType, PlaySkinConfig};
use super::timers::{JudgeTimerType, TimerManager, calculate_frame};
use super::types::{BeatorajaSkin, CustomProperty, Destination, ImageDef, ImageElement, ValueDef};
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
    /// Game state for condition evaluation
    game_state: SkinGameState,
    /// Image elements from skin (for conditional rendering)
    image_elements: Vec<ImageElement>,
    /// Timer manager for animations
    timer_manager: TimerManager,
    /// Property manager for skin customization
    property_manager: PropertyManager,
    /// Property definitions from skin
    property_definitions: Vec<CustomProperty>,
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
            game_state: SkinGameState::default(),
            image_elements: Vec::new(),
            timer_manager: TimerManager::new(),
            property_manager: PropertyManager::new(),
            property_definitions: Vec::new(),
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

        // Store image elements for conditional rendering
        let image_elements = skin.images.clone();

        // Extract property definitions and initialize property manager
        let property_definitions = skin.property.clone();
        let property_manager = PropertyManager::from_definitions(&property_definitions);

        Ok(Self {
            assets,
            play_config,
            judge_config,
            gauge_config,
            combo_value_id,
            scale_x: 1.0,
            scale_y: 1.0,
            game_state: SkinGameState::default(),
            image_elements,
            timer_manager: TimerManager::new(),
            property_manager,
            property_definitions,
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

    /// Get mutable reference to game state for updates
    pub fn game_state_mut(&mut self) -> &mut SkinGameState {
        &mut self.game_state
    }

    /// Get immutable reference to game state
    pub fn game_state(&self) -> &SkinGameState {
        &self.game_state
    }

    /// Update game state from brs game state
    pub fn update_game_state(
        &mut self,
        gauge_value: f32,
        gauge_type: crate::game::GaugeType,
        last_judge: Option<JudgeResult>,
        timing_diff_ms: Option<f64>,
        keys_pressed: &[bool],
        fullcombo_ongoing: bool,
    ) {
        self.game_state.gauge_value = gauge_value;
        self.game_state.gauge_type = match gauge_type {
            crate::game::GaugeType::AssistEasy => GaugeType::AssistEasy,
            crate::game::GaugeType::Easy => GaugeType::Easy,
            crate::game::GaugeType::Normal => GaugeType::Groove,
            crate::game::GaugeType::Hard => GaugeType::Hard,
            crate::game::GaugeType::ExHard | crate::game::GaugeType::Hazard => GaugeType::ExHard,
        };

        self.game_state.last_judge = last_judge.map(|j| match j {
            JudgeResult::PGreat => JudgeType::PGreat,
            JudgeResult::Great => JudgeType::Great,
            JudgeResult::Good => JudgeType::Good,
            JudgeResult::Bad => JudgeType::Bad,
            JudgeResult::Poor => JudgeType::Poor,
        });

        self.game_state.last_timing = timing_diff_ms.map(|diff| {
            if diff.abs() < 5.0 {
                TimingType::Just
            } else if diff > 0.0 {
                TimingType::Fast
            } else {
                TimingType::Slow
            }
        });

        for (i, &pressed) in keys_pressed.iter().enumerate() {
            if i < 8 {
                self.game_state.keys_pressed[i] = pressed;
            }
        }

        self.game_state.fullcombo_ongoing = fullcombo_ongoing;
    }

    /// Set key pressed state
    pub fn set_key_pressed(&mut self, lane: usize, pressed: bool) {
        if lane < 8 {
            self.game_state.keys_pressed[lane] = pressed;
        }
    }

    /// Set LN active state
    pub fn set_ln_active(&mut self, lane: usize, active: bool) {
        if lane < 8 {
            self.game_state.ln_active[lane] = active;
        }
    }

    /// Set options state
    pub fn set_options(
        &mut self,
        random: bool,
        mirror: bool,
        sudden: bool,
        hidden: bool,
        lift: bool,
    ) {
        self.game_state.random_enabled = random;
        self.game_state.mirror_enabled = mirror;
        self.game_state.sudden_enabled = sudden;
        self.game_state.hidden_enabled = hidden;
        self.game_state.lift_enabled = lift;
    }

    /// Check if an element's conditions are met
    pub fn check_conditions(&self, operations: &[i32]) -> bool {
        evaluate_conditions(operations, &self.game_state)
    }

    /// Draw an image element with condition checking
    pub fn draw_image_element(&self, element: &ImageElement, elapsed_ms: i32) {
        // Check conditions
        if !self.check_conditions(&element.base.operations) {
            return;
        }

        // Get destination for current time
        let Some(dst) = self.get_current_destination(element, elapsed_ms) else {
            return;
        };

        // Apply scale
        let x = dst.x as f32 * self.scale_x;
        let y = dst.y as f32 * self.scale_y;
        let w = dst.w as f32 * self.scale_x;
        let h = dst.h as f32 * self.scale_y;
        let alpha = dst.a as f32 / 255.0;

        if alpha <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }

        self.draw_image_with_alpha(element.id, x, y, w, h, alpha);
    }

    /// Get current destination from keyframes
    fn get_current_destination(
        &self,
        element: &ImageElement,
        elapsed_ms: i32,
    ) -> Option<Destination> {
        if element.dst.is_empty() {
            return None;
        }

        // Single destination
        if element.dst.len() == 1 {
            return Some(element.dst[0].clone());
        }

        // Find keyframes to interpolate
        let mut prev = &element.dst[0];
        let mut next = &element.dst[0];

        for i in 0..element.dst.len() {
            if element.dst[i].time <= elapsed_ms {
                prev = &element.dst[i];
                next = element.dst.get(i + 1).unwrap_or(prev);
            } else {
                break;
            }
        }

        // Past all keyframes
        if elapsed_ms >= element.dst.last().map(|d| d.time).unwrap_or(0) {
            return element.dst.last().cloned();
        }

        // Interpolate
        let time_range = next.time - prev.time;
        let t = if time_range > 0 {
            (elapsed_ms - prev.time) as f32 / time_range as f32
        } else {
            0.0
        };

        Some(interpolate_destinations(prev, next, t))
    }

    /// Draw all conditional image elements
    pub fn draw_all_images(&self, elapsed_ms: i32) {
        for element in &self.image_elements {
            self.draw_image_element(element, elapsed_ms);
        }
    }

    // ==========================================================================
    // Timer management (Phase 6)
    // ==========================================================================

    /// Initialize timer manager with scene start time
    pub fn init_timers(&mut self, scene_start: f64) {
        self.timer_manager.init(scene_start);
    }

    /// Get timer manager reference
    pub fn timer_manager(&self) -> &TimerManager {
        &self.timer_manager
    }

    /// Get mutable timer manager reference
    pub fn timer_manager_mut(&mut self) -> &mut TimerManager {
        &mut self.timer_manager
    }

    /// Trigger play start event
    pub fn on_play_start(&mut self, time: f64) {
        self.timer_manager.on_play_start(time);
    }

    /// Trigger music start event
    pub fn on_music_start(&mut self, time: f64) {
        self.timer_manager.on_music_start(time);
    }

    /// Trigger play end event
    pub fn on_play_end(&mut self, time: f64) {
        self.timer_manager.on_play_end(time);
    }

    /// Trigger failed event
    pub fn on_failed(&mut self, time: f64) {
        self.timer_manager.on_failed(time);
    }

    /// Trigger judge event for 1P
    pub fn on_judge(&mut self, result: JudgeResult, time: f64) {
        let timer_type = match result {
            JudgeResult::PGreat => JudgeTimerType::PGreat,
            JudgeResult::Great => JudgeTimerType::Great,
            JudgeResult::Good => JudgeTimerType::Good,
            JudgeResult::Bad => JudgeTimerType::Bad,
            JudgeResult::Poor => JudgeTimerType::Poor,
        };
        self.timer_manager.on_judge_1p(timer_type, time);
    }

    /// Trigger key press event
    pub fn on_key_press(&mut self, lane: usize, time: f64) {
        self.timer_manager.on_key_press_1p(lane, time);
    }

    /// Trigger key release event
    pub fn on_key_release(&mut self, lane: usize, time: f64) {
        self.timer_manager.on_key_release_1p(lane, time);
    }

    /// Trigger combo milestone event
    pub fn on_combo_milestone(&mut self, combo: u32, time: f64) {
        self.timer_manager.on_combo_milestone(combo, time);
    }

    // ==========================================================================
    // Animation support (Phase 6)
    // ==========================================================================

    /// Draw an image element with timer-based animation
    pub fn draw_animated_element(&self, element: &ImageElement, current_time: f64) {
        // Check conditions
        if !self.check_conditions(&element.base.operations) {
            return;
        }

        // Get timer elapsed time
        let timer_id = element.base.timer;
        let elapsed_ms = match self.timer_manager.get_elapsed_ms(timer_id, current_time) {
            Some(ms) => ms,
            None => return, // Timer not started, don't draw
        };

        // Get current destination with interpolation
        let Some(dst) = self.get_animated_destination(element, elapsed_ms) else {
            return;
        };

        // Apply scale
        let x = dst.x as f32 * self.scale_x;
        let y = dst.y as f32 * self.scale_y;
        let w = dst.w as f32 * self.scale_x;
        let h = dst.h as f32 * self.scale_y;
        let alpha = dst.a as f32 / 255.0;

        if alpha <= 0.0 || w <= 0.0 || h <= 0.0 {
            return;
        }

        // Get animation frame if element has frame animation
        let img_def = self.assets.image_defs.get(&element.id);
        let frame = if let Some(def) = img_def {
            let total_frames = def.divx.max(1) * def.divy.max(1);
            if total_frames > 1 {
                let cycle = element.cycle;
                let loop_type = element.loop_type;
                calculate_frame(elapsed_ms, cycle, total_frames, loop_type).unwrap_or(0) as u32
            } else {
                0
            }
        } else {
            0
        };

        // Draw with frame if animated
        if frame > 0 {
            self.draw_image_frame_with_alpha(element.id, frame, x, y, w, h, alpha);
        } else {
            self.draw_image_with_alpha(element.id, x, y, w, h, alpha);
        }
    }

    /// Draw an image element frame with alpha
    #[allow(clippy::too_many_arguments)]
    fn draw_image_frame_with_alpha(
        &self,
        image_id: i32,
        frame: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        alpha: f32,
    ) {
        if let Some((texture, region)) = self.assets.get_texture_for_image_frame(image_id, frame) {
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

    /// Get animated destination with keyframe interpolation and easing
    fn get_animated_destination(
        &self,
        element: &ImageElement,
        elapsed_ms: i32,
    ) -> Option<Destination> {
        if element.dst.is_empty() {
            return None;
        }

        // Single destination
        if element.dst.len() == 1 {
            return Some(element.dst[0].clone());
        }

        // Find keyframes to interpolate
        let mut prev_idx = 0;
        for (i, dst) in element.dst.iter().enumerate() {
            if dst.time <= elapsed_ms {
                prev_idx = i;
            } else {
                break;
            }
        }

        let prev = &element.dst[prev_idx];
        let next = element.dst.get(prev_idx + 1);

        // Past all keyframes
        if next.is_none() || elapsed_ms >= element.dst.last().map(|d| d.time).unwrap_or(0) {
            return element.dst.last().cloned();
        }

        let next = next.unwrap();

        // Interpolate with easing
        let time_range = next.time - prev.time;
        if time_range <= 0 {
            return Some(prev.clone());
        }

        let t = (elapsed_ms - prev.time) as f32 / time_range as f32;
        let t = apply_easing(t.clamp(0.0, 1.0), prev.acc);

        Some(interpolate_destinations(prev, next, t))
    }

    /// Draw all image elements with timer-based animation
    pub fn draw_all_animated(&self, current_time: f64) {
        for element in &self.image_elements {
            self.draw_animated_element(element, current_time);
        }
    }

    /// Draw judge image with animation (fade out effect)
    pub fn draw_judge_animated(
        &self,
        result: JudgeResult,
        x: f32,
        y: f32,
        scale: f32,
        current_time: f64,
        fade_duration_ms: i32,
    ) {
        let Some(config) = &self.judge_config else {
            return;
        };

        // Get timer for judge result
        let timer_type = match result {
            JudgeResult::PGreat => JudgeTimerType::PGreat,
            JudgeResult::Great => JudgeTimerType::Great,
            JudgeResult::Good => JudgeTimerType::Good,
            JudgeResult::Bad => JudgeTimerType::Bad,
            JudgeResult::Poor => JudgeTimerType::Poor,
        };

        let timer_id = match timer_type {
            JudgeTimerType::PGreat => super::conditions::timers::JUDGE_1P_PGREAT,
            JudgeTimerType::Great => super::conditions::timers::JUDGE_1P_GREAT,
            JudgeTimerType::Good => super::conditions::timers::JUDGE_1P_GOOD,
            JudgeTimerType::Bad => super::conditions::timers::JUDGE_1P_BAD,
            JudgeTimerType::Poor => super::conditions::timers::JUDGE_1P_POOR,
            JudgeTimerType::Miss => super::conditions::timers::JUDGE_1P_MISS,
        };

        let elapsed_ms = match self.timer_manager.get_elapsed_ms(timer_id, current_time) {
            Some(ms) => ms,
            None => return,
        };

        // Calculate fade alpha
        let alpha = if fade_duration_ms > 0 && elapsed_ms > fade_duration_ms / 2 {
            let fade_progress =
                (elapsed_ms - fade_duration_ms / 2) as f32 / (fade_duration_ms / 2) as f32;
            (1.0 - fade_progress).max(0.0)
        } else {
            1.0
        };

        if alpha <= 0.0 {
            return;
        }

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

    // ==========================================================================
    // Property management (Phase 7)
    // ==========================================================================

    /// Get property definitions from skin
    pub fn property_definitions(&self) -> &[CustomProperty] {
        &self.property_definitions
    }

    /// Get property manager reference
    pub fn property_manager(&self) -> &PropertyManager {
        &self.property_manager
    }

    /// Get mutable property manager reference
    pub fn property_manager_mut(&mut self) -> &mut PropertyManager {
        &mut self.property_manager
    }

    /// Set property value
    pub fn set_property(&mut self, operation: i32, value: i32) {
        self.property_manager.set_value(operation, value);
        // Update game state custom options
        self.property_manager
            .apply_to_conditions(&mut self.game_state.custom_options);
    }

    /// Get property value
    pub fn get_property(&self, operation: i32) -> i32 {
        self.property_manager.get_value(operation)
    }

    /// Load property values from a PropertyManager (e.g., from settings)
    pub fn load_properties(&mut self, properties: &PropertyManager) {
        for (&op, &value) in properties.all_values() {
            self.property_manager.set_value(op, value);
        }
        // Update game state custom options
        self.property_manager
            .apply_to_conditions(&mut self.game_state.custom_options);
    }

    /// Check if skin has customizable properties
    pub fn has_properties(&self) -> bool {
        !self.property_definitions.is_empty()
    }

    /// Get number of customizable properties
    pub fn property_count(&self) -> usize {
        self.property_definitions.len()
    }
}

/// Interpolate between two destinations
fn interpolate_destinations(from: &Destination, to: &Destination, t: f32) -> Destination {
    let lerp = |a: i32, b: i32| -> i32 { (a as f32 + (b - a) as f32 * t) as i32 };

    Destination {
        time: lerp(from.time, to.time),
        x: lerp(from.x, to.x),
        y: lerp(from.y, to.y),
        w: lerp(from.w, to.w),
        h: lerp(from.h, to.h),
        a: lerp(from.a, to.a),
        r: lerp(from.r, to.r),
        g: lerp(from.g, to.g),
        b: lerp(from.b, to.b),
        angle: lerp(from.angle, to.angle),
        acc: to.acc,
    }
}

/// Apply easing curve to progress (0.0 - 1.0)
fn apply_easing(t: f32, acc: i32) -> f32 {
    match acc {
        0 => t,                           // Linear
        1 => t * t,                       // Ease in (quadratic)
        2 => 1.0 - (1.0 - t) * (1.0 - t), // Ease out (quadratic)
        3 => {
            // Ease in-out (quadratic)
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        4 => t * t * t,               // Ease in (cubic)
        5 => 1.0 - (1.0 - t).powi(3), // Ease out (cubic)
        _ => t,                       // Default to linear
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
