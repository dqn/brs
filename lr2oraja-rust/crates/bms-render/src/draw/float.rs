use bms_skin::skin_float::SkinFloat;
use bms_skin::skin_object::Rect;

use super::number::DigitDrawCommand;

/// Decomposes a float value into digit draw commands.
///
/// Splits into integer part, decimal point (source index 10), and fractional part.
/// Uses the same source image indices as SkinNumber: 0-9 = digits, 10 = period, 11 = minus.
pub fn compute_float_draw(
    value: f32,
    rect: &Rect,
    float_obj: &SkinFloat,
    digit_w: f32,
) -> Vec<DigitDrawCommand> {
    if digit_w <= 0.0 {
        return vec![];
    }

    let is_negative = value < 0.0;
    let abs_value = value.abs();

    let iketa = float_obj.iketa.max(0) as usize;
    let fketa = float_obj.fketa.max(0) as usize;

    // Extract integer and fractional parts
    let int_part = abs_value as u64;
    let frac_part = ((abs_value - int_part as f64 as f32) * 10f32.powi(fketa as i32)) as u64;

    // Decompose integer part into digits
    let mut int_digits: Vec<i32> = Vec::new();
    let mut remaining = int_part;
    if remaining == 0 {
        int_digits.push(0);
    } else {
        while remaining > 0 {
            int_digits.push((remaining % 10) as i32);
            remaining /= 10;
        }
    }
    int_digits.reverse();

    // Decompose fractional part into digits
    let mut frac_digits: Vec<i32> = vec![0; fketa];
    let mut frac_remaining = frac_part;
    for i in (0..fketa).rev() {
        frac_digits[i] = (frac_remaining % 10) as i32;
        frac_remaining /= 10;
    }

    let mut commands = Vec::new();
    let mut x = 0.0_f32;

    // Minus sign
    if is_negative && float_obj.sign_visible {
        commands.push(DigitDrawCommand {
            source_index: 11,
            dst_rect: Rect::new(x, 0.0, digit_w, rect.h),
        });
        x += digit_w;
    }

    // Integer digits (pad or truncate to iketa)
    let start = if int_digits.len() > iketa {
        int_digits.len() - iketa
    } else {
        0
    };
    let pad_count = iketa.saturating_sub(int_digits.len());

    for _ in 0..pad_count {
        let pad_idx = match float_obj.zero_padding {
            1 => 0,  // zero fill
            2 => 10, // space fill
            _ => -1, // no padding (skip)
        };
        if pad_idx >= 0 {
            commands.push(DigitDrawCommand {
                source_index: pad_idx,
                dst_rect: Rect::new(x, 0.0, digit_w, rect.h),
            });
        }
        x += digit_w;
    }

    for &d in &int_digits[start..] {
        commands.push(DigitDrawCommand {
            source_index: d,
            dst_rect: Rect::new(x, 0.0, digit_w, rect.h),
        });
        x += digit_w;
    }

    // Decimal point
    if fketa > 0 {
        commands.push(DigitDrawCommand {
            source_index: 10,
            dst_rect: Rect::new(x, 0.0, digit_w, rect.h),
        });
        x += digit_w;

        // Fractional digits
        for &d in &frac_digits {
            commands.push(DigitDrawCommand {
                source_index: d,
                dst_rect: Rect::new(x, 0.0, digit_w, rect.h),
            });
            x += digit_w;
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_float(iketa: i32, fketa: i32, gain: f32) -> SkinFloat {
        SkinFloat {
            iketa,
            fketa,
            gain,
            ..Default::default()
        }
    }

    #[test]
    fn simple_integer() {
        let rect = Rect::new(0.0, 0.0, 60.0, 20.0);
        let f = make_float(3, 0, 1.0);
        let cmds = compute_float_draw(42.0, &rect, &f, 20.0);
        // No padding (zero_padding=0): only visible digits 4, 2
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].source_index, 4);
        assert_eq!(cmds[1].source_index, 2);
    }

    #[test]
    fn with_decimal() {
        let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
        let f = make_float(2, 2, 1.0);
        let cmds = compute_float_draw(3.14, &rect, &f, 20.0);
        // No padding: 3 + decimal point + 1 + 4 = 4 commands
        assert_eq!(cmds.len(), 4);
        assert_eq!(cmds[0].source_index, 3); // integer 3
        assert_eq!(cmds[1].source_index, 10); // decimal point
        assert_eq!(cmds[2].source_index, 1); // frac 1
        assert_eq!(cmds[3].source_index, 4); // frac 4 (approximately)
    }

    #[test]
    fn zero_value() {
        let rect = Rect::new(0.0, 0.0, 60.0, 20.0);
        let f = make_float(1, 1, 1.0);
        let cmds = compute_float_draw(0.0, &rect, &f, 20.0);
        // 1 int digit + decimal + 1 frac = 3
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].source_index, 0);
        assert_eq!(cmds[1].source_index, 10);
        assert_eq!(cmds[2].source_index, 0);
    }
}
