use std::collections::HashMap;
use std::path::Path;

use macroquad::prelude::*;

use super::video::VideoDecoder;
use crate::bms::{BgaEvent, BgaLayer};

/// BGA (Background Animation) manager with video support
pub struct BgaManager {
    // Static image textures
    textures: HashMap<u32, Texture2D>,
    // Video decoders
    videos: HashMap<u32, VideoDecoder>,
    // Video textures (updated each frame)
    video_textures: HashMap<u32, Texture2D>,

    current_base: Option<u32>,
    current_poor: Option<u32>,
    current_overlay: Option<u32>,
    show_poor: bool,
    event_index: usize,
}

impl BgaManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            videos: HashMap::new(),
            video_textures: HashMap::new(),
            current_base: None,
            current_poor: None,
            current_overlay: None,
            show_poor: false,
            event_index: 0,
        }
    }

    /// Load BGA media (images and videos) from disk
    /// Returns (images_loaded, videos_loaded)
    pub fn load_media(
        &mut self,
        base_path: &Path,
        bmp_files: &HashMap<u32, String>,
    ) -> (usize, usize) {
        let mut images_loaded = 0;
        let mut videos_loaded = 0;

        for (&id, filename) in bmp_files {
            let path = base_path.join(filename);
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                // Static images
                "png" | "jpg" | "jpeg" | "bmp" => {
                    if let Some(tex) = load_texture_sync(&path) {
                        self.textures.insert(id, tex);
                        images_loaded += 1;
                    }
                }
                // Video files
                "mpg" | "mpeg" | "avi" | "wmv" | "mp4" | "webm" | "m4v" => {
                    if let Ok(decoder) = VideoDecoder::open(&path) {
                        // Create empty texture for video frames
                        let tex = Texture2D::from_rgba8(
                            decoder.width() as u16,
                            decoder.height() as u16,
                            &vec![0u8; (decoder.width() * decoder.height() * 4) as usize],
                        );
                        tex.set_filter(FilterMode::Linear);
                        self.video_textures.insert(id, tex);
                        self.videos.insert(id, decoder);
                        videos_loaded += 1;
                    }
                }
                // Unknown extension - try as image first, then video
                _ => {
                    // Try as image
                    if let Some(tex) = load_texture_sync(&path) {
                        self.textures.insert(id, tex);
                        images_loaded += 1;
                    } else if let Ok(decoder) = VideoDecoder::open(&path) {
                        let tex = Texture2D::from_rgba8(
                            decoder.width() as u16,
                            decoder.height() as u16,
                            &vec![0u8; (decoder.width() * decoder.height() * 4) as usize],
                        );
                        tex.set_filter(FilterMode::Linear);
                        self.video_textures.insert(id, tex);
                        self.videos.insert(id, decoder);
                        videos_loaded += 1;
                    } else {
                        // Try fallback extensions for images
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let parent = path.parent().unwrap_or(base_path);

                        for fallback_ext in ["png", "jpg", "jpeg", "bmp"] {
                            let alt_path = parent.join(format!("{}.{}", stem, fallback_ext));
                            if let Some(tex) = load_texture_sync(&alt_path) {
                                self.textures.insert(id, tex);
                                images_loaded += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }

        (images_loaded, videos_loaded)
    }

    /// Load BGA textures from disk (async version, images only)
    #[allow(dead_code)]
    pub async fn load_textures(
        &mut self,
        base_path: &Path,
        bmp_files: &HashMap<u32, String>,
    ) -> usize {
        let mut loaded = 0;

        for (&id, filename) in bmp_files {
            let path = base_path.join(filename);

            // Try loading with original extension
            if let Ok(texture) = load_texture(path.to_str().unwrap_or("")).await {
                texture.set_filter(FilterMode::Linear);
                self.textures.insert(id, texture);
                loaded += 1;
                continue;
            }

            // Try common image extensions as fallback
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let parent = path.parent().unwrap_or(base_path);

            for ext in ["png", "jpg", "jpeg", "bmp"] {
                let alt_path = parent.join(format!("{}.{}", stem, ext));
                if let Ok(texture) = load_texture(alt_path.to_str().unwrap_or("")).await {
                    texture.set_filter(FilterMode::Linear);
                    self.textures.insert(id, texture);
                    loaded += 1;
                    break;
                }
            }
        }

        loaded
    }

    /// Update BGA state based on current time
    pub fn update(&mut self, current_time_ms: f64, bga_events: &[BgaEvent], is_poor: bool) {
        self.show_poor = is_poor;

        // Process BGA events up to current time
        while self.event_index < bga_events.len() {
            let event = &bga_events[self.event_index];
            if event.time_ms > current_time_ms {
                break;
            }

            match event.layer {
                BgaLayer::Base => self.current_base = Some(event.bga_id),
                BgaLayer::Poor => self.current_poor = Some(event.bga_id),
                BgaLayer::Overlay => self.current_overlay = Some(event.bga_id),
            }

            self.event_index += 1;
        }

        // Update video frames for active BGA IDs
        self.update_video_frames(current_time_ms);
    }

    /// Update video frame textures
    fn update_video_frames(&mut self, current_time_ms: f64) {
        // Collect active video IDs
        let active_ids: Vec<u32> = [self.current_base, self.current_poor, self.current_overlay]
            .into_iter()
            .flatten()
            .filter(|id| self.videos.contains_key(id))
            .collect();

        for id in active_ids {
            if let Some(decoder) = self.videos.get_mut(&id) {
                let width = decoder.width() as u16;
                let height = decoder.height() as u16;
                if let Some(frame_data) = decoder.decode_frame_at(current_time_ms) {
                    if let Some(texture) = self.video_textures.get(&id) {
                        texture.update(&Image {
                            bytes: frame_data.to_vec(),
                            width,
                            height,
                        });
                    }
                }
            }
        }
    }

    /// Reset BGA state
    pub fn reset(&mut self) {
        self.current_base = None;
        self.current_poor = None;
        self.current_overlay = None;
        self.show_poor = false;
        self.event_index = 0;

        // Reset video decoders
        for decoder in self.videos.values_mut() {
            let _ = decoder.reset();
        }
    }

    /// Draw BGA
    pub fn draw(&self, x: f32, y: f32, width: f32, height: f32) {
        // Draw base or poor layer
        let base_id = if self.show_poor {
            self.current_poor.or(self.current_base)
        } else {
            self.current_base
        };

        if let Some(id) = base_id {
            // Prefer video texture, fall back to static texture
            let texture = self
                .video_textures
                .get(&id)
                .or_else(|| self.textures.get(&id));

            if let Some(tex) = texture {
                draw_texture_ex(
                    tex,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(width, height)),
                        ..Default::default()
                    },
                );
            }
        }

        // Draw overlay layer on top (if not showing poor)
        if !self.show_poor {
            if let Some(id) = self.current_overlay {
                let texture = self
                    .video_textures
                    .get(&id)
                    .or_else(|| self.textures.get(&id));

                if let Some(tex) = texture {
                    draw_texture_ex(
                        tex,
                        x,
                        y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(width, height)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Check if any media (textures or videos) are loaded
    pub fn has_textures(&self) -> bool {
        !self.textures.is_empty() || !self.videos.is_empty()
    }

    /// Check if any videos are loaded
    #[allow(dead_code)]
    pub fn has_videos(&self) -> bool {
        !self.videos.is_empty()
    }
}

impl Default for BgaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Load texture synchronously using the image crate
fn load_texture_sync(path: &Path) -> Option<Texture2D> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let texture = Texture2D::from_rgba8(rgba.width() as u16, rgba.height() as u16, rgba.as_raw());
    texture.set_filter(FilterMode::Linear);
    Some(texture)
}
