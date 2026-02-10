// Distance field font shader — WGSL port of Java distance_field.frag.
//
// Renders signed distance field (SDF) bitmap fonts with:
//   - Anti-aliased edges via smoothstep
//   - Outline rendering (configurable thickness and color)
//   - Drop shadow (configurable offset, smoothing, and color)
//
// bitmap_type=1: standard distance field (monochrome alpha channel)
// bitmap_type=2: colored distance field (same technique, colored texture)

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct DistanceFieldMaterial {
    color: vec4<f32>,
    outline_color: vec4<f32>,
    shadow_color: vec4<f32>,
    // x = outline_distance (0..0.5, 0 = thick, 0.5 = none)
    // y = shadow_smoothing (0..0.5)
    params: vec4<f32>,
    // x = shadow_offset_x, y = shadow_offset_y
    shadow_offset: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: DistanceFieldMaterial;
@group(2) @binding(1)
var base_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_sampler: sampler;

const SMOOTHING: f32 = 0.0625; // 1.0 / 16.0

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = textureSample(base_texture, base_sampler, in.uv).a;

    // Main glyph with outline
    let outline_factor = smoothstep(0.5 - SMOOTHING, 0.5 + SMOOTHING, distance);
    let blended_color = mix(material.outline_color, material.color, vec4<f32>(outline_factor));
    let outline_distance = material.params.x;
    let alpha = smoothstep(outline_distance - SMOOTHING, outline_distance + SMOOTHING, distance);
    let main_color = vec4<f32>(blended_color.rgb, blended_color.a * alpha);

    // Shadow
    let shadow_uv = in.uv - material.shadow_offset.xy;
    let shadow_distance = textureSample(base_texture, base_sampler, shadow_uv).a;
    let shadow_smoothing = material.params.y;
    let shadow_alpha = smoothstep(0.5 - shadow_smoothing, 0.5 + shadow_smoothing, shadow_distance);
    let shadow = vec4<f32>(material.shadow_color.rgb, material.shadow_color.a * shadow_alpha);

    // Composite: main over shadow
    return mix(shadow, main_color, vec4<f32>(main_color.a));
}
