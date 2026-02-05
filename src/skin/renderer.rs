use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Once;

use anyhow::Result;
use tracing::{info, warn};

use crate::skin::object::{
    InterpolatedDest, apply_offsets, check_option_visibility, get_timer_elapsed,
    interpolate_destinations,
};
use crate::skin::{
    ImageObject, ImageSetObject, Lr2SkinLoader, LuaSkinLoader, MainState, NumberObject, Skin,
    SkinHeader, SkinObject, SkinObjectType, SkinOffset, SkinSourceManager, SliderObject,
    TextObject,
};

/// Runtime skin renderer that manages loaded skin objects.
pub struct SkinRenderer {
    /// Loaded skin data.
    skin: Skin,
    /// Texture source manager.
    sources: SkinSourceManager,
    /// Prepared skin objects.
    objects: Vec<Box<dyn SkinObject>>,
    /// Enabled skin option IDs.
    /// 有効なスキンオプションID。
    skin_options: HashSet<i32>,
    /// Skin offset values.
    /// スキンのオフセット値。
    skin_offsets: HashMap<i32, SkinOffset>,
}

impl SkinRenderer {
    /// Load a skin file (Lua or LR2).
    /// Lua/LR2 のスキンファイルを読み込む。
    pub async fn load(skin_path: &Path) -> Result<Self> {
        Self::load_with_options(skin_path, &HashMap::new()).await
    }

    /// Load a skin file with options.
    /// オプション付きでスキンファイルを読み込む。
    pub async fn load_with_options(
        skin_path: &Path,
        options: &HashMap<String, i32>,
    ) -> Result<Self> {
        let ext = skin_path
            .extension()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut skin_options = HashSet::new();
        let skin = if ext == "lr2skin" || ext == "csv" {
            let loader = Lr2SkinLoader::new();
            // Convert String keys to i32 for LR2 loader
            let lr2_options: HashMap<i32, i32> = options
                .iter()
                .filter_map(|(k, v)| k.parse::<i32>().ok().map(|key| (key, *v)))
                .collect();
            skin_options.extend(options.values().copied());
            loader.load(skin_path, &lr2_options)?
        } else {
            let loader = LuaSkinLoader::new()?;
            let mut resolved_options = options.clone();
            if let Ok(defaults) = LuaSkinLoader::default_options_from_file(skin_path) {
                for (key, value) in defaults {
                    resolved_options.entry(key).or_insert(value);
                }
            }
            skin_options.extend(resolved_options.values().copied());
            loader.load(skin_path, &resolved_options)?
        };

        let base_dir = skin_path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut sources = SkinSourceManager::new(base_dir);
        sources.set_file_map(skin.file_map.clone());

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
                SkinObjectType::Image => {
                    let image_def = skin.images.get(&obj_data.id).cloned();
                    Box::new(ImageObject::new(obj_data.clone(), image_def))
                }
                SkinObjectType::ImageSet => {
                    let imageset_def = skin.image_sets.get(&obj_data.id).cloned();
                    Box::new(ImageSetObject::new(
                        obj_data.clone(),
                        imageset_def,
                        &skin.images,
                    ))
                }
                SkinObjectType::Slider => {
                    let slider_def = skin.sliders.get(&obj_data.id).cloned();
                    Box::new(SliderObject::new(obj_data.clone(), slider_def))
                }
                SkinObjectType::Bga => Box::new(ImageObject::new(obj_data.clone(), None)),
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

        if skin.header.name.contains("EC:FN") {
            if let Some(def) = skin.images.get("sub_frame") {
                info!(
                    "ECFN sub_frame src {} -> {:?}",
                    def.src,
                    sources.get_path(def.src)
                );
            }
            if let Some(def) = skin.images.get("background") {
                info!(
                    "ECFN background src={} x={} y={} w={} h={}",
                    def.src, def.x, def.y, def.w, def.h
                );
            }
            if let Some(obj) = skin.objects.iter().find(|obj| obj.id == "sub_frame") {
                if let Some(dst) = obj.dst.first() {
                    info!(
                        "ECFN sub_frame dst x={} y={} w={} h={}",
                        dst.x, dst.y, dst.w, dst.h
                    );
                }
            }
            if let Some(obj) = skin.objects.iter().find(|obj| obj.id == "background") {
                if let Some(dst) = obj.dst.first() {
                    info!(
                        "ECFN background dst x={} y={} w={} h={}",
                        dst.x, dst.y, dst.w, dst.h
                    );
                }
            }
            if let Some(def) = skin.images.get("lane-bg") {
                info!(
                    "ECFN lane-bg src {} -> {:?}",
                    def.src,
                    sources.get_path(def.src)
                );
            }
            if let Some(def) = skin.images.get("note-w") {
                info!(
                    "ECFN note-w src {} -> {:?}",
                    def.src,
                    sources.get_path(def.src)
                );
            }
            if let Some(def) = skin.images.get("note-b") {
                info!(
                    "ECFN note-b src {} -> {:?}",
                    def.src,
                    sources.get_path(def.src)
                );
            }
        }

        info!(
            "Skin loaded: {} objects, {} sources, {} fonts",
            objects.len(),
            sources.source_count(),
            sources.font_count()
        );

        let skin_offsets = skin.offsets.clone();

        Ok(Self {
            skin,
            sources,
            objects,
            skin_options,
            skin_offsets,
        })
    }

    /// Draw all skin objects.
    pub fn draw(&self, state: &MainState, now_time_us: i64) {
        let mut state = state.clone();
        state.set_skin_options(&self.skin_options);
        state.set_skin_offsets(&self.skin_offsets);
        for obj in &self.objects {
            obj.draw(&state, &self.sources, now_time_us);
        }
    }

    /// Draw skin objects with BGA rendering interleaved at the correct draw order.
    pub fn draw_with_bga<F>(&self, state: &MainState, now_time_us: i64, mut draw_bga: F)
    where
        F: FnMut(&InterpolatedDest, &crate::skin::SkinObjectData),
    {
        static BGA_LOG_ONCE: Once = Once::new();
        let mut state = state.clone();
        state.set_skin_options(&self.skin_options);
        state.set_skin_offsets(&self.skin_offsets);

        for (obj_data, obj) in self.skin.objects.iter().zip(self.objects.iter()) {
            if obj_data.object_type == SkinObjectType::Bga {
                if !check_option_visibility(&obj_data.op, &state) {
                    continue;
                }
                let elapsed_us = get_timer_elapsed(obj_data.timer, &state, now_time_us);
                if elapsed_us < 0 {
                    continue;
                }
                let elapsed_ms = elapsed_us / 1000;
                let Some(dst) =
                    interpolate_destinations(&obj_data.dst, elapsed_ms, obj_data.loop_count)
                else {
                    continue;
                };
                let dst = apply_offsets(dst, obj_data, &state);
                if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
                    continue;
                }
                BGA_LOG_ONCE.call_once(|| {
                    info!(
                        "ECFN BGA dst: id={} x={} y={} w={} h={} a={} stretch={}",
                        obj_data.id, dst.x, dst.y, dst.w, dst.h, dst.a, obj_data.stretch
                    );
                });
                draw_bga(&dst, obj_data);
                continue;
            }
            obj.draw(&state, &self.sources, now_time_us);
        }
    }

    /// Get skin header metadata.
    pub fn header(&self) -> &SkinHeader {
        &self.skin.header
    }

    /// Collect visible BGA destinations for the current frame.
    pub fn bga_destinations(&self, state: &MainState, now_time_us: i64) -> Vec<InterpolatedDest> {
        let mut state = state.clone();
        state.set_skin_options(&self.skin_options);
        state.set_skin_offsets(&self.skin_offsets);

        let mut destinations = Vec::new();
        for obj in &self.skin.objects {
            if obj.object_type != SkinObjectType::Bga {
                continue;
            }
            if !check_option_visibility(&obj.op, &state) {
                continue;
            }
            let elapsed_us = get_timer_elapsed(obj.timer, &state, now_time_us);
            if elapsed_us < 0 {
                continue;
            }
            let elapsed_ms = elapsed_us / 1000;
            let Some(dst) = interpolate_destinations(&obj.dst, elapsed_ms, obj.loop_count) else {
                continue;
            };
            let dst = apply_offsets(dst, obj, &state);
            if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
                continue;
            }
            destinations.push(dst);
        }
        destinations
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
