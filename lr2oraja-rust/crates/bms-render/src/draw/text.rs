use bms_skin::skin_object::Rect;
use bms_skin::skin_text::TextAlign;

/// Computes the text draw position based on alignment.
///
/// Returns the anchor position (x, y) where the text should start.
pub fn compute_text_position(align: TextAlign, dst: &Rect) -> (f32, f32) {
    match align {
        TextAlign::Left => (dst.x, dst.y),
        TextAlign::Center => (dst.x + dst.w * 0.5, dst.y),
        TextAlign::Right => (dst.x + dst.w, dst.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_alignment() {
        let dst = Rect::new(100.0, 50.0, 200.0, 30.0);
        let (x, y) = compute_text_position(TextAlign::Left, &dst);
        assert!((x - 100.0).abs() < 0.001);
        assert!((y - 50.0).abs() < 0.001);
    }

    #[test]
    fn center_alignment() {
        let dst = Rect::new(100.0, 50.0, 200.0, 30.0);
        let (x, y) = compute_text_position(TextAlign::Center, &dst);
        assert!((x - 200.0).abs() < 0.001); // 100 + 200*0.5
        assert!((y - 50.0).abs() < 0.001);
    }

    #[test]
    fn right_alignment() {
        let dst = Rect::new(100.0, 50.0, 200.0, 30.0);
        let (x, y) = compute_text_position(TextAlign::Right, &dst);
        assert!((x - 300.0).abs() < 0.001); // 100 + 200
        assert!((y - 50.0).abs() < 0.001);
    }
}
