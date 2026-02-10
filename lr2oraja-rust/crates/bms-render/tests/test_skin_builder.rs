// Programmatic test skin builder for screenshot tests.
//
// Constructs minimal Skin objects with solid-color images,
// no external skin files needed.

use bms_render::state_provider::StaticStateProvider;
use bms_skin::image_handle::ImageHandle;
use bms_skin::property_id::{BooleanId, FloatId, TimerId};
use bms_skin::skin::Skin;
use bms_skin::skin_graph::SkinGraph;
use bms_skin::skin_header::SkinHeader;
use bms_skin::skin_image::SkinImage;
use bms_skin::skin_object::{Color, Destination, Rect, SkinObjectBase};
use bms_skin::skin_slider::SkinSlider;

use bms_config::resolution::Resolution;

/// Create a solid-color RGBA image.
pub fn solid_color_image(w: u32, h: u32, r: u8, g: u8, b: u8, a: u8) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(w, h);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([r, g, b, a]);
    }
    img
}

/// Helper to make a SkinObjectBase with a single destination at t=0.
fn make_base(x: f32, y: f32, w: f32, h: f32, color: Color) -> SkinObjectBase {
    let mut base = SkinObjectBase::default();
    base.add_destination(Destination {
        time: 0,
        region: Rect::new(x, y, w, h),
        color,
        angle: 0,
        acc: 0,
    });
    base
}

/// Helper to make a SkinObjectBase with two destinations for animation.
fn make_animated_base(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    w: f32,
    h: f32,
    duration_ms: i64,
    color: Color,
) -> SkinObjectBase {
    let mut base = SkinObjectBase::default();
    base.add_destination(Destination {
        time: 0,
        region: Rect::new(x0, y0, w, h),
        color,
        angle: 0,
        acc: 0,
    });
    base.add_destination(Destination {
        time: duration_ms,
        region: Rect::new(x1, y1, w, h),
        color,
        angle: 0,
        acc: 0,
    });
    base
}

/// A pending image to be uploaded into the harness.
pub struct PendingImage {
    pub rgba: image::RgbaImage,
}

/// Builder for constructing test skins programmatically.
pub struct TestSkinBuilder {
    skin: Skin,
    images: Vec<PendingImage>,
    next_handle: u32,
    provider: StaticStateProvider,
}

impl TestSkinBuilder {
    pub fn new(w: f32, h: f32) -> Self {
        let header = SkinHeader {
            source_resolution: Some(Resolution::Sd),
            destination_resolution: Some(Resolution::Sd),
            ..Default::default()
        };
        let mut skin = Skin::new(header);
        // Override dimensions to custom test size
        skin.width = w;
        skin.height = h;
        skin.scale_x = 1.0;
        skin.scale_y = 1.0;

        Self {
            skin,
            images: Vec::new(),
            next_handle: 0,
            provider: StaticStateProvider::default(),
        }
    }

    /// Add a solid-color image object at the given position.
    pub fn add_image(&mut self, x: f32, y: f32, w: f32, h: f32, r: u8, g: u8, b: u8) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let base = make_base(x, y, w, h, color);
        let mut img = SkinImage::from_frames(vec![handle], None, 0);
        img.base = base;
        self.skin.add(img.into());
        self
    }

    /// Add a solid-color image object with custom alpha (0.0-1.0).
    pub fn add_image_with_alpha(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        alpha: f32,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: alpha,
        };
        let base = make_base(x, y, w, h, color);
        let mut img = SkinImage::from_frames(vec![handle], None, 0);
        img.base = base;
        self.skin.add(img.into());
        self
    }

    /// Add an animated image that moves from (x0,y0) to (x1,y1) over duration_ms.
    pub fn add_animated_image(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        duration_ms: i64,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let base = make_animated_base(x0, y0, x1, y1, w, h, duration_ms, color);
        let mut img = SkinImage::from_frames(vec![handle], None, 0);
        img.base = base;
        self.skin.add(img.into());
        self
    }

    /// Add an image object with a draw condition.
    pub fn add_image_with_condition(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        condition_id: i32,
        condition_value: bool,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let mut base = make_base(x, y, w, h, color);
        base.draw_conditions = vec![BooleanId(condition_id)];
        let mut img = SkinImage::from_frames(vec![handle], None, 0);
        img.base = base;
        self.skin.add(img.into());

        self.provider.booleans.insert(condition_id, condition_value);
        self
    }

    /// Add an image object with a timer.
    pub fn add_image_with_timer(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        timer_id: i32,
        timer_value: Option<i64>,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let mut base = make_base(x, y, w, h, color);
        base.timer = Some(TimerId(timer_id));
        let mut img = SkinImage::from_frames(vec![handle], None, 0);
        img.base = base;
        self.skin.add(img.into());

        if let Some(v) = timer_value {
            self.provider.timers.insert(timer_id, v);
        }
        self
    }

    /// Add a slider object.
    pub fn add_slider(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        direction: i32,
        range: i32,
        float_id: i32,
        value: f32,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let base = make_base(x, y, w, h, color);
        let mut slider = SkinSlider::new(FloatId(float_id), direction, range, false);
        slider.base = base;
        slider.source_images = vec![handle];
        self.skin.add(slider.into());

        self.provider.floats.insert(float_id, value);
        self
    }

    /// Add a graph object.
    pub fn add_graph(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: u8,
        g: u8,
        b: u8,
        direction: i32,
        float_id: i32,
        value: f32,
    ) -> &mut Self {
        let handle = ImageHandle(self.next_handle);
        self.next_handle += 1;

        let rgba = solid_color_image(w as u32, h as u32, r, g, b, 255);
        self.images.push(PendingImage { rgba });

        let color = Color::white();
        let base = make_base(x, y, w, h, color);
        let mut graph = SkinGraph::new(FloatId(float_id), direction);
        graph.base = base;
        graph.source_images = vec![handle];
        self.skin.add(graph.into());

        self.provider.floats.insert(float_id, value);
        self
    }

    /// Set the provider time_ms.
    pub fn set_time_ms(&mut self, time_ms: i64) -> &mut Self {
        self.provider.time_ms = time_ms;
        self
    }

    /// Consume the builder and return the skin, images, and state provider.
    pub fn build(self) -> (Skin, Vec<PendingImage>, StaticStateProvider) {
        (self.skin, self.images, self.provider)
    }
}
