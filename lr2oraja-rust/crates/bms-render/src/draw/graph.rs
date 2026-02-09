use bms_skin::skin_graph::GraphDirection;
use bms_skin::skin_object::Rect;

/// Result of computing a graph draw operation.
#[derive(Debug, Clone)]
pub struct GraphDrawCommand {
    /// Source rect (portion of the source image to show).
    pub src_rect: Rect,
    /// Destination rect (where to draw on screen).
    pub dst_rect: Rect,
}

/// Computes the clipped source and destination rects for a graph bar.
///
/// - `direction`: growth direction of the bar
/// - `value`: current value (0.0 to 1.0)
/// - `src`: full source image rect
/// - `dst`: full destination rect from interpolation
pub fn compute_graph_draw(
    direction: GraphDirection,
    value: f32,
    src: &Rect,
    dst: &Rect,
) -> GraphDrawCommand {
    let v = value.clamp(0.0, 1.0);
    match direction {
        GraphDirection::Right => GraphDrawCommand {
            src_rect: Rect::new(src.x, src.y, src.w * v, src.h),
            dst_rect: Rect::new(dst.x, dst.y, dst.w * v, dst.h),
        },
        GraphDirection::Up => {
            let clipped_h = src.h * v;
            let src_y_offset = src.h - clipped_h;
            let dst_y_offset = dst.h - dst.h * v;
            GraphDrawCommand {
                src_rect: Rect::new(src.x, src.y + src_y_offset, src.w, clipped_h),
                dst_rect: Rect::new(dst.x, dst.y + dst_y_offset, dst.w, dst.h * v),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_direction_half() {
        let src = Rect::new(0.0, 0.0, 200.0, 20.0);
        let dst = Rect::new(10.0, 10.0, 300.0, 30.0);
        let cmd = compute_graph_draw(GraphDirection::Right, 0.5, &src, &dst);
        assert!((cmd.src_rect.w - 100.0).abs() < 0.001);
        assert!((cmd.dst_rect.w - 150.0).abs() < 0.001);
        assert!((cmd.src_rect.h - 20.0).abs() < 0.001);
        assert!((cmd.dst_rect.h - 30.0).abs() < 0.001);
    }

    #[test]
    fn right_direction_full() {
        let src = Rect::new(0.0, 0.0, 200.0, 20.0);
        let dst = Rect::new(0.0, 0.0, 300.0, 30.0);
        let cmd = compute_graph_draw(GraphDirection::Right, 1.0, &src, &dst);
        assert!((cmd.src_rect.w - 200.0).abs() < 0.001);
        assert!((cmd.dst_rect.w - 300.0).abs() < 0.001);
    }

    #[test]
    fn up_direction_half() {
        let src = Rect::new(0.0, 0.0, 20.0, 200.0);
        let dst = Rect::new(10.0, 10.0, 30.0, 300.0);
        let cmd = compute_graph_draw(GraphDirection::Up, 0.5, &src, &dst);
        // Source: bottom half of image
        assert!((cmd.src_rect.y - 100.0).abs() < 0.001);
        assert!((cmd.src_rect.h - 100.0).abs() < 0.001);
        // Dest: bottom half of area
        assert!((cmd.dst_rect.y - 160.0).abs() < 0.001);
        assert!((cmd.dst_rect.h - 150.0).abs() < 0.001);
    }

    #[test]
    fn up_direction_zero() {
        let src = Rect::new(0.0, 0.0, 20.0, 200.0);
        let dst = Rect::new(0.0, 0.0, 30.0, 300.0);
        let cmd = compute_graph_draw(GraphDirection::Up, 0.0, &src, &dst);
        assert!((cmd.src_rect.h - 0.0).abs() < 0.001);
        assert!((cmd.dst_rect.h - 0.0).abs() < 0.001);
    }
}
