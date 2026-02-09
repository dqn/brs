use bms_skin::image_handle::ImageHandle;
use bms_skin::skin_object::Rect;
use bms_skin::skin_source::image_index;
use bms_skin::stretch_type::StretchType;

/// Result of computing an image draw operation.
#[derive(Debug, Clone)]
pub struct ImageDrawCommand {
    pub image_handle: ImageHandle,
    pub src_rect: Option<Rect>,
    pub dst_rect: Rect,
}

/// Computes the draw command for a SkinImage.
///
/// - `source_images`: animation frames for the selected source
/// - `timer_time`: elapsed time for the source animation timer
/// - `cycle`: animation cycle in ms
/// - `stretch`: stretch type
/// - `dst`: destination rect from interpolation
/// - `img_w`, `img_h`: image dimensions (None if unknown)
pub fn compute_image_draw(
    source_images: &[ImageHandle],
    timer_time: i64,
    cycle: i32,
    stretch: StretchType,
    dst: &Rect,
    img_w: Option<f32>,
    img_h: Option<f32>,
) -> Option<ImageDrawCommand> {
    if source_images.is_empty() {
        return None;
    }
    let idx = image_index(source_images.len(), timer_time, cycle);
    let handle = source_images[idx];
    if !handle.is_valid() {
        return None;
    }

    let (iw, ih) = (img_w.unwrap_or(dst.w), img_h.unwrap_or(dst.h));
    let (adj_dst, src_region) = stretch.compute((dst.x, dst.y, dst.w, dst.h), iw, ih);

    Some(ImageDrawCommand {
        image_handle: handle,
        src_rect: src_region.map(|(x, y, w, h)| Rect::new(x, y, w, h)),
        dst_rect: Rect::new(adj_dst.0, adj_dst.1, adj_dst.2, adj_dst.3),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_frame_stretch() {
        let handles = vec![ImageHandle(1)];
        let dst = Rect::new(10.0, 20.0, 100.0, 50.0);
        let cmd = compute_image_draw(
            &handles,
            0,
            0,
            StretchType::Stretch,
            &dst,
            Some(200.0),
            Some(100.0),
        );
        let cmd = cmd.unwrap();
        assert_eq!(cmd.image_handle, ImageHandle(1));
        assert!(cmd.src_rect.is_none());
        assert_eq!(cmd.dst_rect, Rect::new(10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn multi_frame_animation_selection() {
        let handles = vec![
            ImageHandle(10),
            ImageHandle(11),
            ImageHandle(12),
            ImageHandle(13),
        ];
        let dst = Rect::new(0.0, 0.0, 64.0, 64.0);
        // cycle=1000ms, time=500ms -> index = (500*4/1000) % 4 = 2
        let cmd = compute_image_draw(&handles, 500, 1000, StretchType::Stretch, &dst, None, None)
            .unwrap();
        assert_eq!(cmd.image_handle, ImageHandle(12));
    }

    #[test]
    fn empty_source_returns_none() {
        let dst = Rect::new(0.0, 0.0, 100.0, 100.0);
        let result = compute_image_draw(&[], 0, 0, StretchType::Stretch, &dst, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn invalid_handle_returns_none() {
        let handles = vec![ImageHandle::NONE];
        let dst = Rect::new(0.0, 0.0, 100.0, 100.0);
        let result = compute_image_draw(&handles, 0, 0, StretchType::Stretch, &dst, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn no_resize_stretch_type() {
        let handles = vec![ImageHandle(5)];
        // 200x100 image into 400x300 destination -> centered at 200x100
        let dst = Rect::new(0.0, 0.0, 400.0, 300.0);
        let cmd = compute_image_draw(
            &handles,
            0,
            0,
            StretchType::NoResize,
            &dst,
            Some(200.0),
            Some(100.0),
        )
        .unwrap();
        assert!(cmd.src_rect.is_none());
        assert!((cmd.dst_rect.w - 200.0).abs() < 0.001);
        assert!((cmd.dst_rect.h - 100.0).abs() < 0.001);
        // Centered
        assert!((cmd.dst_rect.x - 100.0).abs() < 0.001);
        assert!((cmd.dst_rect.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn keep_aspect_ratio_fit_inner() {
        let handles = vec![ImageHandle(7)];
        // 200x100 image into 100x100 destination -> scale by width (0.5)
        let dst = Rect::new(0.0, 0.0, 100.0, 100.0);
        let cmd = compute_image_draw(
            &handles,
            0,
            0,
            StretchType::KeepAspectRatioFitInner,
            &dst,
            Some(200.0),
            Some(100.0),
        )
        .unwrap();
        assert!((cmd.dst_rect.w - 100.0).abs() < 0.001);
        assert!((cmd.dst_rect.h - 50.0).abs() < 0.001);
    }
}
