//! Image region handling for sprite sheets

/// Represents a region within a texture
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImageRegion {
    /// X position in source texture
    pub x: u32,
    /// Y position in source texture
    pub y: u32,
    /// Region width
    pub width: u32,
    /// Region height
    pub height: u32,
}

impl ImageRegion {
    /// Create a new image region
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a full texture region
    pub fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// Get a sub-region for a specific frame in a sprite sheet
    ///
    /// # Arguments
    /// * `frame` - Frame index (0-based)
    /// * `divx` - Number of horizontal divisions
    /// * `divy` - Number of vertical divisions
    pub fn get_frame(&self, frame: u32, divx: u32, divy: u32) -> Self {
        if divx == 0 || divy == 0 {
            return *self;
        }

        let frame_width = self.width / divx;
        let frame_height = self.height / divy;
        let total_frames = divx * divy;

        // Wrap frame number
        let frame = frame % total_frames;

        let frame_x = frame % divx;
        let frame_y = frame / divx;

        Self {
            x: self.x + frame_x * frame_width,
            y: self.y + frame_y * frame_height,
            width: frame_width,
            height: frame_height,
        }
    }

    /// Calculate UV coordinates for rendering
    ///
    /// # Arguments
    /// * `texture_width` - Total texture width
    /// * `texture_height` - Total texture height
    ///
    /// # Returns
    /// (u1, v1, u2, v2) - UV coordinates normalized to 0.0-1.0
    pub fn to_uv(self, texture_width: u32, texture_height: u32) -> (f32, f32, f32, f32) {
        if texture_width == 0 || texture_height == 0 {
            return (0.0, 0.0, 1.0, 1.0);
        }

        let u1 = self.x as f32 / texture_width as f32;
        let v1 = self.y as f32 / texture_height as f32;
        let u2 = (self.x + self.width) as f32 / texture_width as f32;
        let v2 = (self.y + self.height) as f32 / texture_height as f32;

        (u1, v1, u2, v2)
    }

    /// Check if region is empty
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get region area in pixels
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// Create a rect suitable for macroquad drawing
    pub fn to_rect(self) -> macroquad::math::Rect {
        macroquad::math::Rect::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

/// Split a region into a grid of sub-regions
pub fn split_region(region: &ImageRegion, cols: u32, rows: u32) -> Vec<ImageRegion> {
    if cols == 0 || rows == 0 {
        return vec![*region];
    }

    let cell_width = region.width / cols;
    let cell_height = region.height / rows;

    let mut regions = Vec::with_capacity((cols * rows) as usize);

    for row in 0..rows {
        for col in 0..cols {
            regions.push(ImageRegion {
                x: region.x + col * cell_width,
                y: region.y + row * cell_height,
                width: cell_width,
                height: cell_height,
            });
        }
    }

    regions
}

/// Calculate frame count from divx/divy
pub fn frame_count(divx: u32, divy: u32) -> u32 {
    divx.max(1) * divy.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_region_new() {
        let region = ImageRegion::new(10, 20, 100, 50);
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 100);
        assert_eq!(region.height, 50);
    }

    #[test]
    fn test_image_region_full() {
        let region = ImageRegion::full(200, 150);
        assert_eq!(region.x, 0);
        assert_eq!(region.y, 0);
        assert_eq!(region.width, 200);
        assert_eq!(region.height, 150);
    }

    #[test]
    fn test_get_frame_2x2() {
        let region = ImageRegion::new(0, 0, 100, 100);

        let frame0 = region.get_frame(0, 2, 2);
        assert_eq!(frame0.x, 0);
        assert_eq!(frame0.y, 0);
        assert_eq!(frame0.width, 50);
        assert_eq!(frame0.height, 50);

        let frame1 = region.get_frame(1, 2, 2);
        assert_eq!(frame1.x, 50);
        assert_eq!(frame1.y, 0);

        let frame2 = region.get_frame(2, 2, 2);
        assert_eq!(frame2.x, 0);
        assert_eq!(frame2.y, 50);

        let frame3 = region.get_frame(3, 2, 2);
        assert_eq!(frame3.x, 50);
        assert_eq!(frame3.y, 50);
    }

    #[test]
    fn test_get_frame_wrap() {
        let region = ImageRegion::new(0, 0, 100, 100);

        // Frame 4 should wrap to frame 0
        let frame4 = region.get_frame(4, 2, 2);
        let frame0 = region.get_frame(0, 2, 2);
        assert_eq!(frame4, frame0);
    }

    #[test]
    fn test_to_uv() {
        let region = ImageRegion::new(50, 50, 100, 100);
        let (u1, v1, u2, v2) = region.to_uv(200, 200);

        assert!((u1 - 0.25).abs() < 0.001);
        assert!((v1 - 0.25).abs() < 0.001);
        assert!((u2 - 0.75).abs() < 0.001);
        assert!((v2 - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_to_uv_zero_size() {
        let region = ImageRegion::new(0, 0, 100, 100);
        let (u1, v1, u2, v2) = region.to_uv(0, 0);

        // Should return default values
        assert_eq!(u1, 0.0);
        assert_eq!(v1, 0.0);
        assert_eq!(u2, 1.0);
        assert_eq!(v2, 1.0);
    }

    #[test]
    fn test_is_empty() {
        let empty1 = ImageRegion::new(0, 0, 0, 100);
        let empty2 = ImageRegion::new(0, 0, 100, 0);
        let not_empty = ImageRegion::new(0, 0, 100, 100);

        assert!(empty1.is_empty());
        assert!(empty2.is_empty());
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_area() {
        let region = ImageRegion::new(10, 20, 50, 30);
        assert_eq!(region.area(), 1500);
    }

    #[test]
    fn test_split_region() {
        let region = ImageRegion::new(0, 0, 100, 100);
        let parts = split_region(&region, 2, 2);

        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], ImageRegion::new(0, 0, 50, 50));
        assert_eq!(parts[1], ImageRegion::new(50, 0, 50, 50));
        assert_eq!(parts[2], ImageRegion::new(0, 50, 50, 50));
        assert_eq!(parts[3], ImageRegion::new(50, 50, 50, 50));
    }

    #[test]
    fn test_frame_count() {
        assert_eq!(frame_count(4, 4), 16);
        assert_eq!(frame_count(10, 1), 10);
        assert_eq!(frame_count(0, 5), 5); // 0 treated as 1
        assert_eq!(frame_count(3, 0), 3);
    }
}
