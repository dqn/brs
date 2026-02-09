use bms_skin::skin_slider::SliderDirection;

/// Computes the position offset for a slider based on direction and value.
///
/// - `direction`: which direction the slider moves
/// - `range`: maximum pixel displacement
/// - `value`: current value (0.0 to 1.0)
///
/// Returns (offset_x, offset_y) to add to the base position.
pub fn compute_slider_offset(direction: SliderDirection, range: i32, value: f32) -> (f32, f32) {
    let v = value.clamp(0.0, 1.0);
    let offset = range as f32 * v;
    match direction {
        SliderDirection::Up => (0.0, -offset),
        SliderDirection::Down => (0.0, offset),
        SliderDirection::Right => (offset, 0.0),
        SliderDirection::Left => (-offset, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_direction() {
        let (dx, dy) = compute_slider_offset(SliderDirection::Up, 200, 0.5);
        assert!((dx - 0.0).abs() < 0.001);
        assert!((dy - (-100.0)).abs() < 0.001);
    }

    #[test]
    fn right_direction() {
        let (dx, dy) = compute_slider_offset(SliderDirection::Right, 300, 1.0);
        assert!((dx - 300.0).abs() < 0.001);
        assert!((dy - 0.0).abs() < 0.001);
    }

    #[test]
    fn down_direction() {
        let (dx, dy) = compute_slider_offset(SliderDirection::Down, 100, 0.25);
        assert!((dx - 0.0).abs() < 0.001);
        assert!((dy - 25.0).abs() < 0.001);
    }

    #[test]
    fn left_direction() {
        let (dx, dy) = compute_slider_offset(SliderDirection::Left, 400, 0.75);
        assert!((dx - (-300.0)).abs() < 0.001);
        assert!((dy - 0.0).abs() < 0.001);
    }
}
