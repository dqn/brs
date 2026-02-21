// Color, Rectangle, Matrix4 — pure data types for rendering.
// Drop-in replacements for the same types in rendering_stubs.rs.

/// RGBA color with float components in [0.0, 1.0].
/// Corresponds to com.badlogic.gdx.graphics.Color.
#[derive(Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const CLEAR: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parses a hex color string (e.g. "FF0000FF") into a Color.
    /// Corresponds to com.badlogic.gdx.graphics.Color.valueOf(String)
    pub fn value_of(hex: &str) -> Self {
        let hex = hex.trim();
        let len = hex.len();
        if len < 6 {
            return Color::new(1.0, 0.0, 0.0, 1.0); // fallback red
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        let a = if len >= 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0
        } else {
            1.0
        };
        Color::new(r, g, b, a)
    }

    /// Packs r, g, b, a into an integer (Color.rgba8888 equivalent)
    pub fn rgba8888(r: f32, g: f32, b: f32, a: f32) -> i32 {
        ((255.0 * r) as i32) << 24
            | ((255.0 * g) as i32) << 16
            | ((255.0 * b) as i32) << 8
            | ((255.0 * a) as i32)
    }

    /// Corresponds to com.badlogic.gdx.graphics.Color.toIntBits(a, b, g, r)
    /// Note: LibGDX's toIntBits packs as ABGR
    pub fn to_int_bits(a: i32, b: i32, g: i32, r: i32) -> i32 {
        (a << 24) | (b << 16) | (g << 8) | r
    }

    pub fn set(&mut self, other: &Color) {
        self.r = other.r;
        self.g = other.g;
        self.b = other.b;
        self.a = other.a;
    }

    pub fn set_rgba(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
    }

    pub fn equals(&self, other: &Color) -> bool {
        (self.r - other.r).abs() < f32::EPSILON
            && (self.g - other.g).abs() < f32::EPSILON
            && (self.b - other.b).abs() < f32::EPSILON
            && (self.a - other.a).abs() < f32::EPSILON
    }

    /// Convert to [f32; 4] for GPU uniform/vertex data.
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Axis-aligned rectangle.
/// Corresponds to com.badlogic.gdx.math.Rectangle.
#[derive(Clone, Debug, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn set(&mut self, other: &Rectangle) {
        self.x = other.x;
        self.y = other.y;
        self.width = other.width;
        self.height = other.height;
    }

    pub fn set_xywh(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= 0.0 && x <= self.width && y >= 0.0 && y <= self.height
    }

    pub fn equals(&self, other: &Rectangle) -> bool {
        (self.x - other.x).abs() < f32::EPSILON
            && (self.y - other.y).abs() < f32::EPSILON
            && (self.width - other.width).abs() < f32::EPSILON
            && (self.height - other.height).abs() < f32::EPSILON
    }
}

/// 4x4 transformation matrix stored column-major.
/// Corresponds to com.badlogic.gdx.math.Matrix4.
#[derive(Clone, Debug)]
pub struct Matrix4 {
    pub values: [f32; 16],
}

impl Default for Matrix4 {
    fn default() -> Self {
        // Identity matrix
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

impl Matrix4 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set from translation, quaternion rotation, and scale.
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        tx: f32,
        ty: f32,
        tz: f32,
        qx: f32,
        qy: f32,
        qz: f32,
        qw: f32,
        sx: f32,
        sy: f32,
        sz: f32,
    ) {
        // Convert quaternion to rotation matrix, apply scale and translation
        let xx = qx * qx;
        let xy = qx * qy;
        let xz = qx * qz;
        let xw = qx * qw;
        let yy = qy * qy;
        let yz = qy * qz;
        let yw = qy * qw;
        let zz = qz * qz;
        let zw = qz * qw;

        // Column-major order (same as LibGDX)
        self.values[0] = sx * (1.0 - 2.0 * (yy + zz));
        self.values[1] = sx * 2.0 * (xy + zw);
        self.values[2] = sx * 2.0 * (xz - yw);
        self.values[3] = 0.0;

        self.values[4] = sy * 2.0 * (xy - zw);
        self.values[5] = sy * (1.0 - 2.0 * (xx + zz));
        self.values[6] = sy * 2.0 * (yz + xw);
        self.values[7] = 0.0;

        self.values[8] = sz * 2.0 * (xz + yw);
        self.values[9] = sz * 2.0 * (yz - xw);
        self.values[10] = sz * (1.0 - 2.0 * (xx + yy));
        self.values[11] = 0.0;

        self.values[12] = tx;
        self.values[13] = ty;
        self.values[14] = tz;
        self.values[15] = 1.0;
    }

    /// Create an orthographic projection matrix.
    pub fn set_to_ortho(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;

        self.values = [0.0; 16];
        self.values[0] = 2.0 / rml;
        self.values[5] = 2.0 / tmb;
        self.values[10] = -2.0 / fmn;
        self.values[12] = -(right + left) / rml;
        self.values[13] = -(top + bottom) / tmb;
        self.values[14] = -(far + near) / fmn;
        self.values[15] = 1.0;
    }
}
