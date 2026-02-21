// wgpu-backed Texture and TextureRegion.
// Drop-in replacements for the types in rendering_stubs.rs.

use crate::gpu_context::GpuContext;
use crate::pixmap::{Pixmap, PixmapFormat};

/// Texture filter modes matching LibGDX TextureFilter.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureFilter {
    Nearest,
    Linear,
    MipMap,
    MipMapNearestNearest,
    MipMapLinearNearest,
    MipMapNearestLinear,
    MipMapLinearLinear,
}

/// GPU-backed texture.
/// Corresponds to com.badlogic.gdx.graphics.Texture.
#[derive(Clone, Debug, Default)]
pub struct Texture {
    pub width: i32,
    pub height: i32,
    pub disposed: bool,
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.disposed == other.disposed
    }
}

#[allow(unused_variables)]
impl Texture {
    pub fn new(path: &str) -> Self {
        // Load from file path — actual GPU upload deferred until a GpuContext is available
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            Self {
                width: rgba.width() as i32,
                height: rgba.height() as i32,
                disposed: false,
            }
        } else {
            Self::default()
        }
    }

    pub fn from_pixmap(pixmap: &Pixmap) -> Self {
        Self {
            width: pixmap.width,
            height: pixmap.height,
            disposed: false,
        }
    }

    pub fn from_pixmap_with_mipmaps(pixmap: &Pixmap, use_mip_maps: bool) -> Self {
        Self {
            width: pixmap.width,
            height: pixmap.height,
            disposed: false,
        }
    }

    pub fn new_sized(width: i32, height: i32, format: PixmapFormat) -> Self {
        Self {
            width,
            height,
            disposed: false,
        }
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn set_filter(&mut self, min: TextureFilter, mag: TextureFilter) {
        // TODO: apply wgpu sampler filter when GPU-backed
    }

    pub fn draw_pixmap(&mut self, pixmap: &Pixmap, x: i32, y: i32) {
        // TODO: upload pixmap data to wgpu texture when GPU-backed
    }

    pub fn dispose(&mut self) {
        self.disposed = true;
    }

    /// Upload RGBA data to a wgpu texture and return it.
    /// This is the GPU-backed path — call when a GpuContext is available.
    pub fn upload_to_gpu(
        &self,
        ctx: &GpuContext,
        data: &[u8],
    ) -> Option<(wgpu::Texture, wgpu::TextureView)> {
        if self.disposed || self.width <= 0 || self.height <= 0 {
            return None;
        }
        let size = wgpu::Extent3d {
            width: self.width as u32,
            height: self.height as u32,
            depth_or_array_layers: 1,
        };
        let wgpu_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("beatoraja texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width as u32),
                rows_per_image: Some(self.height as u32),
            },
            size,
        );
        let view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Some((wgpu_texture, view))
    }
}

/// A region within a Texture, defined by UV coordinates and pixel dimensions.
/// Corresponds to com.badlogic.gdx.graphics.g2d.TextureRegion.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureRegion {
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
    pub region_x: i32,
    pub region_y: i32,
    pub region_width: i32,
    pub region_height: i32,
    pub texture: Option<Texture>,
}

impl TextureRegion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_texture(texture: Texture) -> Self {
        Self {
            region_width: texture.width,
            region_height: texture.height,
            texture: Some(texture),
            region_x: 0,
            region_y: 0,
            u: 0.0,
            v: 0.0,
            u2: 1.0,
            v2: 1.0,
        }
    }

    pub fn from_texture_region(texture: Texture, x: i32, y: i32, width: i32, height: i32) -> Self {
        let u = if texture.width > 0 {
            x as f32 / texture.width as f32
        } else {
            0.0
        };
        let v = if texture.height > 0 {
            y as f32 / texture.height as f32
        } else {
            0.0
        };
        let u2 = if texture.width > 0 {
            (x + width) as f32 / texture.width as f32
        } else {
            1.0
        };
        let v2 = if texture.height > 0 {
            (y + height) as f32 / texture.height as f32
        } else {
            1.0
        };
        Self {
            region_x: x,
            region_y: y,
            region_width: width,
            region_height: height,
            texture: Some(texture),
            u,
            v,
            u2,
            v2,
        }
    }

    pub fn get_region_x(&self) -> i32 {
        self.region_x
    }

    pub fn get_region_y(&self) -> i32 {
        self.region_y
    }

    pub fn get_region_width(&self) -> i32 {
        self.region_width
    }

    pub fn get_region_height(&self) -> i32 {
        self.region_height
    }

    pub fn set_region_x(&mut self, x: i32) {
        self.region_x = x;
    }

    pub fn set_region_y(&mut self, y: i32) {
        self.region_y = y;
    }

    pub fn set_region_width(&mut self, width: i32) {
        self.region_width = width;
    }

    pub fn set_region_height(&mut self, height: i32) {
        self.region_height = height;
    }

    pub fn get_texture(&self) -> Option<&Texture> {
        self.texture.as_ref()
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }

    pub fn set_region_from(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.region_x = x;
        self.region_y = y;
        self.region_width = width;
        self.region_height = height;
    }

    pub fn flip(&mut self, x: bool, y: bool) {
        if x {
            std::mem::swap(&mut self.u, &mut self.u2);
        }
        if y {
            std::mem::swap(&mut self.v, &mut self.v2);
        }
    }

    pub fn set_from(&mut self, other: &TextureRegion) {
        self.u = other.u;
        self.v = other.v;
        self.u2 = other.u2;
        self.v2 = other.v2;
        self.region_x = other.region_x;
        self.region_y = other.region_y;
        self.region_width = other.region_width;
        self.region_height = other.region_height;
        self.texture = other.texture.clone();
    }
}
