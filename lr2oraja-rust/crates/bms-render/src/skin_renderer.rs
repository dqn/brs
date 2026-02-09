// Main skin render system.
//
// Each frame, iterates over Skin.objects in order, resolves draw conditions,
// interpolates animations, applies offsets, and updates Bevy entities.

use bevy::prelude::*;

use bms_skin::skin::Skin;
use bms_skin::skin_object::SkinOffset;
use bms_skin::skin_object_type::SkinObjectType;

use crate::coord::skin_to_bevy_transform;
use crate::draw;
use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

// ---------------------------------------------------------------------------
// Marker component for skin object entities
// ---------------------------------------------------------------------------

/// Marker component for entities managed by the skin renderer.
#[derive(Component)]
pub struct SkinObjectEntity {
    /// Index into Skin.objects Vec.
    pub object_index: usize,
}

// ---------------------------------------------------------------------------
// Bevy Resource holding the skin render state
// ---------------------------------------------------------------------------

/// Bevy Resource holding all skin rendering state.
#[derive(Resource)]
pub struct SkinRenderState {
    pub skin: Skin,
    pub texture_map: TextureMap,
    pub state_provider: Box<dyn SkinStateProvider>,
}

// ---------------------------------------------------------------------------
// Setup: spawn entities for each skin object
// ---------------------------------------------------------------------------

/// Spawns one Bevy entity per skin object and inserts the SkinRenderState resource.
pub fn setup_skin(
    commands: &mut Commands,
    skin: Skin,
    texture_map: TextureMap,
    state_provider: Box<dyn SkinStateProvider>,
) {
    let count = skin.objects.len();

    // Spawn one entity per skin object (initially invisible)
    for i in 0..count {
        commands.spawn((
            Sprite::default(),
            Transform::default(),
            Visibility::Hidden,
            SkinObjectEntity { object_index: i },
        ));
    }

    commands.insert_resource(SkinRenderState {
        skin,
        texture_map,
        state_provider,
    });
}

// ---------------------------------------------------------------------------
// Per-frame render system
// ---------------------------------------------------------------------------

/// Per-frame system that updates all skin object entities.
pub fn skin_render_system(
    render_state: Option<Res<SkinRenderState>>,
    mut query: Query<(
        &SkinObjectEntity,
        &mut Transform,
        &mut Visibility,
        &mut Sprite,
    )>,
) {
    let Some(state) = render_state else {
        return;
    };

    let skin = &state.skin;
    let provider = &*state.state_provider;
    let tex_map = &state.texture_map;

    for (marker, mut transform, mut visibility, mut sprite) in &mut query {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

        // 1. Check draw conditions
        if !check_draw_conditions(base, provider) {
            *visibility = Visibility::Hidden;
            continue;
        }

        // 2. Resolve timer → animation time
        let time = resolve_timer_time(base, provider);
        let Some(time) = time else {
            *visibility = Visibility::Hidden;
            continue;
        };

        // 3. Interpolate
        let Some((mut rect, color, angle)) = base.interpolate(time) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        // 4. Apply offsets
        let mut angle_offset = 0.0_f32;
        let mut alpha_offset = 0.0_f32;
        for &oid in &base.offset_ids {
            let off = provider.offset_value(oid);
            apply_offset(&mut rect, &off, &mut angle_offset, &mut alpha_offset);
        }

        let final_angle = angle + angle_offset as i32;
        let final_alpha = (color.a + alpha_offset / 255.0).clamp(0.0, 1.0);

        // 5. Object-type-specific dispatch
        let (tex_handle, src_rect_uv) = resolve_object_texture(object, provider, tex_map, time);

        // 6. Update entity
        *transform = skin_to_bevy_transform(
            crate::coord::SkinRect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
            },
            crate::coord::ScreenSize {
                w: skin.width,
                h: skin.height,
            },
            idx as f32 * 0.001,
            crate::coord::RotationParams {
                angle_deg: final_angle,
                center_x: base.center_x,
                center_y: base.center_y,
            },
        );

        // Set sprite size and color
        sprite.custom_size = Some(Vec2::new(rect.w, rect.h));
        sprite.color = Color::srgba(color.r, color.g, color.b, final_alpha);

        if let Some(handle) = tex_handle {
            sprite.image = handle;
        }

        if let Some(uv_rect) = src_rect_uv {
            sprite.rect = Some(uv_rect);
        } else {
            sprite.rect = None;
        }

        *visibility = Visibility::Visible;
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Checks whether all draw conditions are met.
fn check_draw_conditions(
    base: &bms_skin::skin_object::SkinObjectBase,
    provider: &dyn SkinStateProvider,
) -> bool {
    for &cond in &base.draw_conditions {
        if !provider.boolean_value(cond) {
            return false;
        }
    }
    true
}

/// Resolves the animation time from the base timer.
/// Returns None if the timer is required but inactive.
fn resolve_timer_time(
    base: &bms_skin::skin_object::SkinObjectBase,
    provider: &dyn SkinStateProvider,
) -> Option<i64> {
    match base.timer {
        Some(timer_id) => provider.timer_value(timer_id),
        None => Some(provider.now_time_ms()),
    }
}

/// Applies a SkinOffset to the current rect and accumulates angle/alpha offsets.
fn apply_offset(
    rect: &mut bms_skin::skin_object::Rect,
    offset: &SkinOffset,
    angle_offset: &mut f32,
    alpha_offset: &mut f32,
) {
    rect.x += offset.x;
    rect.y += offset.y;
    rect.w += offset.w;
    rect.h += offset.h;
    *angle_offset += offset.r;
    *alpha_offset += offset.a;
}

/// Resolves the texture handle and optional UV rect for a skin object.
fn resolve_object_texture(
    object: &SkinObjectType,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
) -> (Option<Handle<Image>>, Option<bevy::math::Rect>) {
    match object {
        SkinObjectType::Image(img) => {
            // Select source based on ref_id
            let source_idx = img
                .ref_id
                .map(|id| provider.integer_value(id) as usize)
                .unwrap_or(0);
            let source = img.sources.get(source_idx).or(img.sources.first());

            if let Some(source) = source {
                match source {
                    bms_skin::skin_image::SkinImageSource::Frames { images, cycle, .. } => {
                        let idx = bms_skin::skin_source::image_index(images.len(), time, *cycle);
                        if let Some(handle) = images.get(idx)
                            && let Some(entry) = tex_map.get(*handle)
                        {
                            return (Some(entry.handle.clone()), None);
                        }
                    }
                    bms_skin::skin_image::SkinImageSource::Reference(_id) => {
                        // Reference sources need runtime image table resolution (Phase 11)
                    }
                }
            }
            (None, None)
        }
        SkinObjectType::Slider(slider) => {
            let value = slider
                .ref_id
                .map(|id| provider.float_value(id))
                .unwrap_or(0.0);
            let (ox, oy) =
                draw::slider::compute_slider_offset(slider.direction, slider.range, value);
            // Slider offset is applied via transform, texture is from source_images
            let idx = bms_skin::skin_source::image_index(
                slider.source_images.len(),
                time,
                slider.source_cycle,
            );
            if let Some(handle) = slider.source_images.get(idx)
                && let Some(entry) = tex_map.get(*handle)
            {
                return (Some(entry.handle.clone()), None);
            }
            let _ = (ox, oy); // Offset should be applied to transform in Phase 11
            (None, None)
        }
        SkinObjectType::Graph(graph) => {
            let value = graph
                .ref_id
                .map(|id| provider.float_value(id))
                .unwrap_or(0.0);
            let idx = bms_skin::skin_source::image_index(
                graph.source_images.len(),
                time,
                graph.source_cycle,
            );
            if let Some(handle) = graph.source_images.get(idx)
                && let Some(entry) = tex_map.get(*handle)
            {
                let src = bms_skin::skin_object::Rect::new(0.0, 0.0, entry.width, entry.height);
                let dst = bms_skin::skin_object::Rect::new(0.0, 0.0, entry.width, entry.height);
                let cmd = draw::graph::compute_graph_draw(graph.direction, value, &src, &dst);
                let uv = bevy::math::Rect::new(
                    cmd.src_rect.x,
                    cmd.src_rect.y,
                    cmd.src_rect.x + cmd.src_rect.w,
                    cmd.src_rect.y + cmd.src_rect.h,
                );
                return (Some(entry.handle.clone()), Some(uv));
            }
            (None, None)
        }
        // Number, Text, Gauge, and Visualizers need more complex multi-entity rendering.
        // Basic single-entity rendering is stubbed for Phase 10 and will be
        // fully implemented in Phase 11 when entity pools are expanded.
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_provider::StaticStateProvider;
    use bms_skin::property_id::{BooleanId, TimerId};
    use bms_skin::skin_object::{Destination, Rect as SkinRect, SkinObjectBase};

    fn make_base_with_dst(time: i64, x: f32, y: f32, w: f32, h: f32) -> SkinObjectBase {
        let mut base = SkinObjectBase::default();
        base.add_destination(Destination {
            time,
            region: SkinRect::new(x, y, w, h),
            color: bms_skin::skin_object::Color::white(),
            angle: 0,
            acc: 0,
        });
        base
    }

    #[test]
    fn check_draw_conditions_all_true() {
        let mut base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        base.draw_conditions = vec![BooleanId(1), BooleanId(2)];
        let mut p = StaticStateProvider::default();
        p.booleans.insert(1, true);
        p.booleans.insert(2, true);
        assert!(check_draw_conditions(&base, &p));
    }

    #[test]
    fn check_draw_conditions_one_false() {
        let mut base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        base.draw_conditions = vec![BooleanId(1), BooleanId(2)];
        let mut p = StaticStateProvider::default();
        p.booleans.insert(1, true);
        p.booleans.insert(2, false);
        assert!(!check_draw_conditions(&base, &p));
    }

    #[test]
    fn check_draw_conditions_empty() {
        let base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        let p = StaticStateProvider::default();
        assert!(check_draw_conditions(&base, &p));
    }

    #[test]
    fn check_draw_conditions_negated() {
        let mut base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        base.draw_conditions = vec![BooleanId(-5)]; // NOT id=5
        let mut p = StaticStateProvider::default();
        p.booleans.insert(5, false);
        // NOT false = true
        assert!(check_draw_conditions(&base, &p));
    }

    #[test]
    fn resolve_timer_time_no_timer() {
        let base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        let mut p = StaticStateProvider::default();
        p.time_ms = 5000;
        assert_eq!(resolve_timer_time(&base, &p), Some(5000));
    }

    #[test]
    fn resolve_timer_time_active_timer() {
        let mut base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        base.timer = Some(TimerId(10));
        let mut p = StaticStateProvider::default();
        p.timers.insert(10, 3000);
        assert_eq!(resolve_timer_time(&base, &p), Some(3000));
    }

    #[test]
    fn resolve_timer_time_inactive_timer() {
        let mut base = make_base_with_dst(0, 0.0, 0.0, 100.0, 100.0);
        base.timer = Some(TimerId(10));
        let p = StaticStateProvider::default(); // timer 10 not set
        assert_eq!(resolve_timer_time(&base, &p), None);
    }

    #[test]
    fn apply_offset_accumulates() {
        let mut rect = SkinRect::new(10.0, 20.0, 100.0, 50.0);
        let offset = SkinOffset {
            x: 5.0,
            y: -3.0,
            w: 10.0,
            h: -5.0,
            r: 15.0,
            a: 32.0,
        };
        let mut angle_off = 0.0_f32;
        let mut alpha_off = 0.0_f32;
        apply_offset(&mut rect, &offset, &mut angle_off, &mut alpha_off);

        assert!((rect.x - 15.0).abs() < 0.001);
        assert!((rect.y - 17.0).abs() < 0.001);
        assert!((rect.w - 110.0).abs() < 0.001);
        assert!((rect.h - 45.0).abs() < 0.001);
        assert!((angle_off - 15.0).abs() < 0.001);
        assert!((alpha_off - 32.0).abs() < 0.001);
    }

    #[test]
    fn apply_multiple_offsets() {
        let mut rect = SkinRect::new(0.0, 0.0, 100.0, 100.0);
        let off1 = SkinOffset {
            x: 10.0,
            y: 20.0,
            w: 0.0,
            h: 0.0,
            r: 5.0,
            a: 10.0,
        };
        let off2 = SkinOffset {
            x: -5.0,
            y: -10.0,
            w: 0.0,
            h: 0.0,
            r: -3.0,
            a: 20.0,
        };
        let mut angle_off = 0.0_f32;
        let mut alpha_off = 0.0_f32;
        apply_offset(&mut rect, &off1, &mut angle_off, &mut alpha_off);
        apply_offset(&mut rect, &off2, &mut angle_off, &mut alpha_off);

        assert!((rect.x - 5.0).abs() < 0.001);
        assert!((rect.y - 10.0).abs() < 0.001);
        assert!((angle_off - 2.0).abs() < 0.001);
        assert!((alpha_off - 30.0).abs() < 0.001);
    }

    #[test]
    fn interpolation_with_offset() {
        let mut base = SkinObjectBase::default();
        base.add_destination(Destination {
            time: 0,
            region: SkinRect::new(0.0, 0.0, 100.0, 100.0),
            color: bms_skin::skin_object::Color::white(),
            angle: 0,
            acc: 0,
        });
        base.add_destination(Destination {
            time: 100,
            region: SkinRect::new(100.0, 0.0, 100.0, 100.0),
            color: bms_skin::skin_object::Color::white(),
            angle: 0,
            acc: 0,
        });
        base.set_offset_ids(&[1]);

        let (mut rect, _color, _angle) = base.interpolate(50).unwrap();
        // At t=50: x should be 50
        assert!((rect.x - 50.0).abs() < 0.001);

        let offset = SkinOffset {
            x: 10.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            r: 0.0,
            a: 0.0,
        };
        let mut ao = 0.0_f32;
        let mut aao = 0.0_f32;
        apply_offset(&mut rect, &offset, &mut ao, &mut aao);
        // After offset: x = 60
        assert!((rect.x - 60.0).abs() < 0.001);
    }
}
