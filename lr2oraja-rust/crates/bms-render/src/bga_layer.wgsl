// BGA layer overlay shader — WGSL port of Java layer.frag.
//
// Makes black pixels (R=G=B=0) fully transparent so the base BGA
// layer shows through. Non-black pixels are tinted by the material color.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BgaLayerMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: BgaLayerMaterial;
@group(2) @binding(1)
var base_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(base_texture, base_sampler, in.uv);

    // Discard black pixels (R=G=B=0) — Java layer.frag behavior
    if (c.r == 0.0 && c.g == 0.0 && c.b == 0.0) {
        return vec4(0.0);
    }

    return material.color * c;
}
