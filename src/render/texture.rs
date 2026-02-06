use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::traits::render::TextureId;

/// Metadata for a loaded texture.
pub struct TextureEntry {
    pub wgpu_texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}

/// Manages texture loading, caching, and GPU resource creation.
pub struct TextureManager {
    textures: HashMap<TextureId, TextureEntry>,
    path_cache: HashMap<PathBuf, TextureId>,
    next_id: u64,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            path_cache: HashMap::new(),
            next_id: 1,
        }
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_id(&mut self) -> TextureId {
        let id = TextureId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Load a texture from a file path. Returns cached ID if already loaded.
    pub fn load_from_file(
        &mut self,
        path: &Path,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Result<TextureId> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if let Some(&id) = self.path_cache.get(&canonical) {
            return Ok(id);
        }

        let data = std::fs::read(path)
            .map_err(|e| anyhow!("failed to read texture file {}: {}", path.display(), e))?;

        let id = self.load_from_memory(&data, device, queue, bind_group_layout, sampler)?;
        self.path_cache.insert(canonical, id);
        Ok(id)
    }

    /// Load a texture from raw image bytes (PNG, JPEG, etc.).
    pub fn load_from_memory(
        &mut self,
        data: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> Result<TextureId> {
        let img = image::load_from_memory(data)
            .map_err(|e| anyhow!("failed to decode image: {e}"))?
            .to_rgba8();

        let (width, height) = img.dimensions();
        let id = self.alloc_id();

        let entry = create_texture_entry(
            device,
            queue,
            bind_group_layout,
            sampler,
            &img,
            width,
            height,
        );

        self.textures.insert(id, entry);
        Ok(id)
    }

    /// Create a texture from raw RGBA data.
    #[allow(clippy::too_many_arguments)]
    pub fn create_from_rgba(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> TextureId {
        let id = self.alloc_id();
        let entry = create_texture_entry(
            device,
            queue,
            bind_group_layout,
            sampler,
            data,
            width,
            height,
        );
        self.textures.insert(id, entry);
        id
    }

    /// Get texture dimensions.
    pub fn size(&self, id: TextureId) -> Option<(u32, u32)> {
        self.textures.get(&id).map(|e| (e.width, e.height))
    }

    /// Get texture entry for rendering.
    pub fn get(&self, id: TextureId) -> Option<&TextureEntry> {
        self.textures.get(&id)
    }

    /// Get texture view for render target usage.
    pub fn get_view(&self, id: TextureId) -> Option<&wgpu::TextureView> {
        self.textures.get(&id).map(|e| &e.view)
    }

    /// Remove a texture by ID, freeing GPU resources.
    pub fn remove(&mut self, id: TextureId) {
        self.textures.remove(&id);
        self.path_cache.retain(|_, cached_id| *cached_id != id);
    }

    /// Remove all textures and clear caches.
    pub fn clear(&mut self) {
        self.textures.clear();
        self.path_cache.clear();
    }
}

fn create_texture_entry(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    data: &[u8],
    width: u32,
    height: u32,
) -> TextureEntry {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sprite_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texture_bind_group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    TextureEntry {
        wgpu_texture: texture,
        view,
        bind_group,
        width,
        height,
    }
}
