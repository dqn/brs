use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::render::color::{color_to_wgpu, ortho_projection};
use crate::render::shader;
use crate::render::sprite_batch::SpriteBatch;
use crate::render::text::{FontData, TextManager};
use crate::render::texture::TextureManager;
use crate::traits::render::{BlendMode, Color, DstRect, FontId, RenderBackend, SrcRect, TextureId};

/// GPU-backed 2D renderer using wgpu.
pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    alpha_pipeline: wgpu::RenderPipeline,
    additive_pipeline: wgpu::RenderPipeline,

    // Retained for potential dynamic pipeline creation.
    _uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,

    sprite_batch: SpriteBatch,
    texture_manager: TextureManager,
    text_manager: TextManager,

    // Reusable vertex/index buffers to avoid per-frame allocation.
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
    index_buffer: Option<wgpu::Buffer>,
    index_capacity: usize,

    // Per-frame text texture cache keyed by (font, text, size bits).
    text_cache: HashMap<(FontId, String, u32), TextureId>,

    temp_textures: Vec<TextureId>,
    current_frame: Option<wgpu::SurfaceTexture>,
    current_render_target: Option<TextureId>,

    screen_width: u32,
    screen_height: u32,
}

impl WgpuRenderer {
    /// Create a new WgpuRenderer for the given window.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| anyhow!("failed to create surface: {e}"))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("failed to find a suitable GPU adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("brs_device"),
                    required_limits: adapter.limits(),
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(|e| anyhow!("failed to create device: {e}"))?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let uniform_bind_group_layout = shader::create_uniform_bind_group_layout(&device);
        let texture_bind_group_layout = shader::create_texture_bind_group_layout(&device);

        let alpha_pipeline = shader::create_sprite_pipeline(
            &device,
            format,
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
            BlendMode::Alpha,
        );
        let additive_pipeline = shader::create_sprite_pipeline(
            &device,
            format,
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
            BlendMode::Additive,
        );

        let projection = ortho_projection(size.width as f32, size.height as f32);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&projection),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            alpha_pipeline,
            additive_pipeline,
            _uniform_bind_group_layout: uniform_bind_group_layout,
            texture_bind_group_layout,
            uniform_buffer,
            uniform_bind_group,
            sampler,
            sprite_batch: SpriteBatch::new(),
            texture_manager: TextureManager::new(),
            text_manager: TextManager::new(),
            vertex_buffer: None,
            vertex_capacity: 0,
            index_buffer: None,
            index_capacity: 0,
            text_cache: HashMap::new(),
            temp_textures: Vec::new(),
            current_frame: None,
            current_render_target: None,
            screen_width: size.width,
            screen_height: size.height,
        })
    }

    /// Resize the renderer surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.screen_width = width;
        self.screen_height = height;
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        let projection = ortho_projection(width as f32, height as f32);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&projection));
    }

    /// Flush the current sprite batch by submitting draw commands to the GPU.
    fn flush_batch(&mut self, view: &wgpu::TextureView) {
        if self.sprite_batch.commands.is_empty() {
            return;
        }

        let vertex_data = bytemuck::cast_slice(&self.sprite_batch.vertices);
        let index_data = bytemuck::cast_slice(&self.sprite_batch.indices);

        // Reuse vertex buffer if capacity is sufficient, otherwise recreate.
        if self.vertex_capacity < vertex_data.len() {
            self.vertex_capacity = vertex_data.len().next_power_of_two();
            self.vertex_buffer = Some(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("vertex_buffer"),
                    contents: vertex_data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.queue
                .write_buffer(self.vertex_buffer.as_ref().unwrap(), 0, vertex_data);
        }

        // Reuse index buffer if capacity is sufficient, otherwise recreate.
        if self.index_capacity < index_data.len() {
            self.index_capacity = index_data.len().next_power_of_two();
            self.index_buffer = Some(self.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("index_buffer"),
                    contents: index_data,
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            self.queue
                .write_buffer(self.index_buffer.as_ref().unwrap(), 0, index_data);
        }

        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let index_buffer = self.index_buffer.as_ref().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("sprite_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            for cmd in &self.sprite_batch.commands {
                let pipeline = match cmd.blend {
                    BlendMode::Alpha => &self.alpha_pipeline,
                    BlendMode::Additive => &self.additive_pipeline,
                };
                render_pass.set_pipeline(pipeline);

                if let Some(entry) = self.texture_manager.get(cmd.texture) {
                    render_pass.set_bind_group(1, &entry.bind_group, &[]);
                    render_pass.draw_indexed(
                        cmd.index_start..cmd.index_start + cmd.index_count,
                        0,
                        0..1,
                    );
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.sprite_batch.clear();
    }

    /// Get the render target view (either offscreen texture or surface frame).
    fn get_target_view(&self) -> Option<wgpu::TextureView> {
        if let Some(target_id) = self.current_render_target {
            self.texture_manager.get_view(target_id).cloned()
        } else {
            self.current_frame.as_ref().map(|f| {
                f.texture
                    .create_view(&wgpu::TextureViewDescriptor::default())
            })
        }
    }
}

impl RenderBackend for WgpuRenderer {
    fn begin_frame(&mut self) -> Result<()> {
        self.text_cache.clear();
        for id in self.temp_textures.drain(..) {
            self.texture_manager.remove(id);
        }

        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| anyhow!("failed to get surface texture: {e}"))?;
        self.current_frame = Some(frame);
        self.sprite_batch.clear();
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        if let Some(view) = self.get_target_view() {
            self.flush_batch(&view);
        }

        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        Ok(())
    }

    fn load_texture(&mut self, path: &Path) -> Result<TextureId> {
        self.texture_manager.load_from_file(
            path,
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.sampler,
        )
    }

    fn load_texture_from_memory(&mut self, data: &[u8]) -> Result<TextureId> {
        self.texture_manager.load_from_memory(
            data,
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.sampler,
        )
    }

    fn texture_size(&self, id: TextureId) -> Option<(u32, u32)> {
        self.texture_manager.size(id)
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
        let (tw, th) = self
            .texture_manager
            .size(texture)
            .ok_or_else(|| anyhow!("unknown texture: {:?}", texture))?;
        self.sprite_batch
            .push(texture, src, dst, tw, th, color, angle, blend);
        Ok(())
    }

    fn load_font(&mut self, path: &Path) -> Result<FontId> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "ttf" | "otf" => self.text_manager.load_ttf(path),
            "fnt" => {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| anyhow!("failed to read FNT {}: {}", path.display(), e))?;
                let font = crate::render::font::fnt_parser::parse_fnt(&content)?;
                let font_dir = path.parent().unwrap_or(Path::new("."));

                let mut page_textures = Vec::with_capacity(font.pages.len());
                for page_file in &font.pages {
                    let page_path = font_dir.join(page_file);
                    let tex_id = self.texture_manager.load_from_file(
                        &page_path,
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                        &self.sampler,
                    )?;
                    page_textures.push(tex_id);
                }

                Ok(self.text_manager.register_bitmap_font(font, page_textures))
            }
            _ => Err(anyhow!("unsupported font format: .{ext}")),
        }
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
        // We need to get the font data by pointer to avoid borrow issues.
        let font_data = self
            .text_manager
            .get(font)
            .ok_or_else(|| anyhow!("unknown font: {:?}", font))?;

        match font_data {
            FontData::TrueType(renderer) => {
                let cache_key = (font, text.to_string(), size.to_bits());
                let tex_id = if let Some(&cached) = self.text_cache.get(&cache_key) {
                    cached
                } else {
                    let (w, h, rgba) = renderer.rasterize(text, size)?;
                    if w == 0 || h == 0 {
                        return Ok(());
                    }

                    let id = self.texture_manager.create_from_rgba(
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                        &self.sampler,
                        &rgba,
                        w,
                        h,
                    );
                    self.temp_textures.push(id);
                    self.text_cache.insert(cache_key, id);
                    id
                };

                let (w, h) = self
                    .texture_manager
                    .size(tex_id)
                    .ok_or_else(|| anyhow!("text texture missing"))?;

                self.sprite_batch.push(
                    tex_id,
                    SrcRect {
                        x: 0.0,
                        y: 0.0,
                        w: w as f32,
                        h: h as f32,
                    },
                    DstRect {
                        x,
                        y,
                        w: w as f32,
                        h: h as f32,
                    },
                    w,
                    h,
                    color,
                    0.0,
                    BlendMode::Alpha,
                );

                Ok(())
            }
            FontData::Bitmap(bfont_data) => {
                let scale = size / bfont_data.font.size as f32;
                let mut cursor_x = x;
                let mut prev_char = None;

                for ch in text.chars() {
                    let cp = ch as u32;

                    if let Some(prev) = prev_char
                        && let Some(&kern) = bfont_data.font.kernings.get(&(prev, cp))
                    {
                        cursor_x += kern as f32 * scale;
                    }

                    if let Some(glyph) = bfont_data.font.glyphs.get(&cp) {
                        let gx = cursor_x + glyph.xoffset as f32 * scale;
                        let gy = y + glyph.yoffset as f32 * scale;
                        let gw = glyph.width as f32 * scale;
                        let gh = glyph.height as f32 * scale;

                        if let Some(&tex_id) = bfont_data.page_textures.get(glyph.page as usize) {
                            let (tw, th) = self
                                .texture_manager
                                .size(tex_id)
                                .ok_or_else(|| anyhow!("missing bitmap font page texture"))?;

                            self.sprite_batch.push(
                                tex_id,
                                SrcRect {
                                    x: glyph.x as f32,
                                    y: glyph.y as f32,
                                    w: glyph.width as f32,
                                    h: glyph.height as f32,
                                },
                                DstRect {
                                    x: gx,
                                    y: gy,
                                    w: gw,
                                    h: gh,
                                },
                                tw,
                                th,
                                color,
                                0.0,
                                BlendMode::Alpha,
                            );
                        }

                        cursor_x += glyph.xadvance as f32 * scale;
                    }

                    prev_char = Some(cp);
                }
                Ok(())
            }
        }
    }

    fn screen_size(&self) -> (u32, u32) {
        (self.screen_width, self.screen_height)
    }

    fn set_render_target(&mut self, texture: Option<TextureId>) -> Result<()> {
        // Flush current batch before switching targets.
        if let Some(view) = self.get_target_view() {
            self.flush_batch(&view);
        }
        self.current_render_target = texture;
        Ok(())
    }

    fn clear(&mut self, color: Color) -> Result<()> {
        let view = self
            .get_target_view()
            .ok_or_else(|| anyhow!("no render target available"))?;

        // Flush any pending draws before clearing.
        self.flush_batch(&view);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear_encoder"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color_to_wgpu(color)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
