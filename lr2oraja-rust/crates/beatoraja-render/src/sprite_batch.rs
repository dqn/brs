// Batched 2D quad renderer.
// Drop-in replacement for the SpriteBatch stub in rendering_stubs.rs.

use crate::color::{Color, Matrix4};
use crate::shader::ShaderProgram;
use crate::texture::{Texture, TextureRegion};

/// Vertex for a 2D sprite quad.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    /// Returns the wgpu vertex buffer layout for SpriteVertex.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Batched 2D sprite renderer.
/// Corresponds to com.badlogic.gdx.graphics.g2d.SpriteBatch.
///
/// Collects sprite draw calls into a vertex buffer. Actual GPU submission
/// happens when `flush()` is called or when the batch reaches capacity.
#[derive(Debug, Default)]
pub struct SpriteBatch {
    vertices: Vec<SpriteVertex>,
    current_color: [f32; 4],
    blend_src: i32,
    blend_dst: i32,
}

#[allow(unused_variables)]
impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(4096),
            current_color: [1.0, 1.0, 1.0, 1.0],
            blend_src: 0x0302, // GL_SRC_ALPHA
            blend_dst: 0x0303, // GL_ONE_MINUS_SRC_ALPHA
        }
    }

    pub fn set_transform_matrix(&mut self, matrix: &Matrix4) {
        // TODO: store transform for GPU uniform when rendering pipeline is wired
    }

    pub fn set_shader(&mut self, shader: Option<&ShaderProgram>) {
        // TODO: switch render pipeline
    }

    pub fn set_color(&mut self, color: &Color) {
        self.current_color = color.to_array();
    }

    pub fn get_color(&self) -> Color {
        Color::new(
            self.current_color[0],
            self.current_color[1],
            self.current_color[2],
            self.current_color[3],
        )
    }

    pub fn set_blend_function(&mut self, src: i32, dst: i32) {
        self.blend_src = src;
        self.blend_dst = dst;
    }

    pub fn flush(&mut self) {
        // TODO: submit vertex buffer to GPU when rendering pipeline is wired
        self.vertices.clear();
    }

    /// Draw a full texture at (x, y) with size (w, h).
    pub fn draw_texture(&mut self, texture: &Texture, x: f32, y: f32, w: f32, h: f32) {
        self.push_quad(x, y, w, h, 0.0, 0.0, 1.0, 1.0);
    }

    /// Draw a texture region at (x, y) with size (w, h).
    pub fn draw_region(&mut self, region: &TextureRegion, x: f32, y: f32, w: f32, h: f32) {
        self.push_quad(x, y, w, h, region.u, region.v, region.u2, region.v2);
    }

    /// Draw a texture region with rotation and scale.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_region_rotated(
        &mut self,
        region: &TextureRegion,
        x: f32,
        y: f32,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        sx: f32,
        sy: f32,
        angle: f32,
    ) {
        let cos = angle.to_radians().cos();
        let sin = angle.to_radians().sin();

        // Compute corner offsets from origin, apply scale
        let corners: [(f32, f32); 4] = [
            (-cx * sx, -cy * sy),
            ((w - cx) * sx, -cy * sy),
            ((w - cx) * sx, (h - cy) * sy),
            (-cx * sx, (h - cy) * sy),
        ];

        let color = self.current_color;
        let (u1, v1, u2, v2) = (region.u, region.v, region.u2, region.v2);
        let uvs = [(u1, v1), (u2, v1), (u2, v2), (u1, v2)];

        // Two triangles: 0-1-2, 0-2-3
        for &idx in &[0, 1, 2, 0, 2, 3] {
            let (ox, oy) = corners[idx];
            let px = x + cx + ox * cos - oy * sin;
            let py = y + cy + ox * sin + oy * cos;
            self.vertices.push(SpriteVertex {
                position: [px, py],
                tex_coord: [uvs[idx].0, uvs[idx].1],
                color,
            });
        }
    }

    /// Get the raw vertex data for GPU upload.
    pub fn vertices(&self) -> &[SpriteVertex] {
        &self.vertices
    }

    /// Push a simple axis-aligned quad.
    #[allow(clippy::too_many_arguments)]
    fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32, u1: f32, v1: f32, u2: f32, v2: f32) {
        let color = self.current_color;
        // Two triangles: top-left, top-right, bottom-right, top-left, bottom-right, bottom-left
        let verts = [
            SpriteVertex {
                position: [x, y],
                tex_coord: [u1, v1],
                color,
            },
            SpriteVertex {
                position: [x + w, y],
                tex_coord: [u2, v1],
                color,
            },
            SpriteVertex {
                position: [x + w, y + h],
                tex_coord: [u2, v2],
                color,
            },
            SpriteVertex {
                position: [x, y],
                tex_coord: [u1, v1],
                color,
            },
            SpriteVertex {
                position: [x + w, y + h],
                tex_coord: [u2, v2],
                color,
            },
            SpriteVertex {
                position: [x, y + h],
                tex_coord: [u1, v2],
                color,
            },
        ];
        self.vertices.extend_from_slice(&verts);
    }
}
