use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::skin::loader::json_loader::JsonSkinLoader;
use crate::skin::loader::lr2_csv_loader::Lr2CsvLoader;
use crate::skin::lua::lua_loader::LuaSkinLoader;
use crate::skin::skin_data::{SkinData, SkinObject};
use crate::traits::render::{RenderBackend, TextureId};

/// Load a skin file, detecting format by extension, and load all textures.
pub fn load_skin(
    path: &Path,
    renderer: &mut dyn RenderBackend,
    dst_width: u32,
    dst_height: u32,
) -> Result<SkinData> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut skin_data = match ext.as_str() {
        "luaskin" | "lua" => {
            let skin_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            let loader = LuaSkinLoader::new(skin_dir);
            loader.load(
                path,
                dst_width,
                dst_height,
                &HashMap::new(),
                &HashMap::new(),
            )?
        }
        "json" => JsonSkinLoader::load(path, dst_width, dst_height)?,
        "csv" | "lr2skin" => Lr2CsvLoader::load(path, dst_width, dst_height)?,
        _ => return Err(anyhow!("Unsupported skin format: {}", ext)),
    };

    // Load textures for all sources
    let mut texture_map: HashMap<i32, TextureId> = HashMap::new();
    for (id, source) in &mut skin_data.sources {
        match renderer.load_texture(&source.path) {
            Ok(tex_id) => {
                source.texture = Some(tex_id);
                texture_map.insert(*id, tex_id);
            }
            Err(e) => {
                tracing::warn!("Failed to load texture {}: {}", source.path.display(), e);
            }
        }
    }

    // Assign textures to objects
    for obj in &mut skin_data.objects {
        assign_texture(obj, &texture_map);
    }

    Ok(skin_data)
}

/// Assign textures to a skin object based on its source ID.
fn assign_texture(obj: &mut SkinObject, texture_map: &HashMap<i32, TextureId>) {
    match obj {
        SkinObject::Image(img) => {
            img.texture = texture_map.get(&img.src).copied();
        }
        SkinObject::Number(num) => {
            num.texture = texture_map.get(&num.src).copied();
        }
        SkinObject::Slider(sl) => {
            sl.texture = texture_map.get(&sl.src).copied();
        }
        SkinObject::Bargraph(bg) => {
            bg.texture = texture_map.get(&bg.src).copied();
        }
        SkinObject::Graph(g) => {
            g.texture = texture_map.get(&g.src).copied();
        }
        SkinObject::ImageSet(is) => {
            for entry in &mut is.images {
                entry.texture = texture_map.get(&entry.src).copied();
            }
        }
        SkinObject::Gauge(gauge) => {
            for entry in &mut gauge.textures {
                entry.texture = texture_map.get(&entry.src).copied();
            }
        }
        SkinObject::Judge(judge) => {
            for entry in &mut judge.textures {
                entry.texture = texture_map.get(&entry.src).copied();
            }
            for entry in &mut judge.number_textures {
                entry.texture = texture_map.get(&entry.src).copied();
            }
        }
        SkinObject::Text(_) => {
            // Text objects use font_id, not textures
        }
    }
}
