// Embedded texture loader for built-in skin assets.
//
// Decodes PNG images from compile-time embedded bytes and registers them
// in the TextureMap so skin objects can reference them via reserved
// ImageHandle constants (e.g., EMBEDDED_JUDGEDETAIL).

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bms_skin::image_handle::ImageHandle;

use crate::texture_map::TextureMap;

const JUDGEDETAIL_PNG: &[u8] = include_bytes!("../assets/judgedetail.png");

/// Loads all embedded textures and registers them in the texture map.
pub fn load_embedded_textures(images: &mut Assets<Image>, texture_map: &mut TextureMap) {
    load_png(
        images,
        texture_map,
        ImageHandle::EMBEDDED_JUDGEDETAIL,
        JUDGEDETAIL_PNG,
    );
}

fn load_png(
    images: &mut Assets<Image>,
    texture_map: &mut TextureMap,
    handle: ImageHandle,
    png_bytes: &[u8],
) {
    let decoded = image::load_from_memory(png_bytes).expect("Failed to decode embedded PNG");
    let rgba = decoded.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());

    let bevy_image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    let bevy_handle = images.add(bevy_image);
    texture_map.insert(handle, bevy_handle, w as f32, h as f32);
}
