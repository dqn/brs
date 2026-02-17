// Sub-object resolution for JSON skin loading.
//
// Resolves image, number, text, and bar sub-destination references
// used by composite skin objects like song lists, judges, etc.

use std::collections::HashMap;
use std::path::Path;

use crate::image_handle::ImageHandle;
use crate::loader::json_skin::{FlexId, JsonSkinData};
use crate::property_id;
use crate::skin_object::SkinObjectBase;
use crate::skin_source::{build_number_source_set, split_grid};
use crate::skin_visualizer::parse_color;

use super::{apply_destination, resolve_font_type};

use crate::loader::json_skin::JsonDestination;

/// Resolves an image FlexId to an ImageHandle integer.
///
/// Searches `data.image` for a matching ID, then looks up its `src` in
/// `source_images` to get the handle value.
pub(super) fn resolve_image_ref(
    data: &JsonSkinData,
    flex_id: &FlexId,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<i32> {
    let img_def = data.image.iter().find(|i| i.id == *flex_id)?;
    let handle = source_images.get(img_def.src.as_str())?;
    Some(handle.0 as i32)
}

/// Resolves a sub-destination reference to a SkinImage.
///
/// Finds the image definition matching the sub-destination ID, builds a
/// SkinImage with its source handle, and applies the sub-destination.
pub(super) fn resolve_sub_image(
    data: &JsonSkinData,
    sub_dst: &JsonDestination,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<crate::skin_image::SkinImage> {
    let img_def = data.image.iter().find(|i| i.id == sub_dst.id)?;
    let handle = source_images.get(img_def.src.as_str())?;
    let timer = img_def.timer.as_ref().and_then(|t| t.as_id());
    let mut img = crate::skin_image::SkinImage::from_frames(vec![*handle], timer, img_def.cycle);
    apply_destination(&mut img.base, sub_dst);
    Some(img)
}

/// Resolves a sub-destination reference to a SkinNumber.
///
/// Finds the value definition matching the sub-destination ID, builds a
/// SkinNumber with the same logic as try_build_number(), and applies the
/// sub-destination.
pub(super) fn resolve_sub_number(
    data: &JsonSkinData,
    sub_dst: &JsonDestination,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<crate::skin_number::SkinNumber> {
    let val_def = data.value.iter().find(|v| v.id == sub_dst.id)?;
    let ref_id = if let Some(ref val) = val_def.value {
        val.as_id().unwrap_or(val_def.ref_id)
    } else {
        val_def.ref_id
    };

    let timer = val_def.timer.as_ref().and_then(|t| t.as_id());

    // Resolve source image and split into grid
    let grid = source_images
        .get(val_def.src.as_str())
        .map(|&handle| {
            split_grid(
                handle,
                val_def.x,
                val_def.y,
                val_def.w,
                val_def.h,
                val_def.divx,
                val_def.divy,
            )
        })
        .unwrap_or_default();

    let (digit_sources, minus_digit_sources, zeropadding_override) =
        build_number_source_set(&grid, timer, val_def.cycle);

    let zeropadding = zeropadding_override.unwrap_or(val_def.zeropadding);

    let mut num = crate::skin_number::SkinNumber {
        base: SkinObjectBase::default(),
        ref_id: Some(property_id::IntegerId(ref_id)),
        keta: val_def.digit,
        zero_padding: crate::skin_number::ZeroPadding::from_i32(zeropadding),
        align: crate::skin_number::NumberAlign::from_i32(val_def.align),
        space: val_def.space,
        digit_sources,
        minus_digit_sources,
        ..Default::default()
    };
    apply_destination(&mut num.base, sub_dst);
    if let Some(offsets) = &val_def.offset {
        num.digit_offsets = offsets
            .iter()
            .map(|o| crate::skin_object::SkinOffset {
                x: o.x as f32,
                y: o.y as f32,
                w: o.w as f32,
                h: o.h as f32,
                ..Default::default()
            })
            .collect();
    }
    Some(num)
}

/// Resolves a sub-destination reference to a SkinText.
///
/// Finds the text definition matching the sub-destination ID, builds a
/// SkinText with the same logic as try_build_text(), and applies the
/// sub-destination.
pub(super) fn resolve_sub_text(
    data: &JsonSkinData,
    sub_dst: &JsonDestination,
    skin_path: Option<&Path>,
) -> Option<crate::skin_text::SkinText> {
    let text_def = data.text.iter().find(|t| t.id == sub_dst.id)?;
    let ref_id = if let Some(ref val) = text_def.value {
        val.as_id().unwrap_or(text_def.ref_id)
    } else {
        text_def.ref_id
    };
    let outline_color = if text_def.outline_width > 0.0 {
        Some(parse_color(&text_def.outline_color))
    } else {
        None
    };
    let shadow = if text_def.shadow_offset_x != 0.0 || text_def.shadow_offset_y != 0.0 {
        Some(crate::skin_text::TextShadow {
            color: parse_color(&text_def.shadow_color),
            offset_x: text_def.shadow_offset_x,
            offset_y: text_def.shadow_offset_y,
            smoothness: text_def.shadow_smoothness,
        })
    } else {
        None
    };
    let font_type = resolve_font_type(data, &text_def.font, skin_path);
    let mut text = crate::skin_text::SkinText {
        base: SkinObjectBase::default(),
        ref_id: Some(property_id::StringId(ref_id)),
        constant_text: text_def.constant_text.clone(),
        font_size: text_def.size as f32,
        align: crate::skin_text::TextAlign::from_i32(text_def.align),
        wrapping: text_def.wrapping,
        overflow: crate::skin_text::TextOverflow::from_i32(text_def.overflow),
        outline_color,
        outline_width: text_def.outline_width,
        shadow,
        font_type,
        ..Default::default()
    };
    apply_destination(&mut text.base, sub_dst);
    Some(text)
}

/// Resolves a bar image from imageset or image.
///
/// First searches `data.imageset` for the sub-destination ID. If found,
/// resolves all images in the set. Falls back to `resolve_sub_image()`.
pub(super) fn resolve_bar_image(
    data: &JsonSkinData,
    sub_dst: &JsonDestination,
    source_images: &HashMap<String, ImageHandle>,
) -> Option<crate::skin_image::SkinImage> {
    // Try imageset first
    if let Some(set_def) = data.imageset.iter().find(|s| s.id == sub_dst.id) {
        let ref_id = if let Some(ref val) = set_def.value {
            val.as_id().unwrap_or(set_def.ref_id)
        } else {
            set_def.ref_id
        };
        let mut sources = Vec::new();
        for image_ref in &set_def.images {
            let Some(image_def) = data.image.iter().find(|img| img.id == *image_ref) else {
                continue;
            };
            let Some(&handle) = source_images.get(image_def.src.as_str()) else {
                continue;
            };
            let timer = image_def.timer.as_ref().and_then(|t| t.as_id());
            sources.push(crate::skin_image::SkinImageSource::Frames {
                images: vec![handle],
                timer,
                cycle: image_def.cycle,
            });
        }
        if sources.is_empty() {
            return None;
        }
        let mut img = if ref_id != 0 {
            crate::skin_image::SkinImage::with_ref(sources, property_id::IntegerId(ref_id))
        } else {
            crate::skin_image::SkinImage {
                sources,
                ..Default::default()
            }
        };
        apply_destination(&mut img.base, sub_dst);
        return Some(img);
    }
    // Fall back to single image
    resolve_sub_image(data, sub_dst, source_images)
}
