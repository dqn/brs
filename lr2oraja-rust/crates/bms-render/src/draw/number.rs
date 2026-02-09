use bms_skin::skin_number::{NumberAlign, ZeroPadding};
use bms_skin::skin_object::Rect;

/// A single digit to draw.
#[derive(Debug, Clone)]
pub struct DigitDrawCommand {
    /// Index into the source image set (0-9 for digits, 10 for space, 11 for minus).
    pub source_index: i32,
    /// Destination position for this digit.
    pub dst_rect: Rect,
}

/// Configuration for number rendering.
#[derive(Debug, Clone, Copy)]
pub struct NumberConfig {
    pub keta: i32,
    pub zero_padding: ZeroPadding,
    pub align: NumberAlign,
    pub space: i32,
    pub digit_w: f32,
    pub negative: bool,
}

/// Decomposes a number into digit draw commands.
///
/// - `value`: the integer to display
/// - `dst`: base destination rect (for the leftmost digit position)
/// - `config`: number rendering configuration
pub fn compute_number_draw(value: i32, dst: &Rect, config: NumberConfig) -> Vec<DigitDrawCommand> {
    let NumberConfig {
        keta,
        zero_padding,
        align,
        space,
        digit_w,
        negative,
    } = config;
    if keta <= 0 || digit_w <= 0.0 {
        return vec![];
    }

    let is_negative = value < 0 && negative;
    let abs_value = value.unsigned_abs();

    // Decompose into digits (rightmost first, then reverse)
    let mut digits: Vec<i32> = Vec::new();
    let mut remaining = abs_value;
    if remaining == 0 {
        digits.push(0);
    } else {
        while remaining > 0 {
            digits.push((remaining % 10) as i32);
            remaining /= 10;
        }
    }
    digits.reverse();

    let keta = keta as usize;

    // Build the display array with padding
    let mut display: Vec<Option<i32>> = Vec::with_capacity(keta);
    let num_digits = digits.len();

    if num_digits + if is_negative { 1 } else { 0 } >= keta {
        // Truncate: show rightmost keta digits (no room for minus)
        for &d in &digits[digits.len().saturating_sub(keta)..] {
            display.push(Some(d));
        }
    } else {
        // Pad leading positions
        let content_len = num_digits + if is_negative { 1 } else { 0 };
        let pad_count = keta - content_len;

        for _ in 0..pad_count {
            match zero_padding {
                ZeroPadding::None => display.push(None),
                ZeroPadding::Zero => display.push(Some(0)),
                ZeroPadding::Space => display.push(Some(10)),
            }
        }

        if is_negative {
            display.push(Some(11)); // minus sign
        }

        for &d in &digits {
            display.push(Some(d));
        }
    }

    // Count visible digits for alignment
    let visible_count = display.iter().filter(|d| d.is_some()).count();
    let total_width = digit_w * keta as f32 + space as f32 * (keta as f32 - 1.0);
    let visible_width =
        digit_w * visible_count as f32 + space as f32 * visible_count.saturating_sub(1) as f32;

    // Calculate starting x based on alignment
    let start_x = match align {
        NumberAlign::Right => dst.x,
        NumberAlign::Left => dst.x,
        NumberAlign::Center => dst.x + (total_width - visible_width) * 0.5,
    };

    let mut commands = Vec::new();

    match align {
        NumberAlign::Right => {
            // Right-aligned: positions are fixed, empty slots show nothing
            for (i, &entry) in display.iter().enumerate() {
                let pos_x = dst.x + (digit_w + space as f32) * i as f32;
                if let Some(source_index) = entry {
                    commands.push(DigitDrawCommand {
                        source_index,
                        dst_rect: Rect::new(pos_x, dst.y, digit_w, dst.h),
                    });
                }
            }
        }
        NumberAlign::Left | NumberAlign::Center => {
            // Left/Center: skip None entries, pack visible digits
            let mut x = start_x;
            for &entry in &display {
                if let Some(source_index) = entry {
                    commands.push(DigitDrawCommand {
                        source_index,
                        dst_rect: Rect::new(x, dst.y, digit_w, dst.h),
                    });
                    x += digit_w + space as f32;
                }
            }
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dst() -> Rect {
        Rect::new(0.0, 0.0, 200.0, 32.0)
    }

    #[test]
    fn single_digit_value_5() {
        let cmds = compute_number_draw(
            5,
            &dst(),
            NumberConfig {
                keta: 1,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].source_index, 5);
    }

    #[test]
    fn multi_digit_right_aligned_no_padding() {
        // value=123, keta=5, no padding, right aligned
        let cmds = compute_number_draw(
            123,
            &dst(),
            NumberConfig {
                keta: 5,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        // 2 hidden + 3 visible digits
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].source_index, 1);
        assert_eq!(cmds[1].source_index, 2);
        assert_eq!(cmds[2].source_index, 3);
        // Right-aligned: digits at positions 2,3,4 (0-indexed)
        assert!((cmds[0].dst_rect.x - 40.0).abs() < 0.001);
        assert!((cmds[1].dst_rect.x - 60.0).abs() < 0.001);
        assert!((cmds[2].dst_rect.x - 80.0).abs() < 0.001);
    }

    #[test]
    fn zero_padding_mode() {
        let cmds = compute_number_draw(
            42,
            &dst(),
            NumberConfig {
                keta: 5,
                zero_padding: ZeroPadding::Zero,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 5);
        assert_eq!(cmds[0].source_index, 0); // leading zero
        assert_eq!(cmds[1].source_index, 0); // leading zero
        assert_eq!(cmds[2].source_index, 0); // leading zero
        assert_eq!(cmds[3].source_index, 4);
        assert_eq!(cmds[4].source_index, 2);
    }

    #[test]
    fn space_padding_mode() {
        let cmds = compute_number_draw(
            7,
            &dst(),
            NumberConfig {
                keta: 3,
                zero_padding: ZeroPadding::Space,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].source_index, 10); // space glyph
        assert_eq!(cmds[1].source_index, 10); // space glyph
        assert_eq!(cmds[2].source_index, 7);
    }

    #[test]
    fn negative_value_with_minus() {
        let cmds = compute_number_draw(
            -42,
            &dst(),
            NumberConfig {
                keta: 5,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: true,
            },
        );
        // 2 hidden + minus + 4 + 2 = pattern: None, None, 11, 4, 2
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].source_index, 11); // minus
        assert_eq!(cmds[1].source_index, 4);
        assert_eq!(cmds[2].source_index, 2);
    }

    #[test]
    fn left_alignment_packs_digits() {
        let cmds = compute_number_draw(
            12,
            &dst(),
            NumberConfig {
                keta: 5,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Left,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 2);
        // Left aligned: packed from left edge
        assert!((cmds[0].dst_rect.x - 0.0).abs() < 0.001);
        assert!((cmds[1].dst_rect.x - 20.0).abs() < 0.001);
    }

    #[test]
    fn center_alignment_centers_digits() {
        let cmds = compute_number_draw(
            5,
            &dst(),
            NumberConfig {
                keta: 3,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Center,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 1);
        // total_width = 20*3 = 60, visible_width = 20*1 = 20
        // start_x = 0 + (60 - 20) * 0.5 = 20
        assert!((cmds[0].dst_rect.x - 20.0).abs() < 0.001);
    }

    #[test]
    fn value_zero() {
        let cmds = compute_number_draw(
            0,
            &dst(),
            NumberConfig {
                keta: 3,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].source_index, 0);
    }

    #[test]
    fn keta_1_truncation() {
        // keta=1, value=42 -> show only rightmost digit (2)
        let cmds = compute_number_draw(
            42,
            &dst(),
            NumberConfig {
                keta: 1,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].source_index, 2);
    }

    #[test]
    fn large_number_exceeding_keta() {
        // value=99999, keta=3 -> show 999
        let cmds = compute_number_draw(
            99999,
            &dst(),
            NumberConfig {
                keta: 3,
                zero_padding: ZeroPadding::None,
                align: NumberAlign::Right,
                space: 0,
                digit_w: 20.0,
                negative: false,
            },
        );
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].source_index, 9);
        assert_eq!(cmds[1].source_index, 9);
        assert_eq!(cmds[2].source_index, 9);
    }
}
