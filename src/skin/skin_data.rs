use std::collections::HashMap;
use std::path::PathBuf;

use crate::skin::object::bargraph::BargraphObject;
use crate::skin::object::gauge::GaugeObject;
use crate::skin::object::graph::GraphObject;
use crate::skin::object::image::ImageObject;
use crate::skin::object::image_set::ImageSetObject;
use crate::skin::object::judge::JudgeObject;
use crate::skin::object::number::NumberObject;
use crate::skin::object::slider::SliderObject;
use crate::skin::object::text::TextObject;
use crate::skin::skin_header::SkinHeader;
use crate::traits::render::TextureId;

/// A skin object that can be one of several types.
#[derive(Debug, Clone)]
pub enum SkinObject {
    Image(ImageObject),
    ImageSet(ImageSetObject),
    Number(NumberObject),
    Slider(SliderObject),
    Text(TextObject),
    Graph(GraphObject),
    Gauge(GaugeObject),
    Judge(JudgeObject),
    Bargraph(BargraphObject),
}

/// Source image definition (loaded from skin file).
#[derive(Debug, Clone)]
pub struct SkinSource {
    pub id: i32,
    pub path: PathBuf,
    pub texture: Option<TextureId>,
}

/// Complete skin data container.
#[derive(Debug, Clone)]
pub struct SkinData {
    pub header: SkinHeader,
    /// Source image definitions, keyed by source ID.
    pub sources: HashMap<i32, SkinSource>,
    /// All skin objects in draw order.
    pub objects: Vec<SkinObject>,
    /// Scale factor X (dst_w / src_w).
    pub scale_x: f32,
    /// Scale factor Y (dst_h / src_h).
    pub scale_y: f32,
}

impl SkinData {
    pub fn new(header: SkinHeader, dst_width: u32, dst_height: u32) -> Self {
        let scale_x = dst_width as f32 / header.src_width as f32;
        let scale_y = dst_height as f32 / header.src_height as f32;
        Self {
            header,
            sources: HashMap::new(),
            objects: Vec::new(),
            scale_x,
            scale_y,
        }
    }

    pub fn add_source(&mut self, source: SkinSource) {
        self.sources.insert(source.id, source);
    }

    pub fn add_object(&mut self, object: SkinObject) {
        self.objects.push(object);
    }
}
