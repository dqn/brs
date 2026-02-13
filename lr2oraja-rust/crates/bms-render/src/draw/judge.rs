use bms_skin::image_handle::{ImageHandle, ImageRegion};
use bms_skin::property_id::IntegerId;
use bms_skin::skin_image::SkinImageSource;
use bms_skin::skin_judge::SkinJudge;
use bms_skin::skin_object::Rect;
use bms_skin::skin_source::image_index;

use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

use super::number::{self, NumberConfig};

/// A draw command for judge rendering (either judge image or combo digit).
#[derive(Debug, Clone)]
pub struct JudgeDrawCommand {
    pub image_region: ImageRegion,
    pub dst_rect: Rect,
}

/// Computes draw commands for a SkinJudge object.
///
/// Returns commands for the judge image and combo number digits.
pub fn compute_judge_draw(
    judge: &SkinJudge,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    rect: &Rect,
) -> Vec<JudgeDrawCommand> {
    let mut commands = Vec::new();

    // Get current judge type: property IDs 75 (1P) / 175 (2P)
    let judge_type_id = if judge.player == 0 { 75 } else { 175 };
    let judge_type = provider.integer_value(IntegerId(judge_type_id)) as usize;
    if judge_type >= bms_skin::skin_judge::JUDGE_COUNT {
        return commands;
    }

    // Render judge image
    if let Some(ref img) = judge.judge_images[judge_type] {
        let handle = resolve_skin_image_handle(img, time);
        if let Some(handle) = handle
            && tex_map.get(handle).is_some()
        {
            commands.push(JudgeDrawCommand {
                image_region: ImageRegion {
                    handle,
                    x: 0.0,
                    y: 0.0,
                    w: 0.0,
                    h: 0.0,
                },
                dst_rect: Rect::new(0.0, 0.0, rect.w, rect.h),
            });
        }
    }

    // Render combo number
    if let Some(ref num) = judge.judge_counts[judge_type] {
        // Get combo count: property IDs 71 (1P) / 171 (2P)
        let combo_id = if judge.player == 0 { 71 } else { 171 };
        let combo = provider.integer_value(IntegerId(combo_id));

        let digit_images = num.digit_sources.get_images(time);
        if let Some(digit_images) = digit_images {
            let digit_w = if num.keta > 0 {
                rect.w / num.keta as f32
            } else {
                rect.w
            };

            let config = NumberConfig {
                keta: num.keta,
                zero_padding: num.zero_padding,
                align: num.align,
                space: num.space,
                digit_w,
                negative: num.has_minus_images,
            };

            let num_dst = Rect::new(0.0, rect.h * 0.5, rect.w, rect.h * 0.5);
            let digit_cmds = number::compute_number_draw(combo, &num_dst, config);

            for cmd in &digit_cmds {
                let src_idx = cmd.source_index as usize;
                if src_idx < digit_images.len()
                    && tex_map.get(digit_images[src_idx].handle).is_some()
                {
                    commands.push(JudgeDrawCommand {
                        image_region: digit_images[src_idx],
                        dst_rect: cmd.dst_rect,
                    });
                }
            }
        }
    }

    commands
}

/// Resolves the current image handle from a SkinImage.
fn resolve_skin_image_handle(
    img: &bms_skin::skin_image::SkinImage,
    time: i64,
) -> Option<ImageHandle> {
    let source = img.sources.first()?;
    match source {
        SkinImageSource::Frames { images, cycle, .. } => {
            if images.is_empty() {
                return None;
            }
            let idx = image_index(images.len(), time, *cycle);
            Some(images[idx])
        }
        SkinImageSource::Reference(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_judge_produces_no_commands() {
        let judge = SkinJudge::default();
        let provider = crate::state_provider::StaticStateProvider::default();
        let tex_map = TextureMap::new();
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

        let cmds = compute_judge_draw(&judge, &provider, &tex_map, 0, &rect);
        assert!(cmds.is_empty());
    }
}
