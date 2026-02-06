use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};

use crate::traits::render::{BlendMode, Color, DstRect, FontId, RenderBackend, SrcRect, TextureId};

/// Recorded draw command for testing.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    BeginFrame,
    EndFrame,
    Clear(Color),
    DrawSprite {
        texture: TextureId,
        src: SrcRect,
        dst: DstRect,
        color: Color,
        angle: f32,
        blend: BlendMode,
    },
    DrawText {
        font: FontId,
        text: String,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    },
    SetRenderTarget(Option<TextureId>),
}

/// Mock texture data for the command recorder.
struct MockTexture {
    width: u32,
    height: u32,
}

/// A mock RenderBackend that records draw commands for snapshot testing.
/// Does not require a GPU.
pub struct CommandRecorder {
    commands: Vec<DrawCommand>,
    textures: HashMap<TextureId, MockTexture>,
    next_texture_id: u64,
    next_font_id: u64,
    fonts: HashMap<FontId, ()>,
    screen_width: u32,
    screen_height: u32,
}

impl CommandRecorder {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            commands: Vec::new(),
            textures: HashMap::new(),
            next_texture_id: 1,
            next_font_id: 1,
            fonts: HashMap::new(),
            screen_width,
            screen_height,
        }
    }

    /// Get all recorded commands.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Clear recorded commands.
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    /// Register a mock texture with specified dimensions.
    pub fn register_texture(&mut self, width: u32, height: u32) -> TextureId {
        let id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        self.textures.insert(id, MockTexture { width, height });
        id
    }
}

impl RenderBackend for CommandRecorder {
    fn begin_frame(&mut self) -> Result<()> {
        self.commands.push(DrawCommand::BeginFrame);
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        self.commands.push(DrawCommand::EndFrame);
        Ok(())
    }

    fn load_texture(&mut self, _path: &Path) -> Result<TextureId> {
        // For testing, create a default 1x1 mock texture.
        Ok(self.register_texture(1, 1))
    }

    fn load_texture_from_memory(&mut self, data: &[u8]) -> Result<TextureId> {
        // Try to decode image dimensions if valid.
        if let Ok(img) = image::load_from_memory(data) {
            let (w, h) = (img.width(), img.height());
            Ok(self.register_texture(w, h))
        } else {
            // Fallback for raw data.
            Ok(self.register_texture(1, 1))
        }
    }

    fn texture_size(&self, id: TextureId) -> Option<(u32, u32)> {
        self.textures.get(&id).map(|t| (t.width, t.height))
    }

    fn draw_sprite(
        &mut self,
        texture: TextureId,
        src: SrcRect,
        dst: DstRect,
        color: Color,
        angle: f32,
        blend: BlendMode,
    ) -> Result<()> {
        if !self.textures.contains_key(&texture) {
            return Err(anyhow!("unknown texture: {:?}", texture));
        }
        self.commands.push(DrawCommand::DrawSprite {
            texture,
            src,
            dst,
            color,
            angle,
            blend,
        });
        Ok(())
    }

    fn load_font(&mut self, _path: &Path) -> Result<FontId> {
        let id = FontId(self.next_font_id);
        self.next_font_id += 1;
        self.fonts.insert(id, ());
        Ok(id)
    }

    fn draw_text(
        &mut self,
        font: FontId,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> Result<()> {
        if !self.fonts.contains_key(&font) {
            return Err(anyhow!("unknown font: {:?}", font));
        }
        self.commands.push(DrawCommand::DrawText {
            font,
            text: text.to_string(),
            x,
            y,
            size,
            color,
        });
        Ok(())
    }

    fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    fn set_render_target(&mut self, texture: Option<TextureId>) -> Result<()> {
        if let Some(id) = texture
            && !self.textures.contains_key(&id)
        {
            return Err(anyhow!("unknown render target texture: {:?}", id));
        }
        self.commands.push(DrawCommand::SetRenderTarget(texture));
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<()> {
        self.commands.push(DrawCommand::Clear(color));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_end_frame() {
        let mut recorder = CommandRecorder::new(1280, 720);
        recorder.begin_frame().unwrap();
        recorder.end_frame().unwrap();

        assert_eq!(
            recorder.commands(),
            &[DrawCommand::BeginFrame, DrawCommand::EndFrame]
        );
    }

    #[test]
    fn test_draw_sprite() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let tex = recorder.register_texture(64, 64);

        recorder
            .draw_sprite(
                tex,
                SrcRect {
                    x: 0.0,
                    y: 0.0,
                    w: 64.0,
                    h: 64.0,
                },
                DstRect {
                    x: 100.0,
                    y: 100.0,
                    w: 128.0,
                    h: 128.0,
                },
                Color::WHITE,
                0.0,
                BlendMode::Alpha,
            )
            .unwrap();

        assert_eq!(recorder.commands().len(), 1);
        match &recorder.commands()[0] {
            DrawCommand::DrawSprite {
                texture,
                dst,
                blend,
                ..
            } => {
                assert_eq!(*texture, tex);
                assert_eq!(dst.x, 100.0);
                assert_eq!(dst.w, 128.0);
                assert_eq!(*blend, BlendMode::Alpha);
            }
            _ => panic!("expected DrawSprite"),
        }
    }

    #[test]
    fn test_draw_sprite_unknown_texture() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let result = recorder.draw_sprite(
            TextureId(999),
            SrcRect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            DstRect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
            Color::WHITE,
            0.0,
            BlendMode::Alpha,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_draw_text() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let font = recorder.load_font(Path::new("test.ttf")).unwrap();

        recorder
            .draw_text(font, "Hello", 10.0, 20.0, 24.0, Color::WHITE)
            .unwrap();

        assert_eq!(recorder.commands().len(), 1);
        match &recorder.commands()[0] {
            DrawCommand::DrawText { text, x, y, .. } => {
                assert_eq!(text, "Hello");
                assert_eq!(*x, 10.0);
                assert_eq!(*y, 20.0);
            }
            _ => panic!("expected DrawText"),
        }
    }

    #[test]
    fn test_draw_text_unknown_font() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let result = recorder.draw_text(FontId(999), "Hello", 0.0, 0.0, 12.0, Color::WHITE);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let color = Color::new(0.1, 0.2, 0.3, 1.0);
        recorder.clear(color).unwrap();

        assert_eq!(recorder.commands(), &[DrawCommand::Clear(color)]);
    }

    #[test]
    fn test_set_render_target() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let tex = recorder.register_texture(256, 256);

        recorder.set_render_target(Some(tex)).unwrap();
        recorder.set_render_target(None).unwrap();

        assert_eq!(
            recorder.commands(),
            &[
                DrawCommand::SetRenderTarget(Some(tex)),
                DrawCommand::SetRenderTarget(None),
            ]
        );
    }

    #[test]
    fn test_set_render_target_unknown_texture() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let result = recorder.set_render_target(Some(TextureId(999)));
        assert!(result.is_err());
    }

    #[test]
    fn test_screen_size() {
        let recorder = CommandRecorder::new(1920, 1080);
        assert_eq!(recorder.screen_size(), (1920, 1080));
    }

    #[test]
    fn test_load_texture() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let tex = recorder.load_texture(Path::new("nonexistent.png")).unwrap();
        assert_eq!(recorder.texture_size(tex), Some((1, 1)));
    }

    #[test]
    fn test_clear_commands() {
        let mut recorder = CommandRecorder::new(1280, 720);
        recorder.begin_frame().unwrap();
        recorder.clear_commands();
        assert!(recorder.commands().is_empty());
    }

    #[test]
    fn test_full_frame_snapshot() {
        let mut recorder = CommandRecorder::new(1280, 720);
        let tex = recorder.register_texture(64, 64);
        let font = recorder.load_font(Path::new("test.ttf")).unwrap();

        recorder.begin_frame().unwrap();
        recorder.clear(Color::new(0.0, 0.0, 0.0, 1.0)).unwrap();
        recorder
            .draw_sprite(
                tex,
                SrcRect {
                    x: 0.0,
                    y: 0.0,
                    w: 64.0,
                    h: 64.0,
                },
                DstRect {
                    x: 50.0,
                    y: 50.0,
                    w: 64.0,
                    h: 64.0,
                },
                Color::WHITE,
                0.0,
                BlendMode::Alpha,
            )
            .unwrap();
        recorder
            .draw_text(font, "Score: 100", 10.0, 10.0, 16.0, Color::WHITE)
            .unwrap();
        recorder.end_frame().unwrap();

        assert_eq!(recorder.commands().len(), 5);
        assert_eq!(recorder.commands()[0], DrawCommand::BeginFrame);
        assert!(matches!(recorder.commands()[1], DrawCommand::Clear(_)));
        assert!(matches!(
            recorder.commands()[2],
            DrawCommand::DrawSprite { .. }
        ));
        assert!(matches!(
            recorder.commands()[3],
            DrawCommand::DrawText { .. }
        ));
        assert_eq!(recorder.commands()[4], DrawCommand::EndFrame);
    }
}
