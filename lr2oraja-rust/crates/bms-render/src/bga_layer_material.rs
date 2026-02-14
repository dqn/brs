// BGA layer overlay material — Bevy Material2d definition.
//
// Wraps the bga_layer.wgsl shader for rendering BGA overlay layers.
// Black pixels (R=G=B=0) are made fully transparent so the base BGA
// layer shows through.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

/// Material for rendering BGA overlay layers with black-key transparency.
///
/// Provides a color uniform and texture for the bga_layer.wgsl shader,
/// which discards pixels where R=G=B=0.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BgaLayerMaterial {
    /// Tint color applied to non-black pixels.
    #[uniform(0)]
    pub color: LinearRgba,
    /// The BGA layer texture.
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for BgaLayerMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://bms_render/bga_layer.wgsl".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_default_fields() {
        let mat = BgaLayerMaterial {
            color: LinearRgba::WHITE,
            texture: Handle::default(),
        };
        assert_eq!(mat.color, LinearRgba::WHITE);
    }
}
