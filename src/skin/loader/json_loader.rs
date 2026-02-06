use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::skin::destination::{Destination, DestinationSet};
use crate::skin::object::image::ImageObject;
use crate::skin::object::number::NumberObject;
use crate::skin::skin_data::{SkinData, SkinObject, SkinSource};
use crate::skin::skin_header::{SkinHeader, SkinType};

/// Beatoraja JSON skin loader.
pub struct JsonSkinLoader;

/// JSON skin top-level structure.
#[derive(Debug, Deserialize)]
struct JsonSkin {
    #[serde(rename = "type")]
    skin_type: Option<i32>,
    name: Option<String>,
    w: Option<u32>,
    h: Option<u32>,
    scene: Option<i32>,
    input: Option<i32>,
    fadeout: Option<i32>,
    source: Option<Vec<JsonSource>>,
    destination: Option<Vec<JsonDestination>>,
}

#[derive(Debug, Deserialize)]
struct JsonSource {
    id: i32,
    path: String,
}

#[derive(Debug, Deserialize)]
struct JsonDestination {
    id: Option<String>,
    src: Option<i32>,
    #[serde(rename = "ref")]
    ref_id: Option<i32>,
    x: Option<i32>,
    y: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    divx: Option<i32>,
    divy: Option<i32>,
    digit: Option<i32>,
    timer: Option<i32>,
    #[serde(rename = "loop")]
    loop_ms: Option<i32>,
    blend: Option<i32>,
    filter: Option<i32>,
    center: Option<i32>,
    op: Option<Vec<i32>>,
    offset: Option<i32>,
    dst: Option<Vec<JsonDstEntry>>,
}

#[derive(Debug, Deserialize)]
struct JsonDstEntry {
    time: Option<i64>,
    x: Option<f32>,
    y: Option<f32>,
    w: Option<f32>,
    h: Option<f32>,
    acc: Option<i32>,
    a: Option<i32>,
    r: Option<i32>,
    g: Option<i32>,
    b: Option<i32>,
    angle: Option<i32>,
}

impl JsonSkinLoader {
    /// Load a JSON skin file.
    pub fn load(path: &Path, dst_width: u32, dst_height: u32) -> Result<SkinData> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read JSON skin: {}", path.display()))?;

        let json_skin: JsonSkin = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON skin: {}", path.display()))?;

        let skin_type =
            SkinType::from_id(json_skin.skin_type.unwrap_or(0)).unwrap_or(SkinType::Play7Keys);

        let header = SkinHeader {
            skin_type,
            name: json_skin.name.unwrap_or_default(),
            path: path.to_path_buf(),
            src_width: json_skin.w.unwrap_or(1920),
            src_height: json_skin.h.unwrap_or(1080),
            scene: json_skin.scene.unwrap_or(3_600_000),
            input: json_skin.input.unwrap_or(0),
            fadeout: json_skin.fadeout.unwrap_or(0),
            ..Default::default()
        };

        let mut skin_data = SkinData::new(header, dst_width, dst_height);
        let skin_dir = path.parent().unwrap_or(Path::new("."));

        // Parse sources
        if let Some(sources) = json_skin.source {
            for src in sources {
                skin_data.add_source(SkinSource {
                    id: src.id,
                    path: skin_dir.join(&src.path),
                    texture: None,
                });
            }
        }

        // Parse destinations
        if let Some(destinations) = json_skin.destination {
            for dst in destinations {
                if let Some(obj) = Self::parse_destination(&dst, &skin_data) {
                    skin_data.add_object(obj);
                }
            }
        }

        Ok(skin_data)
    }

    fn parse_destination(dst: &JsonDestination, skin: &SkinData) -> Option<SkinObject> {
        let mut dst_set = DestinationSet {
            timer: dst.timer.unwrap_or(0),
            loop_ms: dst.loop_ms.unwrap_or(0),
            blend: dst.blend.unwrap_or(0),
            filter: dst.filter.unwrap_or(0),
            center: dst.center.unwrap_or(0),
            ..Default::default()
        };

        if let Some(ref ops) = dst.op {
            dst_set.options = ops.iter().copied().filter(|&o| o != 0).collect();
        }
        if let Some(offset) = dst.offset
            && offset > 0
        {
            dst_set.offsets.push(offset);
        }

        if let Some(ref entries) = dst.dst {
            for entry in entries {
                dst_set.add_destination(Destination::new(
                    entry.time.unwrap_or(0),
                    entry.x.unwrap_or(0.0) * skin.scale_x,
                    entry.y.unwrap_or(0.0) * skin.scale_y,
                    entry.w.unwrap_or(0.0) * skin.scale_x,
                    entry.h.unwrap_or(0.0) * skin.scale_y,
                    entry.acc.unwrap_or(0),
                    entry.a.unwrap_or(255),
                    entry.r.unwrap_or(255),
                    entry.g.unwrap_or(255),
                    entry.b.unwrap_or(255),
                    entry.angle.unwrap_or(0),
                ));
            }
        }

        if dst_set.is_empty() {
            return None;
        }

        let id = dst.id.clone().unwrap_or_default();
        let src = dst.src.unwrap_or(-1);

        // Determine type based on fields
        if dst.digit.is_some() {
            return Some(SkinObject::Number(NumberObject {
                id,
                ref_id: dst.ref_id.unwrap_or(0),
                src,
                src_x: dst.x.unwrap_or(0),
                src_y: dst.y.unwrap_or(0),
                src_w: dst.w.unwrap_or(0),
                src_h: dst.h.unwrap_or(0),
                div_x: dst.divx.unwrap_or(10),
                digit: dst.digit.unwrap_or(0),
                dst: dst_set,
                ..Default::default()
            }));
        }

        Some(SkinObject::Image(ImageObject {
            id,
            src,
            src_x: dst.x.unwrap_or(0),
            src_y: dst.y.unwrap_or(0),
            src_w: dst.w.unwrap_or(0),
            src_h: dst.h.unwrap_or(0),
            div_x: dst.divx.unwrap_or(1),
            div_y: dst.divy.unwrap_or(1),
            dst: dst_set,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_json_skin() {
        let json = r#"{
            "type": 0,
            "name": "test",
            "w": 1920,
            "h": 1080,
            "source": [
                {"id": 0, "path": "bg.png"}
            ],
            "destination": [
                {
                    "id": "bg",
                    "src": 0,
                    "x": 0, "y": 0, "w": 1920, "h": 1080,
                    "dst": [
                        {"time": 0, "x": 0, "y": 0, "w": 1920, "h": 1080}
                    ]
                }
            ]
        }"#;

        let skin: JsonSkin = serde_json::from_str(json).unwrap();
        assert_eq!(skin.name.unwrap(), "test");
        assert_eq!(skin.source.as_ref().unwrap().len(), 1);
        assert_eq!(skin.destination.as_ref().unwrap().len(), 1);
    }
}
