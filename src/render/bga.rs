//! BGA (Background Animation) processor for displaying background images.

use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// BGA change event.
#[derive(Debug, Clone)]
pub struct BgaEvent {
    pub time_ms: f64,
    pub bga_id: u16,
    pub layer: BgaLayer,
}

/// BGA layer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgaLayer {
    /// Main BGA layer.
    Base,
    /// Layer that appears on top of base.
    Layer,
    /// Layer that appears on poor judgment.
    Poor,
}

/// Processor for BGA images during gameplay.
pub struct BgaProcessor {
    /// Loaded BGA images by ID.
    images: HashMap<u16, Texture2D>,
    /// BGA events sorted by time.
    events: Vec<BgaEvent>,
    /// Current event index.
    current_index: usize,
    /// Current base BGA ID.
    current_base_id: Option<u16>,
    /// Current layer BGA ID.
    current_layer_id: Option<u16>,
    /// Poor layer BGA ID.
    poor_layer_id: Option<u16>,
    /// Whether to show poor layer (on miss/poor judgment).
    show_poor: bool,
    /// Time when poor was triggered.
    poor_trigger_time: Option<f64>,
}

impl BgaProcessor {
    /// Duration to show poor layer (ms).
    const POOR_DURATION_MS: f64 = 200.0;

    /// Create a new BGA processor.
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            events: Vec::new(),
            current_index: 0,
            current_base_id: None,
            current_layer_id: None,
            poor_layer_id: None,
            show_poor: false,
            poor_trigger_time: None,
        }
    }

    /// Load BGA images from file paths.
    /// The bga_files map contains ID -> relative path mappings.
    pub async fn load_images(
        &mut self,
        bga_files: &HashMap<u16, String>,
        base_dir: &Path,
    ) -> usize {
        let mut loaded = 0;

        for (&id, path) in bga_files {
            let full_path = base_dir.join(path);

            // Try loading with original extension
            if let Ok(texture) = load_texture(full_path.to_string_lossy().as_ref()).await {
                self.images.insert(id, texture);
                loaded += 1;
                continue;
            }

            // Try common image extensions if original fails
            let stem = full_path.with_extension("");
            for ext in &["bmp", "png", "jpg", "jpeg"] {
                let alt_path = stem.with_extension(ext);
                if let Ok(texture) = load_texture(alt_path.to_string_lossy().as_ref()).await {
                    self.images.insert(id, texture);
                    loaded += 1;
                    break;
                }
            }
        }

        loaded
    }

    /// Set BGA events from parsed BMS data.
    pub fn set_events(&mut self, events: Vec<BgaEvent>) {
        self.events = events;
        self.events
            .sort_by(|a, b| a.time_ms.partial_cmp(&b.time_ms).unwrap());
    }

    /// Update the processor for the current time.
    pub fn update(&mut self, current_time_ms: f64) {
        // Process events up to current time
        while self.current_index < self.events.len() {
            let event = &self.events[self.current_index];
            if event.time_ms > current_time_ms {
                break;
            }

            match event.layer {
                BgaLayer::Base => self.current_base_id = Some(event.bga_id),
                BgaLayer::Layer => self.current_layer_id = Some(event.bga_id),
                BgaLayer::Poor => self.poor_layer_id = Some(event.bga_id),
            }

            self.current_index += 1;
        }

        // Check if poor layer should be hidden
        if self.show_poor {
            if let Some(trigger_time) = self.poor_trigger_time {
                if current_time_ms - trigger_time > Self::POOR_DURATION_MS {
                    self.show_poor = false;
                    self.poor_trigger_time = None;
                }
            }
        }
    }

    /// Trigger the poor layer display.
    pub fn trigger_poor(&mut self, current_time_ms: f64) {
        if self.poor_layer_id.is_some() {
            self.show_poor = true;
            self.poor_trigger_time = Some(current_time_ms);
        }
    }

    /// Draw the BGA at the specified position and size.
    pub fn draw(&self, x: f32, y: f32, width: f32, height: f32) {
        // Draw base layer
        if let Some(id) = self.current_base_id {
            if let Some(texture) = self.images.get(&id) {
                draw_texture_ex(
                    texture,
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

        // Draw layer on top
        if let Some(id) = self.current_layer_id {
            if let Some(texture) = self.images.get(&id) {
                draw_texture_ex(
                    texture,
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

        // Draw poor layer if active
        if self.show_poor {
            if let Some(id) = self.poor_layer_id {
                if let Some(texture) = self.images.get(&id) {
                    draw_texture_ex(
                        texture,
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

    /// Check if any BGA images are loaded.
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    /// Get the number of loaded images.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// Reset the processor state.
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.current_base_id = None;
        self.current_layer_id = None;
        self.show_poor = false;
        self.poor_trigger_time = None;
    }
}

impl Default for BgaProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bga_processor_new() {
        let processor = BgaProcessor::new();
        assert!(!processor.has_images());
        assert_eq!(processor.image_count(), 0);
    }

    #[test]
    fn test_bga_events_update() {
        let mut processor = BgaProcessor::new();

        let events = vec![
            BgaEvent {
                time_ms: 1000.0,
                bga_id: 1,
                layer: BgaLayer::Base,
            },
            BgaEvent {
                time_ms: 2000.0,
                bga_id: 2,
                layer: BgaLayer::Base,
            },
        ];
        processor.set_events(events);

        // Before first event
        processor.update(500.0);
        assert_eq!(processor.current_base_id, None);

        // After first event
        processor.update(1500.0);
        assert_eq!(processor.current_base_id, Some(1));

        // After second event
        processor.update(2500.0);
        assert_eq!(processor.current_base_id, Some(2));
    }

    #[test]
    fn test_poor_layer_timing() {
        let mut processor = BgaProcessor::new();
        processor.poor_layer_id = Some(99);

        // Trigger poor
        processor.trigger_poor(1000.0);
        assert!(processor.show_poor);

        // Still showing
        processor.update(1100.0);
        assert!(processor.show_poor);

        // Should be hidden after duration
        processor.update(1300.0);
        assert!(!processor.show_poor);
    }
}
