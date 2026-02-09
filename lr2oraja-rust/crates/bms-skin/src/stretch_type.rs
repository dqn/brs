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
}
