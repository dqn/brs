use macroquad::prelude::*;

use crate::skin::object::{
    SkinObject, apply_offsets, check_option_visibility, get_timer_elapsed, interpolate_destinations,
};
use crate::skin::{MainState, NumberDef, SkinObjectData, SkinSourceManager};

/// Alignment mode for number display.
const ALIGN_LEFT: i32 = 0;
const ALIGN_RIGHT: i32 = 1;
const ALIGN_CENTER: i32 = 2;

/// Skin object that renders digit-based numbers.
pub struct NumberObject {
    /// Object data from skin definition.
    pub data: SkinObjectData,
    /// Number definition.
    pub number_def: Option<NumberDef>,
    /// Whether the object is prepared.
    prepared: bool,
}

impl NumberObject {
    /// Create a new number object.
    pub fn new(data: SkinObjectData, number_def: Option<NumberDef>) -> Self {
        Self {
            data,
            number_def,
            prepared: false,
        }
    }

    /// Get digits from value.
    fn get_digits(&self, value: i32) -> Vec<i32> {
        let Some(ref number_def) = self.number_def else {
            return vec![];
        };

        let digit_count = number_def.digit.max(1) as usize;
        let has_minus = number_def.divx >= 11;
        let has_space = number_def.divx >= 12;

        let is_negative = value < 0;
        let abs_value = value.unsigned_abs();

        let mut digits: Vec<i32> = Vec::with_capacity(digit_count);

        // Extract digits from right to left
        let mut remaining = abs_value;
        for _ in 0..digit_count {
            digits.push((remaining % 10) as i32);
            remaining /= 10;
        }

        // Reverse to get left to right
        digits.reverse();

        // Handle zero padding or space replacement
        if number_def.zeropadding == 0 {
            // Replace leading zeros with spaces (if available) or keep as is
            let mut found_nonzero = false;
            for i in 0..digits.len() {
                if digits[i] != 0 {
                    found_nonzero = true;
                }
                if !found_nonzero && i < digits.len() - 1 {
                    // Leading zero - replace with space if available
                    if has_space {
                        digits[i] = 11; // Space index
                    } else {
                        digits[i] = -1; // Don't draw
                    }
                }
            }
        }

        // Handle negative sign
        if is_negative && has_minus {
            // Find first non-space digit and replace the one before it with minus
            for i in 0..digits.len() {
                if digits[i] != 11 && digits[i] != -1 {
                    if i > 0 {
                        digits[i - 1] = 10; // Minus index
                    }
                    break;
                }
            }
        }

        digits
    }

    /// Draw a single digit.
    #[allow(clippy::too_many_arguments)]
    fn draw_digit(
        &self,
        digit: i32,
        x: f32,
        y: f32,
        digit_w: f32,
        digit_h: f32,
        color: Color,
        texture: &Texture2D,
    ) {
        let Some(ref number_def) = self.number_def else {
            return;
        };

        if digit < 0 {
            return; // Don't draw invisible digits
        }

        let divx = number_def.divx.max(1) as usize;
        let divy = number_def.divy.max(1) as usize;
        let frame_w = number_def.w as f32 / divx as f32;
        let frame_h = number_def.h as f32 / divy as f32;

        // Calculate source position for this digit
        let digit_idx = digit as usize;
        let src_x = (digit_idx % divx) as f32 * frame_w;
        let src_y = (digit_idx / divx) as f32 * frame_h;

        let src_rect = Rect::new(
            number_def.x as f32 + src_x,
            number_def.y as f32 + src_y,
            frame_w,
            frame_h,
        );

        draw_texture_ex(
            texture,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(digit_w, digit_h)),
                source: Some(src_rect),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }
}

impl SkinObject for NumberObject {
    fn prepare(&mut self, _sources: &SkinSourceManager) {
        self.prepared = true;
    }

    fn draw(&self, state: &MainState, sources: &SkinSourceManager, now_time_us: i64) {
        if !self.is_visible(state) {
            return;
        }

        let Some(ref number_def) = self.number_def else {
            return;
        };

        let Some(texture) = sources.get(number_def.src) else {
            return;
        };

        // Calculate elapsed time
        let elapsed_us = get_timer_elapsed(self.data.timer, state, now_time_us);
        if elapsed_us < 0 {
            return; // Timer not active
        }

        // Get interpolated destination
        let elapsed_ms = elapsed_us / 1000;
        let Some(dst) = interpolate_destinations(&self.data.dst, elapsed_ms, self.data.loop_count)
        else {
            return;
        };
        let dst = apply_offsets(dst, &self.data, state);

        // Skip if invisible
        if dst.a <= 0.0 || dst.w <= 0.0 || dst.h <= 0.0 {
            return;
        }

        // Get value from state
        let value = state.number(number_def.ref_id);
        let digits = self.get_digits(value);

        if digits.is_empty() {
            return;
        }

        // Calculate digit dimensions
        let digit_w = dst.w;
        let digit_h = dst.h;
        let spacing = number_def.space as f32;
        let shiftbase = digits.iter().take_while(|digit| **digit < 0).count();
        let shift = match number_def.align {
            ALIGN_LEFT => 0.0,
            ALIGN_RIGHT => (digit_w + spacing) * shiftbase as f32,
            ALIGN_CENTER => (digit_w + spacing) * 0.5 * shiftbase as f32,
            _ => (digit_w + spacing) * 0.5 * shiftbase as f32,
        };
        let start_x = dst.x - shift;

        let color = Color::new(dst.r / 255.0, dst.g / 255.0, dst.b / 255.0, dst.a / 255.0);

        // Draw each digit
        for (i, &digit) in digits.iter().enumerate() {
            let x = start_x + (digit_w + spacing) * i as f32;
            self.draw_digit(digit, x, dst.y, digit_w, digit_h, color, &texture.texture);
        }
    }

    fn is_visible(&self, state: &MainState) -> bool {
        check_option_visibility(&self.data.op, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::Destination;

    #[test]
    fn test_number_object_creation() {
        let data = SkinObjectData {
            id: "test".to_string(),
            dst: vec![Destination {
                x: 100.0,
                y: 100.0,
                w: 24.0,
                h: 32.0,
                ..Default::default()
            }],
            ..Default::default()
        };
        let obj = NumberObject::new(data, None);
        assert!(!obj.prepared);
    }

    #[test]
    fn test_get_digits_positive() {
        let data = SkinObjectData::default();
        let number_def = NumberDef {
            id: "test".to_string(),
            digit: 4,
            divx: 10,
            divy: 1,
            zeropadding: 1, // Enable zero padding
            ..Default::default()
        };
        let obj = NumberObject::new(data, Some(number_def));

        let digits = obj.get_digits(123);
        assert_eq!(digits, vec![0, 1, 2, 3]);

        let digits = obj.get_digits(5);
        assert_eq!(digits, vec![0, 0, 0, 5]);
    }

    #[test]
    fn test_get_digits_without_padding() {
        let data = SkinObjectData::default();
        let number_def = NumberDef {
            id: "test".to_string(),
            digit: 4,
            divx: 12, // Has space character
            divy: 1,
            zeropadding: 0, // No zero padding
            ..Default::default()
        };
        let obj = NumberObject::new(data, Some(number_def));

        let digits = obj.get_digits(42);
        // Leading zeros should be replaced with space (11)
        assert_eq!(digits, vec![11, 11, 4, 2]);
    }

    #[test]
    fn test_get_digits_negative() {
        let data = SkinObjectData::default();
        let number_def = NumberDef {
            id: "test".to_string(),
            digit: 4,
            divx: 11, // Has minus sign
            divy: 1,
            zeropadding: 0,
            ..Default::default()
        };
        let obj = NumberObject::new(data, Some(number_def));

        let digits = obj.get_digits(-42);
        // Should have minus sign (10) before the number
        assert_eq!(digits, vec![-1, 10, 4, 2]);
    }
}
