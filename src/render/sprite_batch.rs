use crate::render::color::{color_to_array, rotated_quad};
use crate::traits::render::{BlendMode, Color, DstRect, SrcRect, TextureId};

/// Vertex data for a textured quad.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    /// Vertex buffer layout for wgpu pipeline.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // uv
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// A draw command in the sprite batch.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub texture: TextureId,
    pub blend: BlendMode,
    pub vertex_start: u32,
    pub index_start: u32,
    pub index_count: u32,
}

/// Batches textured quads for efficient rendering.
/// Groups draws by texture and blend mode to minimize state changes.
#[derive(Default)]
pub struct SpriteBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub commands: Vec<DrawCommand>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all batched data for a new frame.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.commands.clear();
    }

    /// Add a textured quad to the batch.
    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        texture: TextureId,
        src: SrcRect,
        dst: DstRect,
        tex_width: u32,
        tex_height: u32,
        color: Color,
        angle: f32,
        blend: BlendMode,
    ) {
        let tw = tex_width as f32;
        let th = tex_height as f32;

        // Normalize UV coordinates.
        let u0 = src.x / tw;
        let v0 = src.y / th;
        let u1 = (src.x + src.w) / tw;
        let v1 = (src.y + src.h) / th;

        let positions = rotated_quad(dst.x, dst.y, dst.w, dst.h, angle);
        let rgba = color_to_array(color);

        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            Vertex {
                position: positions[0],
                uv: [u0, v0],
                color: rgba,
            },
            Vertex {
                position: positions[1],
                uv: [u1, v0],
                color: rgba,
            },
            Vertex {
                position: positions[2],
                uv: [u1, v1],
                color: rgba,
            },
            Vertex {
                position: positions[3],
                uv: [u0, v1],
                color: rgba,
            },
        ]);

        let index_start = self.indices.len() as u32;
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Try to merge with the last command if same texture and blend mode.
        if let Some(last) = self.commands.last_mut()
            && last.texture == texture
            && last.blend == blend
        {
            last.index_count += 6;
            return;
        }

        self.commands.push(DrawCommand {
            texture,
            blend,
            vertex_start: base,
            index_start,
            index_count: 6,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_single_quad() {
        let mut batch = SpriteBatch::new();
        batch.push(
            TextureId(1),
            SrcRect {
                x: 0.0,
                y: 0.0,
                w: 64.0,
                h: 64.0,
            },
            DstRect {
                x: 100.0,
                y: 100.0,
                w: 64.0,
                h: 64.0,
            },
            64,
            64,
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.indices.len(), 6);
        assert_eq!(batch.commands.len(), 1);
        assert_eq!(batch.commands[0].index_count, 6);
    }

    #[test]
    fn test_batch_merges_same_texture() {
        let mut batch = SpriteBatch::new();
        let tex = TextureId(1);
        for _ in 0..3 {
            batch.push(
                tex,
                SrcRect {
                    x: 0.0,
                    y: 0.0,
                    w: 32.0,
                    h: 32.0,
                },
                DstRect {
                    x: 0.0,
                    y: 0.0,
                    w: 32.0,
                    h: 32.0,
                },
                32,
                32,
                Color::WHITE,
                0.0,
                BlendMode::Alpha,
            );
        }

        assert_eq!(batch.vertices.len(), 12);
        assert_eq!(batch.indices.len(), 18);
        // Should merge into a single command.
        assert_eq!(batch.commands.len(), 1);
        assert_eq!(batch.commands[0].index_count, 18);
    }

    #[test]
    fn test_different_textures_create_separate_commands() {
        let mut batch = SpriteBatch::new();
        let src = SrcRect {
            x: 0.0,
            y: 0.0,
            w: 32.0,
            h: 32.0,
        };
        let dst = DstRect {
            x: 0.0,
            y: 0.0,
            w: 32.0,
            h: 32.0,
        };

        batch.push(
            TextureId(1),
            src,
            dst,
            32,
            32,
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );
        batch.push(
            TextureId(2),
            src,
            dst,
            32,
            32,
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );

        assert_eq!(batch.commands.len(), 2);
    }

    #[test]
    fn test_different_blend_modes_create_separate_commands() {
        let mut batch = SpriteBatch::new();
        let tex = TextureId(1);
        let src = SrcRect {
            x: 0.0,
            y: 0.0,
            w: 32.0,
            h: 32.0,
        };
        let dst = DstRect {
            x: 0.0,
            y: 0.0,
            w: 32.0,
            h: 32.0,
        };

        batch.push(tex, src, dst, 32, 32, Color::WHITE, 0.0, BlendMode::Alpha);
        batch.push(
            tex,
            src,
            dst,
            32,
            32,
            Color::WHITE,
            0.0,
            BlendMode::Additive,
        );

        assert_eq!(batch.commands.len(), 2);
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let mut batch = SpriteBatch::new();
        batch.push(
            TextureId(1),
            SrcRect {
                x: 32.0,
                y: 16.0,
                w: 64.0,
                h: 32.0,
            },
            DstRect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
            },
            256,
            128,
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );

        // UV should be normalized by texture dimensions.
        let v0 = &batch.vertices[0];
        assert!((v0.uv[0] - 32.0 / 256.0).abs() < 1e-6);
        assert!((v0.uv[1] - 16.0 / 128.0).abs() < 1e-6);

        let v2 = &batch.vertices[2];
        assert!((v2.uv[0] - 96.0 / 256.0).abs() < 1e-6);
        assert!((v2.uv[1] - 48.0 / 128.0).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut batch = SpriteBatch::new();
        batch.push(
            TextureId(1),
            SrcRect {
                x: 0.0,
                y: 0.0,
                w: 32.0,
                h: 32.0,
            },
            DstRect {
                x: 0.0,
                y: 0.0,
                w: 32.0,
                h: 32.0,
            },
            32,
            32,
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );

        batch.clear();
        assert!(batch.vertices.is_empty());
        assert!(batch.indices.is_empty());
        assert!(batch.commands.is_empty());
    }
}
