use crate::traits::render::Color;

/// Convert Color to a [f32; 4] array suitable for vertex data.
pub fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

/// Convert Color to wgpu::Color.
pub fn color_to_wgpu(c: Color) -> wgpu::Color {
    wgpu::Color {
        r: c.r as f64,
        g: c.g as f64,
        b: c.b as f64,
        a: c.a as f64,
    }
}

/// Create orthographic projection matrix for 2D rendering.
/// Top-left origin, Y-down coordinate system (beatoraja compatible).
pub fn ortho_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0],
    ]
}

/// Compute the 4 corner positions of a rotated rectangle.
/// Returns vertices in order: top-left, top-right, bottom-right, bottom-left.
pub fn rotated_quad(x: f32, y: f32, w: f32, h: f32, angle: f32) -> [[f32; 2]; 4] {
    if angle == 0.0 {
        return [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
    }

    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let cos = angle.cos();
    let sin = angle.sin();

    let corners = [
        [-w * 0.5, -h * 0.5],
        [w * 0.5, -h * 0.5],
        [w * 0.5, h * 0.5],
        [-w * 0.5, h * 0.5],
    ];

    corners.map(|[dx, dy]| [cx + dx * cos - dy * sin, cy + dx * sin + dy * cos])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_array() {
        let c = Color::new(0.1, 0.2, 0.3, 0.4);
        let arr = color_to_array(c);
        assert_eq!(arr, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn test_ortho_projection() {
        let proj = ortho_projection(1280.0, 720.0);
        // Top-left corner (0,0) should map to NDC (-1, 1)
        let x = 0.0 * proj[0][0] + 0.0 * proj[1][0] + 0.0 * proj[2][0] + proj[3][0];
        let y = 0.0 * proj[0][1] + 0.0 * proj[1][1] + 0.0 * proj[2][1] + proj[3][1];
        assert!((x - (-1.0)).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);

        // Bottom-right corner (1280,720) should map to NDC (1, -1)
        let x = 1280.0 * proj[0][0] + 720.0 * proj[1][0] + 0.0 * proj[2][0] + proj[3][0];
        let y = 1280.0 * proj[0][1] + 720.0 * proj[1][1] + 0.0 * proj[2][1] + proj[3][1];
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_rotated_quad_no_rotation() {
        let verts = rotated_quad(10.0, 20.0, 100.0, 50.0, 0.0);
        assert_eq!(verts[0], [10.0, 20.0]);
        assert_eq!(verts[1], [110.0, 20.0]);
        assert_eq!(verts[2], [110.0, 70.0]);
        assert_eq!(verts[3], [10.0, 70.0]);
    }

    #[test]
    fn test_rotated_quad_90_degrees() {
        let verts = rotated_quad(0.0, 0.0, 100.0, 100.0, std::f32::consts::FRAC_PI_2);
        // Center is (50, 50), rotating 90 degrees should swap and negate
        let cx = 50.0;
        let cy = 50.0;
        for v in &verts {
            let dx = v[0] - cx;
            let dy = v[1] - cy;
            // Distance from center should be preserved
            let dist = (dx * dx + dy * dy).sqrt();
            assert!((dist - 70.710_678).abs() < 0.01);
        }
    }
}
