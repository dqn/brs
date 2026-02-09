// Blend mode helper functions for skin rendering.
//
// The skin system stores blend mode as i32 on SkinObjectBase:
//   0 = Alpha (normal): SrcAlpha, OneMinusSrcAlpha
//   2 = Additive (glow): SrcAlpha, One
//   9 = Invert: OneMinusDstColor, Zero

use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};

/// Blend mode constants matching skin system values.
pub const BLEND_ALPHA: i32 = 0;
pub const BLEND_ADDITIVE: i32 = 2;
pub const BLEND_INVERT: i32 = 9;

/// Returns the BlendState for standard alpha blending.
pub fn alpha_blend_state() -> BlendState {
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
    }
}

/// Returns the BlendState for additive blending (glow effects).
pub fn additive_blend_state() -> BlendState {
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
    }
}

/// Returns the BlendState for invert blending.
pub fn invert_blend_state() -> BlendState {
    BlendState {
        color: BlendComponent {
            src_factor: BlendFactor::OneMinusDst,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            operation: BlendOperation::Add,
        },
    }
}

/// Returns the appropriate BlendState for a skin blend mode value.
/// Falls back to alpha blend for unknown values.
pub fn blend_state_for_mode(blend: i32) -> BlendState {
    match blend {
        BLEND_ADDITIVE => additive_blend_state(),
        BLEND_INVERT => invert_blend_state(),
        _ => alpha_blend_state(),
    }
}

/// Returns true if this blend mode requires a custom material (not standard Sprite).
pub fn needs_custom_material(blend: i32) -> bool {
    blend == BLEND_ADDITIVE || blend == BLEND_INVERT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_blend_state() {
        let state = alpha_blend_state();
        assert_eq!(state.color.src_factor, BlendFactor::SrcAlpha);
        assert_eq!(state.color.dst_factor, BlendFactor::OneMinusSrcAlpha);
        assert_eq!(state.color.operation, BlendOperation::Add);
        assert_eq!(state.alpha.src_factor, BlendFactor::One);
        assert_eq!(state.alpha.dst_factor, BlendFactor::OneMinusSrcAlpha);
        assert_eq!(state.alpha.operation, BlendOperation::Add);
    }

    #[test]
    fn test_additive_blend_state() {
        let state = additive_blend_state();
        assert_eq!(state.color.src_factor, BlendFactor::SrcAlpha);
        assert_eq!(state.color.dst_factor, BlendFactor::One);
        assert_eq!(state.color.operation, BlendOperation::Add);
        assert_eq!(state.alpha.src_factor, BlendFactor::One);
        assert_eq!(state.alpha.dst_factor, BlendFactor::One);
        assert_eq!(state.alpha.operation, BlendOperation::Add);
    }

    #[test]
    fn test_invert_blend_state() {
        let state = invert_blend_state();
        assert_eq!(state.color.src_factor, BlendFactor::OneMinusDst);
        assert_eq!(state.color.dst_factor, BlendFactor::Zero);
        assert_eq!(state.color.operation, BlendOperation::Add);
        assert_eq!(state.alpha.src_factor, BlendFactor::One);
        assert_eq!(state.alpha.dst_factor, BlendFactor::Zero);
        assert_eq!(state.alpha.operation, BlendOperation::Add);
    }

    #[test]
    fn test_blend_state_for_mode() {
        // Alpha (0 and unknown values)
        let alpha = blend_state_for_mode(BLEND_ALPHA);
        assert_eq!(alpha, alpha_blend_state());

        // Additive (2)
        let additive = blend_state_for_mode(BLEND_ADDITIVE);
        assert_eq!(additive, additive_blend_state());

        // Invert (9)
        let invert = blend_state_for_mode(BLEND_INVERT);
        assert_eq!(invert, invert_blend_state());

        // Unknown values fall back to alpha
        assert_eq!(blend_state_for_mode(1), alpha_blend_state());
        assert_eq!(blend_state_for_mode(99), alpha_blend_state());
        assert_eq!(blend_state_for_mode(-1), alpha_blend_state());
    }

    #[test]
    fn test_needs_custom_material() {
        assert!(!needs_custom_material(BLEND_ALPHA));
        assert!(needs_custom_material(BLEND_ADDITIVE));
        assert!(needs_custom_material(BLEND_INVERT));
        assert!(!needs_custom_material(1));
        assert!(!needs_custom_material(99));
        assert!(!needs_custom_material(-1));
    }
}
