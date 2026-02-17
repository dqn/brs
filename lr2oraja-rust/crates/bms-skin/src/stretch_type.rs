// Image stretch types ported from StretchType.java.

use serde::{Deserialize, Serialize};

/// Type alias for destination rectangle (x, y, w, h).
type Rect = (f32, f32, f32, f32);

/// How an image is stretched within its destination rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum StretchType {
    /// Stretch to fill the destination (default).
    #[default]
    Stretch = 0,
    /// Keep aspect ratio, fit inside the destination.
    KeepAspectRatioFitInner = 1,
    /// Keep aspect ratio, cover the entire destination.
    KeepAspectRatioFitOuter = 2,
    /// Keep aspect ratio, cover the entire destination (trimmed).
    KeepAspectRatioFitOuterTrimmed = 3,
    /// Keep aspect ratio, fit to destination width.
    KeepAspectRatioFitWidth = 4,
    /// Keep aspect ratio, fit to destination width (trimmed).
    KeepAspectRatioFitWidthTrimmed = 5,
    /// Keep aspect ratio, fit to destination height.
    KeepAspectRatioFitHeight = 6,
    /// Keep aspect ratio, fit to destination height (trimmed).
    KeepAspectRatioFitHeightTrimmed = 7,
    /// Keep aspect ratio, no expanding (only shrink if needed).
    KeepAspectRatioNoExpanding = 8,
    /// No resize, center in destination.
    NoResize = 9,
    /// No resize, center with trimming.
    NoResizeTrimmed = 10,
}

impl StretchType {
    /// Returns the numeric ID.
    pub fn id(self) -> i32 {
        self as i32
    }

    /// Looks up a stretch type by numeric ID.
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Stretch),
            1 => Some(Self::KeepAspectRatioFitInner),
            2 => Some(Self::KeepAspectRatioFitOuter),
            3 => Some(Self::KeepAspectRatioFitOuterTrimmed),
            4 => Some(Self::KeepAspectRatioFitWidth),
            5 => Some(Self::KeepAspectRatioFitWidthTrimmed),
            6 => Some(Self::KeepAspectRatioFitHeight),
            7 => Some(Self::KeepAspectRatioFitHeightTrimmed),
            8 => Some(Self::KeepAspectRatioNoExpanding),
            9 => Some(Self::NoResize),
            10 => Some(Self::NoResizeTrimmed),
            _ => None,
        }
    }

    /// Computes the adjusted destination rectangle and optional source region
    /// for the given image dimensions.
    ///
    /// `dst` is the destination rectangle (x, y, w, h).
    /// `img_w` and `img_h` are the source image dimensions.
    ///
    /// Returns `(adjusted_dst, source_region)` where source_region is
    /// `Some((sx, sy, sw, sh))` if the source image should be trimmed, or
    /// `None` if the full source should be used.
    pub fn compute(self, dst: Rect, img_w: f32, img_h: f32) -> (Rect, Option<Rect>) {
        let (x, y, w, h) = dst;

        match self {
            Self::Stretch => ((x, y, w, h), None),

            Self::KeepAspectRatioFitInner => {
                let scale_x = w / img_w;
                let scale_y = h / img_h;
                let mut r = (x, y, w, h);
                if scale_x <= scale_y {
                    fit_height(&mut r, img_h * scale_x);
                } else {
                    fit_width(&mut r, img_w * scale_y);
                }
                (r, None)
            }

            Self::KeepAspectRatioFitOuter => {
                let scale_x = w / img_w;
                let scale_y = h / img_h;
                let mut r = (x, y, w, h);
                if scale_x >= scale_y {
                    fit_height(&mut r, img_h * scale_x);
                } else {
                    fit_width(&mut r, img_w * scale_y);
                }
                (r, None)
            }

            Self::KeepAspectRatioFitOuterTrimmed => {
                let scale_x = w / img_w;
                let scale_y = h / img_h;
                let mut r = (x, y, w, h);
                let mut src = (0.0, 0.0, img_w, img_h);
                if scale_x >= scale_y {
                    fit_height_trimmed(&mut r, scale_x, &mut src);
                } else {
                    fit_width_trimmed(&mut r, scale_y, &mut src);
                }
                (r, Some(src))
            }

            Self::KeepAspectRatioFitWidth => {
                let mut r = (x, y, w, h);
                fit_height(&mut r, img_h * w / img_w);
                (r, None)
            }

            Self::KeepAspectRatioFitWidthTrimmed => {
                let scale = w / img_w;
                let mut r = (x, y, w, h);
                let mut src = (0.0, 0.0, img_w, img_h);
                fit_height_trimmed(&mut r, scale, &mut src);
                (r, Some(src))
            }

            Self::KeepAspectRatioFitHeight => {
                let mut r = (x, y, w, h);
                fit_width(&mut r, img_w * h / img_h);
                (r, None)
            }

            Self::KeepAspectRatioFitHeightTrimmed => {
                let scale = h / img_h;
                let mut r = (x, y, w, h);
                let mut src = (0.0, 0.0, img_w, img_h);
                fit_width_trimmed(&mut r, scale, &mut src);
                (r, Some(src))
            }

            Self::KeepAspectRatioNoExpanding => {
                let scale = 1.0_f32.min((w / img_w).min(h / img_h));
                let mut r = (x, y, w, h);
                fit_width(&mut r, img_w * scale);
                fit_height(&mut r, img_h * scale);
                (r, None)
            }

            Self::NoResize => {
                let mut r = (x, y, w, h);
                fit_width(&mut r, img_w);
                fit_height(&mut r, img_h);
                (r, None)
            }

            Self::NoResizeTrimmed => {
                let mut r = (x, y, w, h);
                let mut src = (0.0, 0.0, img_w, img_h);
                fit_width_trimmed(&mut r, 1.0, &mut src);
                fit_height_trimmed(&mut r, 1.0, &mut src);
                (r, Some(src))
            }
        }
    }
}

// Center-aligns the rectangle horizontally to a new width.
fn fit_width(rect: &mut (f32, f32, f32, f32), width: f32) {
    let cx = rect.0 + rect.2 * 0.5;
    rect.2 = width;
    rect.0 = cx - width * 0.5;
}

// Center-aligns the rectangle vertically to a new height.
fn fit_height(rect: &mut (f32, f32, f32, f32), height: f32) {
    let cy = rect.1 + rect.3 * 0.5;
    rect.3 = height;
    rect.1 = cy - height * 0.5;
}

// Trims the source image horizontally if scaled size exceeds destination width.
fn fit_width_trimmed(rect: &mut (f32, f32, f32, f32), scale: f32, src: &mut (f32, f32, f32, f32)) {
    let width = scale * src.2;
    if rect.2 < width {
        let cx = src.0 + src.2 * 0.5;
        let w = rect.2 / scale;
        src.0 = cx - w * 0.5;
        src.2 = w;
    } else {
        fit_width(rect, width);
    }
}

// Trims the source image vertically if scaled size exceeds destination height.
fn fit_height_trimmed(rect: &mut (f32, f32, f32, f32), scale: f32, src: &mut (f32, f32, f32, f32)) {
    let height = scale * src.3;
    if rect.3 < height {
        let cy = src.1 + src.3 * 0.5;
        let h = rect.3 / scale;
        src.1 = cy - h * 0.5;
        src.3 = h;
    } else {
        fit_height(rect, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_id_round_trip() {
        for id in 0..=10 {
            let st = StretchType::from_id(id).unwrap();
            assert_eq!(st.id(), id);
        }
        assert!(StretchType::from_id(11).is_none());
        assert!(StretchType::from_id(-1).is_none());
    }

    #[test]
    fn test_stretch_passthrough() {
        let (dst, src) = StretchType::Stretch.compute((10.0, 20.0, 100.0, 50.0), 200.0, 100.0);
        assert_eq!(dst, (10.0, 20.0, 100.0, 50.0));
        assert!(src.is_none());
    }

    #[test]
    fn test_no_resize() {
        // 200x100 image into 400x300 destination -> centered at 200x100
        let (dst, src) = StretchType::NoResize.compute((0.0, 0.0, 400.0, 300.0), 200.0, 100.0);
        assert_eq!(dst.2, 200.0); // width = image width
        assert_eq!(dst.3, 100.0); // height = image height
        // Centered
        assert!((dst.0 - 100.0).abs() < 0.001);
        assert!((dst.1 - 100.0).abs() < 0.001);
        assert!(src.is_none());
    }

    #[test]
    fn test_fit_inner() {
        // 200x100 image into 100x100 destination -> scale by width (0.5)
        let (dst, _) =
            StretchType::KeepAspectRatioFitInner.compute((0.0, 0.0, 100.0, 100.0), 200.0, 100.0);
        assert_eq!(dst.2, 100.0); // width stays
        assert_eq!(dst.3, 50.0); // height = 100 * 0.5
    }

    #[test]
    fn test_fit_outer() {
        // 200x100 image into 100x100 destination -> scale by height (1.0)
        let (dst, _) =
            StretchType::KeepAspectRatioFitOuter.compute((0.0, 0.0, 100.0, 100.0), 200.0, 100.0);
        assert_eq!(dst.2, 200.0); // width = 200 * 1.0
        assert_eq!(dst.3, 100.0); // height stays
    }

    #[test]
    fn test_default() {
        assert_eq!(StretchType::default(), StretchType::Stretch);
    }

    #[test]
    fn test_fit_outer_trimmed() {
        // 200x100 image into 100x100 dst.
        // scale_x=0.5, scale_y=1.0. scale_y > scale_x, so fit_width_trimmed is called.
        // Scaled width = 1.0 * 200 = 200 > dst.w=100, so source is trimmed horizontally.
        let (dst, src) = StretchType::KeepAspectRatioFitOuterTrimmed.compute(
            (0.0, 0.0, 100.0, 100.0),
            200.0,
            100.0,
        );
        let src = src.expect("source region should be trimmed");
        assert!((dst.0 - 0.0).abs() < 0.001);
        assert!((dst.1 - 0.0).abs() < 0.001);
        assert!((dst.2 - 100.0).abs() < 0.001);
        assert!((dst.3 - 100.0).abs() < 0.001);
        // Source trimmed: cx=100, w=100/1.0=100, so src.x=50, src.w=100
        assert!((src.0 - 50.0).abs() < 0.001);
        assert!((src.1 - 0.0).abs() < 0.001);
        assert!((src.2 - 100.0).abs() < 0.001);
        assert!((src.3 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_fit_width_trimmed() {
        // 200x400 image into 100x100 dst. scale=0.5.
        // Scaled height = 0.5 * 400 = 200 > dst.h=100, so source is trimmed vertically.
        let (dst, src) = StretchType::KeepAspectRatioFitWidthTrimmed.compute(
            (0.0, 0.0, 100.0, 100.0),
            200.0,
            400.0,
        );
        let src = src.expect("source region should be trimmed");
        assert!((dst.2 - 100.0).abs() < 0.001);
        assert!((dst.3 - 100.0).abs() < 0.001);
        // Source trimmed vertically: cy=200, h=100/0.5=200, src.y=100, src.h=200
        assert!((src.0 - 0.0).abs() < 0.001);
        assert!((src.1 - 100.0).abs() < 0.001);
        assert!((src.2 - 200.0).abs() < 0.001);
        assert!((src.3 - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_fit_height_trimmed() {
        // 400x200 image into 100x100 dst. scale=100/200=0.5.
        // Scaled width = 0.5 * 400 = 200 > dst.w=100, so source is trimmed horizontally.
        let (dst, src) = StretchType::KeepAspectRatioFitHeightTrimmed.compute(
            (0.0, 0.0, 100.0, 100.0),
            400.0,
            200.0,
        );
        let src = src.expect("source region should be trimmed");
        assert!((dst.2 - 100.0).abs() < 0.001);
        assert!((dst.3 - 100.0).abs() < 0.001);
        // Source trimmed horizontally: cx=200, w=100/0.5=200, src.x=100, src.w=200
        assert!((src.0 - 100.0).abs() < 0.001);
        assert!((src.1 - 0.0).abs() < 0.001);
        assert!((src.2 - 200.0).abs() < 0.001);
        assert!((src.3 - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_no_expanding_small_image() {
        // 50x25 image into 200x200 dst.
        // scale = min(1.0, min(4.0, 8.0)) = 1.0. Image stays 50x25, centered.
        let (dst, src) =
            StretchType::KeepAspectRatioNoExpanding.compute((0.0, 0.0, 200.0, 200.0), 50.0, 25.0);
        assert!(src.is_none());
        assert!((dst.2 - 50.0).abs() < 0.001);
        assert!((dst.3 - 25.0).abs() < 0.001);
        // Centered: x = (200 - 50) / 2 = 75, y = (200 - 25) / 2 = 87.5
        assert!((dst.0 - 75.0).abs() < 0.001);
        assert!((dst.1 - 87.5).abs() < 0.001);
    }

    #[test]
    fn test_no_expanding_large_image() {
        // 400x200 image into 100x100 dst.
        // scale = min(1.0, min(0.25, 0.5)) = 0.25. Image shrinks to 100x50, centered.
        let (dst, src) =
            StretchType::KeepAspectRatioNoExpanding.compute((0.0, 0.0, 100.0, 100.0), 400.0, 200.0);
        assert!(src.is_none());
        assert!((dst.2 - 100.0).abs() < 0.001);
        assert!((dst.3 - 50.0).abs() < 0.001);
        // Centered vertically: y = (100 - 50) / 2 = 25
        assert!((dst.0 - 0.0).abs() < 0.001);
        assert!((dst.1 - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_no_resize_trimmed() {
        // 200x300 image into 100x100 dst. Image > dst on both axes, so both trimmed.
        let (dst, src) =
            StretchType::NoResizeTrimmed.compute((0.0, 0.0, 100.0, 100.0), 200.0, 300.0);
        let src = src.expect("source region should be trimmed");
        assert!((dst.2 - 100.0).abs() < 0.001);
        assert!((dst.3 - 100.0).abs() < 0.001);
        // Width trimmed: cx=100, w=100, src.x=50, src.w=100
        // Height trimmed: cy=150, h=100, src.y=100, src.h=100
        assert!((src.0 - 50.0).abs() < 0.001);
        assert!((src.1 - 100.0).abs() < 0.001);
        assert!((src.2 - 100.0).abs() < 0.001);
        assert!((src.3 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_fit_width() {
        // 200x100 image into 400x400 dst. Width fits to 400, height = 100 * 400/200 = 200.
        // Centered vertically in 400.
        let (dst, src) =
            StretchType::KeepAspectRatioFitWidth.compute((0.0, 0.0, 400.0, 400.0), 200.0, 100.0);
        assert!(src.is_none());
        assert!((dst.2 - 400.0).abs() < 0.001);
        assert!((dst.3 - 200.0).abs() < 0.001);
        // Centered: y = (400 - 200) / 2 = 100
        assert!((dst.0 - 0.0).abs() < 0.001);
        assert!((dst.1 - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_fit_height() {
        // 200x100 image into 400x400 dst. Height fits to 400, width = 200 * 400/100 = 800.
        // Centered horizontally.
        let (dst, src) =
            StretchType::KeepAspectRatioFitHeight.compute((0.0, 0.0, 400.0, 400.0), 200.0, 100.0);
        assert!(src.is_none());
        assert!((dst.2 - 800.0).abs() < 0.001);
        assert!((dst.3 - 400.0).abs() < 0.001);
        // Centered: x = (400 - 800) / 2 = -200
        assert!((dst.0 - (-200.0)).abs() < 0.001);
        assert!((dst.1 - 0.0).abs() < 0.001);
    }
}
