// Distance field font material — Bevy Material2d definition.
//
// Wraps the distance_field.wgsl shader for rendering SDF bitmap fonts.
// Used for bitmap_type=1 (standard distance field) and
// bitmap_type=2 (colored distance field).
//
// TODO: Integrate into skin_renderer.rs for BitmapTextMarker entities
// with bitmap_type != 0. Currently only the type definitions and
// shader asset are provided; actual rendering uses standard sprites.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

/// Bitmap font type constants matching Java SkinTextBitmap.
pub const BITMAP_TYPE_STANDARD: i32 = 0;
pub const BITMAP_TYPE_DISTANCE_FIELD: i32 = 1;
pub const BITMAP_TYPE_COLORED_DISTANCE_FIELD: i32 = 2;

/// Material for rendering distance field bitmap fonts.
///
/// Provides uniforms for the distance_field.wgsl shader:
/// glyph color, outline, shadow, and smoothing parameters.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DistanceFieldMaterial {
    /// Glyph foreground color.
    #[uniform(0)]
    pub color: LinearRgba,
    /// Outline color (blended at distance field edge).
    #[uniform(0)]
    pub outline_color: LinearRgba,
    /// Shadow color.
    #[uniform(0)]
    pub shadow_color: LinearRgba,
    /// x = outline_distance (0.0..0.5), y = shadow_smoothing (0.0..0.5), z/w unused.
    #[uniform(0)]
    pub params: Vec4,
    /// x = shadow_offset_x, y = shadow_offset_y, z/w unused.
    #[uniform(0)]
    pub shadow_offset: Vec4,
    /// The SDF texture (font atlas page).
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for DistanceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "distance_field.wgsl".into()
    }
}

impl Default for DistanceFieldMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::WHITE,
            outline_color: LinearRgba::NONE,
            shadow_color: LinearRgba::NONE,
            params: Vec4::new(0.5, 0.0, 0.0, 0.0), // no outline, no shadow
            shadow_offset: Vec4::ZERO,
            texture: Handle::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material_no_outline_no_shadow() {
        let mat = DistanceFieldMaterial::default();
        assert_eq!(mat.color, LinearRgba::WHITE);
        assert_eq!(mat.outline_color, LinearRgba::NONE);
        assert_eq!(mat.shadow_color, LinearRgba::NONE);
        // outline_distance = 0.5 means no outline
        assert!((mat.params.x - 0.5).abs() < f32::EPSILON);
        // shadow_smoothing = 0
        assert!(mat.params.y.abs() < f32::EPSILON);
        assert_eq!(mat.shadow_offset, Vec4::ZERO);
    }

    #[test]
    fn bitmap_type_constants() {
        assert_eq!(BITMAP_TYPE_STANDARD, 0);
        assert_eq!(BITMAP_TYPE_DISTANCE_FIELD, 1);
        assert_eq!(BITMAP_TYPE_COLORED_DISTANCE_FIELD, 2);
    }
}
