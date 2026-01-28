//! Converter from beatoraja skin format to internal representation

use super::types::{BeatorajaSkin, Destination, ImageElement, SkinType};

/// Resolved destination for rendering
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedDestination {
    /// X position (screen coordinates)
    pub x: f32,
    /// Y position (screen coordinates)
    pub y: f32,
    /// Width
    pub w: f32,
    /// Height
    pub h: f32,
    /// Alpha (0.0-1.0)
    pub a: f32,
    /// Red (0.0-1.0)
    pub r: f32,
    /// Green (0.0-1.0)
    pub g: f32,
    /// Blue (0.0-1.0)
    pub b: f32,
    /// Rotation angle (degrees)
    pub angle: f32,
}

impl ResolvedDestination {
    /// Create from a beatoraja destination
    pub fn from_destination(dst: &Destination, scale_x: f32, scale_y: f32) -> Self {
        Self {
            x: dst.x as f32 * scale_x,
            y: dst.y as f32 * scale_y,
            w: dst.w as f32 * scale_x,
            h: dst.h as f32 * scale_y,
            a: dst.a as f32 / 255.0,
            r: dst.r as f32 / 255.0,
            g: dst.g as f32 / 255.0,
            b: dst.b as f32 / 255.0,
            angle: dst.angle as f32,
        }
    }

    /// Interpolate between two destinations
    pub fn interpolate(from: &ResolvedDestination, to: &ResolvedDestination, t: f32) -> Self {
        Self {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
            w: from.w + (to.w - from.w) * t,
            h: from.h + (to.h - from.h) * t,
            a: from.a + (to.a - from.a) * t,
            r: from.r + (to.r - from.r) * t,
            g: from.g + (to.g - from.g) * t,
            b: from.b + (to.b - from.b) * t,
            angle: from.angle + (to.angle - from.angle) * t,
        }
    }
}

/// Image region in source texture
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageRegion {
    /// Source X
    pub x: u32,
    /// Source Y
    pub y: u32,
    /// Region width
    pub w: u32,
    /// Region height
    pub h: u32,
}

impl ImageRegion {
    /// Get a sub-region for frame animation
    pub fn get_frame(&self, frame: u32, divx: u32, divy: u32) -> Self {
        if divx == 0 || divy == 0 {
            return *self;
        }

        let frame_w = self.w / divx;
        let frame_h = self.h / divy;
        let frame_x = frame % divx;
        let frame_y = (frame / divx) % divy;

        Self {
            x: self.x + frame_x * frame_w,
            y: self.y + frame_y * frame_h,
            w: frame_w,
            h: frame_h,
        }
    }

    /// Create UV coordinates for rendering (normalized 0.0-1.0)
    pub fn to_uv(self, texture_w: u32, texture_h: u32) -> (f32, f32, f32, f32) {
        if texture_w == 0 || texture_h == 0 {
            return (0.0, 0.0, 1.0, 1.0);
        }

        let u1 = self.x as f32 / texture_w as f32;
        let v1 = self.y as f32 / texture_h as f32;
        let u2 = (self.x + self.w) as f32 / texture_w as f32;
        let v2 = (self.y + self.h) as f32 / texture_h as f32;

        (u1, v1, u2, v2)
    }
}

/// Skin scale factors
#[derive(Debug, Clone, Copy)]
pub struct SkinScale {
    /// Horizontal scale (screen_width / skin_width)
    pub x: f32,
    /// Vertical scale (screen_height / skin_height)
    pub y: f32,
}

impl SkinScale {
    /// Create scale factors from skin and screen dimensions
    pub fn new(skin_w: i32, skin_h: i32, screen_w: f32, screen_h: f32) -> Self {
        Self {
            x: screen_w / skin_w as f32,
            y: screen_h / skin_h as f32,
        }
    }

    /// Scale a position
    pub fn scale_pos(&self, x: i32, y: i32) -> (f32, f32) {
        (x as f32 * self.x, y as f32 * self.y)
    }

    /// Scale a size
    pub fn scale_size(&self, w: i32, h: i32) -> (f32, f32) {
        (w as f32 * self.x, h as f32 * self.y)
    }
}

/// Skin type information
#[derive(Debug, Clone, Copy)]
pub struct SkinTypeInfo {
    /// Skin type
    pub skin_type: SkinType,
    /// Number of lanes (for play skins)
    pub lane_count: usize,
    /// Has scratch lane
    pub has_scratch: bool,
    /// Is double play
    pub is_double: bool,
}

impl SkinTypeInfo {
    /// Get type info from skin type value
    pub fn from_type(skin_type: i32) -> Self {
        match skin_type {
            0 => Self {
                skin_type: SkinType::PLAY_7KEYS,
                lane_count: 8,
                has_scratch: true,
                is_double: false,
            },
            1 => Self {
                skin_type: SkinType::PLAY_5KEYS,
                lane_count: 6,
                has_scratch: true,
                is_double: false,
            },
            2 => Self {
                skin_type: SkinType::PLAY_14KEYS,
                lane_count: 16,
                has_scratch: true,
                is_double: true,
            },
            3 => Self {
                skin_type: SkinType::PLAY_10KEYS,
                lane_count: 12,
                has_scratch: true,
                is_double: true,
            },
            4 => Self {
                skin_type: SkinType::PLAY_9KEYS,
                lane_count: 9,
                has_scratch: false,
                is_double: false,
            },
            5 => Self {
                skin_type: SkinType::MUSIC_SELECT,
                lane_count: 0,
                has_scratch: false,
                is_double: false,
            },
            6 => Self {
                skin_type: SkinType::DECIDE,
                lane_count: 0,
                has_scratch: false,
                is_double: false,
            },
            7 => Self {
                skin_type: SkinType::RESULT,
                lane_count: 0,
                has_scratch: false,
                is_double: false,
            },
            8 => Self {
                skin_type: SkinType::COURSE_RESULT,
                lane_count: 0,
                has_scratch: false,
                is_double: false,
            },
            _ => Self {
                skin_type: SkinType::PLAY_7KEYS,
                lane_count: 8,
                has_scratch: true,
                is_double: false,
            },
        }
    }

    /// Check if this is a play skin
    pub fn is_play_skin(&self) -> bool {
        self.lane_count > 0
    }
}

/// Resolve current destination from animation keyframes
pub fn resolve_destination(
    element: &ImageElement,
    elapsed_ms: i32,
    scale: &SkinScale,
) -> Option<ResolvedDestination> {
    if element.dst.is_empty() {
        return None;
    }

    // Single destination - no animation
    if element.dst.len() == 1 {
        return Some(ResolvedDestination::from_destination(
            &element.dst[0],
            scale.x,
            scale.y,
        ));
    }

    // Find the two keyframes to interpolate between
    let mut prev_dst = &element.dst[0];
    let mut next_dst = &element.dst[0];

    for i in 0..element.dst.len() {
        if element.dst[i].time <= elapsed_ms {
            prev_dst = &element.dst[i];
            next_dst = element.dst.get(i + 1).unwrap_or(prev_dst);
        } else {
            break;
        }
    }

    // If we're past all keyframes
    if elapsed_ms >= element.dst.last().map(|d| d.time).unwrap_or(0) {
        return Some(ResolvedDestination::from_destination(
            element.dst.last()?,
            scale.x,
            scale.y,
        ));
    }

    // Calculate interpolation factor
    let time_range = next_dst.time - prev_dst.time;
    let t = if time_range > 0 {
        let progress = (elapsed_ms - prev_dst.time) as f32 / time_range as f32;
        apply_acceleration(progress, next_dst.acc)
    } else {
        0.0
    };

    let from = ResolvedDestination::from_destination(prev_dst, scale.x, scale.y);
    let to = ResolvedDestination::from_destination(next_dst, scale.x, scale.y);

    Some(ResolvedDestination::interpolate(&from, &to, t))
}

/// Apply acceleration curve
fn apply_acceleration(progress: f32, acc: i32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    match acc {
        0 => progress,                                  // Linear
        1 => progress * progress,                       // Ease in (quadratic)
        2 => 1.0 - (1.0 - progress) * (1.0 - progress), // Ease out
        3 => {
            // Ease in-out
            if progress < 0.5 {
                2.0 * progress * progress
            } else {
                1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
            }
        }
        4 => progress * progress * progress, // Ease in (cubic)
        5 => 1.0 - (1.0 - progress).powi(3), // Ease out (cubic)
        _ => progress,
    }
}

/// Validate skin structure
pub fn validate_skin(skin: &BeatorajaSkin) -> Vec<String> {
    let mut warnings = Vec::new();

    // Check header
    if skin.header.name.is_empty() {
        warnings.push("Skin has no name".to_string());
    }

    if skin.header.w <= 0 || skin.header.h <= 0 {
        warnings.push(format!(
            "Invalid skin dimensions: {}x{}",
            skin.header.w, skin.header.h
        ));
    }

    // Check for orphan image references
    for img in &skin.images {
        if img.id > 0 && skin.get_image(img.id).is_none() {
            warnings.push(format!(
                "Image element references undefined image ID: {}",
                img.id
            ));
        }
    }

    // Check source references in image definitions
    for img_def in &skin.image {
        if skin.get_source(img_def.src).is_none() {
            warnings.push(format!(
                "Image definition {} references undefined source: {}",
                img_def.id, img_def.src
            ));
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_region_frame() {
        let region = ImageRegion {
            x: 0,
            y: 0,
            w: 100,
            h: 100,
        };

        // 2x2 grid
        let frame0 = region.get_frame(0, 2, 2);
        assert_eq!(frame0.x, 0);
        assert_eq!(frame0.y, 0);
        assert_eq!(frame0.w, 50);
        assert_eq!(frame0.h, 50);

        let frame1 = region.get_frame(1, 2, 2);
        assert_eq!(frame1.x, 50);
        assert_eq!(frame1.y, 0);

        let frame2 = region.get_frame(2, 2, 2);
        assert_eq!(frame2.x, 0);
        assert_eq!(frame2.y, 50);
    }

    #[test]
    fn test_image_region_uv() {
        let region = ImageRegion {
            x: 50,
            y: 50,
            w: 100,
            h: 100,
        };

        let (u1, v1, u2, v2) = region.to_uv(200, 200);
        assert!((u1 - 0.25).abs() < 0.001);
        assert!((v1 - 0.25).abs() < 0.001);
        assert!((u2 - 0.75).abs() < 0.001);
        assert!((v2 - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_skin_scale() {
        let scale = SkinScale::new(1280, 720, 1920.0, 1080.0);

        assert!((scale.x - 1.5).abs() < 0.001);
        assert!((scale.y - 1.5).abs() < 0.001);

        let (x, y) = scale.scale_pos(100, 200);
        assert!((x - 150.0).abs() < 0.001);
        assert!((y - 300.0).abs() < 0.001);
    }

    #[test]
    fn test_skin_type_info() {
        let info = SkinTypeInfo::from_type(0);
        assert_eq!(info.lane_count, 8);
        assert!(info.has_scratch);
        assert!(!info.is_double);
        assert!(info.is_play_skin());

        let info = SkinTypeInfo::from_type(5);
        assert_eq!(info.lane_count, 0);
        assert!(!info.is_play_skin());
    }

    #[test]
    fn test_resolved_destination_interpolate() {
        let from = ResolvedDestination {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
            a: 0.0,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            angle: 0.0,
        };

        let to = ResolvedDestination {
            x: 100.0,
            y: 100.0,
            w: 200.0,
            h: 200.0,
            a: 1.0,
            r: 0.0,
            g: 0.0,
            b: 0.0,
            angle: 90.0,
        };

        let mid = ResolvedDestination::interpolate(&from, &to, 0.5);
        assert!((mid.x - 50.0).abs() < 0.001);
        assert!((mid.y - 50.0).abs() < 0.001);
        assert!((mid.w - 150.0).abs() < 0.001);
        assert!((mid.a - 0.5).abs() < 0.001);
        assert!((mid.angle - 45.0).abs() < 0.001);
    }
}
