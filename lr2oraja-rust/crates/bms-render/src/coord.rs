// Coordinate conversion from skin space to Bevy space.
//
// Skin coordinate system: origin at top-left, Y increases downward.
// Objects are positioned by their top-left corner.
//
// Bevy coordinate system: origin at center, Y increases upward.
// Sprites are positioned by their center.

use bevy::prelude::*;

/// Skin object rectangle.
#[derive(Debug, Clone, Copy)]
pub struct SkinRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Screen dimensions.
#[derive(Debug, Clone, Copy)]
pub struct ScreenSize {
    pub w: f32,
    pub h: f32,
}

/// Rotation parameters.
#[derive(Debug, Clone, Copy)]
pub struct RotationParams {
    pub angle_deg: i32,
    pub center_x: f32,
    pub center_y: f32,
}

/// Convert skin coordinates (top-left origin, Y-down) to Bevy position (center origin, Y-up).
/// Skin objects are positioned by their top-left corner; Bevy sprites use center anchor.
pub fn skin_to_bevy_position(x: f32, y: f32, w: f32, h: f32, screen_w: f32, screen_h: f32) -> Vec2 {
    let bevy_x = (x + w * 0.5) - screen_w * 0.5;
    let bevy_y = screen_h * 0.5 - (y + h * 0.5);
    Vec2::new(bevy_x, bevy_y)
}

/// Convert skin coordinates to a full Bevy Transform with z-order and rotation.
/// `angle_deg` is rotation in degrees (clockwise in skin space, counter-clockwise in Bevy).
/// `center_x`/`center_y` are 0.0–1.0 relative to the object size (0.5, 0.5 = center).
pub fn skin_to_bevy_transform(
    rect: SkinRect,
    screen: ScreenSize,
    z_order: f32,
    rotation: RotationParams,
) -> Transform {
    let SkinRect { x, y, w, h } = rect;
    let ScreenSize {
        w: screen_w,
        h: screen_h,
    } = screen;
    let RotationParams {
        angle_deg,
        center_x,
        center_y,
    } = rotation;
    // Position: convert skin top-left to Bevy center
    let pos = skin_to_bevy_position(x, y, w, h, screen_w, screen_h);

    // Rotation: skin uses clockwise degrees, Bevy uses counter-clockwise radians
    let angle_rad = -(angle_deg as f32).to_radians();
    let rotation = Quat::from_rotation_z(angle_rad);

    // If rotation center is not (0.5, 0.5), adjust position so rotation
    // happens around the correct point.
    // Offset from sprite center to rotation center (in skin pixels):
    let offset_x = (center_x - 0.5) * w;
    let offset_y = -(center_y - 0.5) * h; // flip Y

    // If there's rotation and the center isn't at (0.5, 0.5),
    // rotate the offset and adjust position
    let final_pos = if angle_deg != 0 && (center_x != 0.5 || center_y != 0.5) {
        let rotated_offset = rotation
            .mul_vec3(Vec3::new(offset_x, offset_y, 0.0))
            .truncate();
        Vec2::new(
            pos.x + offset_x - rotated_offset.x,
            pos.y + offset_y - rotated_offset.y,
        )
    } else {
        pos
    };

    Transform {
        translation: Vec3::new(final_pos.x, final_pos.y, z_order),
        rotation,
        scale: Vec3::ONE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const SCREEN_W: f32 = 1280.0;
    const SCREEN_H: f32 = 720.0;

    fn approx_eq_vec2(a: Vec2, b: Vec2, eps: f32) -> bool {
        (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps
    }

    fn approx_eq_f32(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_top_left_corner() {
        // Skin (0,0) with 100x100 on 1280x720
        // bevy_x = (0 + 50) - 640 = -590
        // bevy_y = 360 - (0 + 50) = 310
        let pos = skin_to_bevy_position(0.0, 0.0, 100.0, 100.0, SCREEN_W, SCREEN_H);
        assert!(
            approx_eq_vec2(pos, Vec2::new(-590.0, 310.0), 0.01),
            "expected (-590, 310), got ({}, {})",
            pos.x,
            pos.y,
        );
    }

    #[test]
    fn test_center_of_screen() {
        // Object centered on screen: x=540, y=260, w=200, h=200
        // bevy_x = (540 + 100) - 640 = 0
        // bevy_y = 360 - (260 + 100) = 0
        let pos = skin_to_bevy_position(540.0, 260.0, 200.0, 200.0, SCREEN_W, SCREEN_H);
        assert!(
            approx_eq_vec2(pos, Vec2::new(0.0, 0.0), 0.01),
            "expected (0, 0), got ({}, {})",
            pos.x,
            pos.y,
        );
    }

    #[test]
    fn test_bottom_right_corner() {
        // Bottom-right: x=1180, y=620, w=100, h=100 on 1280x720
        // bevy_x = (1180 + 50) - 640 = 590
        // bevy_y = 360 - (620 + 50) = -310
        let pos = skin_to_bevy_position(1180.0, 620.0, 100.0, 100.0, SCREEN_W, SCREEN_H);
        assert!(
            approx_eq_vec2(pos, Vec2::new(590.0, -310.0), 0.01),
            "expected (590, -310), got ({}, {})",
            pos.x,
            pos.y,
        );
    }

    #[test]
    fn test_zero_size_rect() {
        // Zero-size rect at (100, 200)
        // bevy_x = (100 + 0) - 640 = -540
        // bevy_y = 360 - (200 + 0) = 160
        let pos = skin_to_bevy_position(100.0, 200.0, 0.0, 0.0, SCREEN_W, SCREEN_H);
        assert!(
            approx_eq_vec2(pos, Vec2::new(-540.0, 160.0), 0.01),
            "expected (-540, 160), got ({}, {})",
            pos.x,
            pos.y,
        );
    }

    #[test]
    fn test_transform_no_rotation() {
        // Transform with 0 rotation at (0,0), 100x100
        let t = skin_to_bevy_transform(
            SkinRect {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
            },
            ScreenSize {
                w: SCREEN_W,
                h: SCREEN_H,
            },
            5.0,
            RotationParams {
                angle_deg: 0,
                center_x: 0.5,
                center_y: 0.5,
            },
        );
        assert!(approx_eq_f32(t.translation.x, -590.0, 0.01));
        assert!(approx_eq_f32(t.translation.y, 310.0, 0.01));
        assert!(approx_eq_f32(t.translation.z, 5.0, 0.01));
        // No rotation: quaternion should be identity
        let angle = t.rotation.to_euler(EulerRot::ZYX).0;
        assert!(approx_eq_f32(angle, 0.0, 0.001));
    }

    #[test]
    fn test_transform_90_degree_center_rotation() {
        // 90 degree clockwise rotation around center (0.5, 0.5)
        let t = skin_to_bevy_transform(
            SkinRect {
                x: 100.0,
                y: 100.0,
                w: 200.0,
                h: 200.0,
            },
            ScreenSize {
                w: SCREEN_W,
                h: SCREEN_H,
            },
            1.0,
            RotationParams {
                angle_deg: 90,
                center_x: 0.5,
                center_y: 0.5,
            },
        );
        // Position should be same as no-rotation case since center_x/y = 0.5
        let pos = skin_to_bevy_position(100.0, 100.0, 200.0, 200.0, SCREEN_W, SCREEN_H);
        assert!(approx_eq_f32(t.translation.x, pos.x, 0.01));
        assert!(approx_eq_f32(t.translation.y, pos.y, 0.01));
        // Rotation: -90 degrees in Bevy = -PI/2
        let angle = t.rotation.to_euler(EulerRot::ZYX).0;
        assert!(
            approx_eq_f32(angle, -FRAC_PI_2, 0.001),
            "expected {}, got {}",
            -FRAC_PI_2,
            angle,
        );
    }

    #[test]
    fn test_transform_rotation_non_center_point() {
        // 90 degree rotation around top-left corner (center_x=0, center_y=0)
        let t = skin_to_bevy_transform(
            SkinRect {
                x: 200.0,
                y: 200.0,
                w: 100.0,
                h: 100.0,
            },
            ScreenSize {
                w: SCREEN_W,
                h: SCREEN_H,
            },
            2.0,
            RotationParams {
                angle_deg: 90,
                center_x: 0.0,
                center_y: 0.0,
            },
        );
        // offset_x = (0.0 - 0.5) * 100 = -50
        // offset_y = -(0.0 - 0.5) * 100 = 50
        // After 90 degree CW (= -90 in Bevy), rotated offset: (50, 50)
        // final_x = pos.x + (-50) - (50) = pos.x - 100
        // final_y = pos.y + (50) - (50) = pos.y
        let pos = skin_to_bevy_position(200.0, 200.0, 100.0, 100.0, SCREEN_W, SCREEN_H);
        assert!(approx_eq_f32(t.translation.x, pos.x - 100.0, 0.5));
        assert!(approx_eq_f32(t.translation.y, pos.y, 0.5));
        assert!(approx_eq_f32(t.translation.z, 2.0, 0.01));
    }

    #[test]
    fn test_non_standard_screen_size() {
        // 640x480 screen, object at (0, 0), 64x48
        // bevy_x = (0 + 32) - 320 = -288
        // bevy_y = 240 - (0 + 24) = 216
        let pos = skin_to_bevy_position(0.0, 0.0, 64.0, 48.0, 640.0, 480.0);
        assert!(
            approx_eq_vec2(pos, Vec2::new(-288.0, 216.0), 0.01),
            "expected (-288, 216), got ({}, {})",
            pos.x,
            pos.y,
        );
    }

    #[test]
    fn test_symmetry_top_left_bottom_right() {
        // Top-left and bottom-right of same-size objects should be symmetric about origin
        let tl = skin_to_bevy_position(0.0, 0.0, 100.0, 100.0, SCREEN_W, SCREEN_H);
        let br = skin_to_bevy_position(1180.0, 620.0, 100.0, 100.0, SCREEN_W, SCREEN_H);
        assert!(approx_eq_f32(tl.x, -br.x, 0.01));
        assert!(approx_eq_f32(tl.y, -br.y, 0.01));
    }

    #[test]
    fn test_transform_180_degree_rotation() {
        // 180 degree rotation around center — position should not change
        let t = skin_to_bevy_transform(
            SkinRect {
                x: 100.0,
                y: 100.0,
                w: 200.0,
                h: 200.0,
            },
            ScreenSize {
                w: SCREEN_W,
                h: SCREEN_H,
            },
            0.0,
            RotationParams {
                angle_deg: 180,
                center_x: 0.5,
                center_y: 0.5,
            },
        );
        let pos = skin_to_bevy_position(100.0, 100.0, 200.0, 200.0, SCREEN_W, SCREEN_H);
        assert!(approx_eq_f32(t.translation.x, pos.x, 0.01));
        assert!(approx_eq_f32(t.translation.y, pos.y, 0.01));
        // Rotation: -180 degrees = -PI
        let angle = t.rotation.to_euler(EulerRot::ZYX).0;
        assert!(
            approx_eq_f32(angle.abs(), std::f32::consts::PI, 0.001),
            "expected PI, got {}",
            angle,
        );
    }
}
