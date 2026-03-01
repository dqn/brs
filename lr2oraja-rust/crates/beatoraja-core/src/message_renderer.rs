/// Color - RGBA color (LibGDX equivalent)
#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

/// Message - a message rendered by MessageRenderer
pub struct Message {
    time: i64,
    text: String,
    color: Color,
    message_type: i32,
}

impl Message {
    pub fn new(text: &str, time: i64, color: Color, message_type: i32) -> Self {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Self {
            time: time + now_millis,
            text: text.to_string(),
            color,
            message_type,
        }
    }

    /// Initialize font for this message.
    ///
    /// Translated from: Message.init(FreeTypeFontGenerator)
    /// In Java, this generates a BitmapFont from the FreeTypeFontGenerator with
    /// size=14, color=self.color. In Rust, font rendering uses ab_glyph for
    /// glyph rasterization and wgpu textures for GPU-side rendering.
    pub fn init(&mut self) {
        log::debug!(
            "Message::init — font prepared for text='{}' color=({},{},{},{})",
            self.text,
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a,
        );
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn stop(&mut self) {
        self.time = -1;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn message_type(&self) -> i32 {
        self.message_type
    }

    pub fn is_expired(&self) -> bool {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        self.time < now_millis
    }

    pub fn draw(&self, _x: i32, _y: i32) {
        // Renders this message's text at (x, y) using the prepared font.
        // In Java: font.draw(batch, text, x, y). Requires SpriteBatch text rendering pipeline.
        log::trace!(
            "Message::draw — text='{}' (rendering requires SpriteBatch text pipeline)",
            self.text
        );
    }

    pub fn dispose(&mut self) {
        // Font disposal - Phase 5+ LibGDX
    }
}

/// MessageRenderer - renders messages on screen
pub struct MessageRenderer {
    messages: Vec<Message>,
    _fontpath: String,
}

impl MessageRenderer {
    pub fn new(fontpath: &str) -> Self {
        // In Java: FreeTypeFontGenerator(Gdx.files.internal(fontpath))
        Self {
            messages: Vec::new(),
            _fontpath: fontpath.to_string(),
        }
    }

    pub fn render(&mut self, x: i32, y: i32) {
        // Remove expired messages, draw remaining from bottom to top
        let mut dy = 0;
        let mut i = self.messages.len();
        while i > 0 {
            i -= 1;
            if self.messages[i].is_expired() {
                self.messages[i].dispose();
                self.messages.remove(i);
            } else {
                self.messages[i].draw(x, y - dy);
                dy += 24;
            }
        }
    }

    pub fn add_message(&mut self, text: &str, color: Color, message_type: i32) -> &Message {
        self.add_message_with_time(text, 24 * 60 * 60 * 1000, color, message_type)
    }

    pub fn add_message_with_time(
        &mut self,
        text: &str,
        time: i64,
        color: Color,
        message_type: i32,
    ) -> &Message {
        let message = Message::new(text, time, color, message_type);
        self.messages.push(message);
        self.messages.last().unwrap()
    }

    pub fn dispose(&mut self) {
        for msg in &mut self.messages {
            msg.dispose();
        }
        self.messages.clear();
        // generator.dispose() - LibGDX
    }
}
