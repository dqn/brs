use macroquad::prelude::*;
use std::sync::OnceLock;

static FONT: OnceLock<Font> = OnceLock::new();

pub fn init_font() {
    let font =
        load_ttf_font_from_bytes(include_bytes!("../../assets/fonts/NotoSansJP-Regular.ttf"))
            .expect("Failed to load font");
    FONT.set(font).expect("Font already initialized");
}

pub fn font() -> &'static Font {
    FONT.get().expect("Font not initialized")
}

pub fn draw_text_jp(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font()),
            font_size: font_size as u16,
            color,
            ..Default::default()
        },
    );
}

pub fn measure_text_jp(text: &str, font_size: f32) -> TextDimensions {
    measure_text(text, Some(font()), font_size as u16, 1.0)
}
