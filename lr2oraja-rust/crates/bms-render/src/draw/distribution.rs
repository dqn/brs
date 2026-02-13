use bms_skin::image_handle::ImageHandle;
use bms_skin::property_id::IntegerId;
use bms_skin::skin_distribution_graph::SkinDistributionGraph;
use bms_skin::skin_object::Rect;

use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

/// A single bar in the distribution graph.
#[derive(Debug, Clone)]
pub struct DistributionBarCommand {
    pub image_handle: ImageHandle,
    pub dst_rect: Rect,
}

/// Computes draw commands for a SkinDistributionGraph.
///
/// Each bar's width is proportional to its count relative to the total.
pub fn compute_distribution_draw(
    dg: &SkinDistributionGraph,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    rect: &Rect,
) -> Vec<DistributionBarCommand> {
    let mut commands = Vec::new();

    let entry_count = dg.images.len();
    if entry_count == 0 {
        return commands;
    }

    // Get distribution data from provider
    // Property ID base: 370 (lamp) or 380 (rank)
    let base_id = if dg.graph_type == 0 { 370 } else { 380 };

    let mut counts: Vec<i32> = Vec::with_capacity(entry_count);
    let mut total = 0i64;
    for i in 0..entry_count {
        let count = provider.integer_value(IntegerId(base_id + i as i32));
        counts.push(count);
        total += count as i64;
    }

    if total == 0 {
        return commands;
    }

    let bar_h = rect.h / entry_count as f32;
    let mut y = 0.0_f32;

    for (i, &count) in counts.iter().enumerate() {
        if count <= 0 {
            y += bar_h;
            continue;
        }

        let Some(image_id) = dg.images[i] else {
            y += bar_h;
            continue;
        };

        let handle = ImageHandle(image_id as u32);
        if tex_map.get(handle).is_none() {
            y += bar_h;
            continue;
        }

        let bar_w = (count as f64 / total as f64 * rect.w as f64) as f32;

        commands.push(DistributionBarCommand {
            image_handle: handle,
            dst_rect: Rect::new(0.0, y, bar_w, bar_h),
        });

        y += bar_h;
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_distribution() {
        let dg = SkinDistributionGraph::default();
        let provider = crate::state_provider::StaticStateProvider::default();
        let tex_map = TextureMap::new();
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);

        let cmds = compute_distribution_draw(&dg, &provider, &tex_map, &rect);
        assert!(cmds.is_empty());
    }
}
