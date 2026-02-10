// Distance field font material — Bevy Material2d definition.
//
// Wraps the distance_field.wgsl shader for rendering SDF bitmap fonts.
// Used for bitmap_type=1 (standard distance field) and
// bitmap_type=2 (colored distance field).

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
        "embedded://bms_render/distance_field.wgsl".into()
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

/// Compute outline distance from outline width.
/// outline_width=0 → 0.5 (no outline), larger width → smaller distance (thicker outline).
pub fn compute_outline_distance(outline_width: f32) -> f32 {
    (0.5 - outline_width / 2.0).max(0.1)
}

/// Compute shadow offset in UV space from pixel offsets and page dimensions.
pub fn compute_shadow_offset(offset_x: f32, offset_y: f32, page_w: f32, page_h: f32) -> Vec4 {
    Vec4::new(offset_x / page_w, offset_y / page_h, 0.0, 0.0)
}

/// Compute shadow smoothing from a smoothness value.
pub fn compute_shadow_smoothing(smoothness: f32) -> f32 {
    smoothness / 2.0
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

    #[test]
    fn compute_outline_distance_no_outline() {
        assert!((compute_outline_distance(0.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_outline_distance_thick() {
        // width=0.6 → 0.5 - 0.3 = 0.2
        assert!((compute_outline_distance(0.6) - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_outline_distance_clamped() {
        // width=1.0 → 0.5 - 0.5 = 0.0, clamped to 0.1
        assert!((compute_outline_distance(1.0) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_shadow_offset_basic() {
        let offset = compute_shadow_offset(2.0, 3.0, 256.0, 256.0);
        assert!((offset.x - 2.0 / 256.0).abs() < f32::EPSILON);
        assert!((offset.y - 3.0 / 256.0).abs() < f32::EPSILON);
        assert!(offset.z.abs() < f32::EPSILON);
        assert!(offset.w.abs() < f32::EPSILON);
    }

    #[test]
    fn compute_shadow_smoothing_basic() {
        assert!((compute_shadow_smoothing(0.5) - 0.25).abs() < f32::EPSILON);
        assert!(compute_shadow_smoothing(0.0).abs() < f32::EPSILON);
    }
}
