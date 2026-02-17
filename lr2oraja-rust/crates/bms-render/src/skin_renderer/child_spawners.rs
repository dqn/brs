// Child entity spawning functions for multi-entity skin objects.

use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use bms_skin::skin_object_type::SkinObjectType;

use super::components::MultiEntityChild;
use crate::bga_layer_material::BgaLayerMaterial;
use crate::coord::skin_to_bevy_transform;
use crate::draw;
use crate::draw::bar::BarScrollState;
use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

/// Computes a hash of the current multi-entity state for change detection.
pub fn compute_multi_entity_hash(
    object: &SkinObjectType,
    provider: &dyn SkinStateProvider,
    time: i64,
    rect: &bms_skin::skin_object::Rect,
    bar_state: Option<&BarScrollState>,
) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    time.hash(&mut hasher);
    rect.x.to_bits().hash(&mut hasher);
    rect.y.to_bits().hash(&mut hasher);
    rect.w.to_bits().hash(&mut hasher);
    rect.h.to_bits().hash(&mut hasher);

    match object {
        SkinObjectType::Number(num) => {
            0u8.hash(&mut hasher);
            let value = num.ref_id.map(|id| provider.integer_value(id)).unwrap_or(0);
            value.hash(&mut hasher);
        }
        SkinObjectType::Float(f) => {
            1u8.hash(&mut hasher);
            let value = f.ref_id.map(|id| provider.float_value(id)).unwrap_or(0.0);
            value.to_bits().hash(&mut hasher);
        }
        SkinObjectType::Gauge(g) => {
            2u8.hash(&mut hasher);
            // Gauge value from float provider (groove gauge ref)
            let value = provider.float_value(bms_skin::property_id::FloatId(107));
            value.to_bits().hash(&mut hasher);
            g.nodes.hash(&mut hasher);
        }
        SkinObjectType::Judge(j) => {
            3u8.hash(&mut hasher);
            j.player.hash(&mut hasher);
            // Current judge type
            let judge_type =
                provider.integer_value(bms_skin::property_id::IntegerId(if j.player == 0 {
                    75
                } else {
                    175
                }));
            judge_type.hash(&mut hasher);
            // Combo count
            let combo =
                provider.integer_value(bms_skin::property_id::IntegerId(if j.player == 0 {
                    71
                } else {
                    171
                }));
            combo.hash(&mut hasher);
        }
        SkinObjectType::DistributionGraph(dg) => {
            4u8.hash(&mut hasher);
            dg.graph_type.hash(&mut hasher);
        }
        SkinObjectType::Bar(_) => {
            5u8.hash(&mut hasher);
            if let Some(bs) = bar_state {
                bs.selected_index.hash(&mut hasher);
                bs.angle_lerp.to_bits().hash(&mut hasher);
                bs.angle.hash(&mut hasher);
                bs.total_bars.hash(&mut hasher);
                for slot in &bs.slots {
                    slot.lamp_id.hash(&mut hasher);
                    slot.level.hash(&mut hasher);
                    slot.difficulty.hash(&mut hasher);
                    slot.text_type.hash(&mut hasher);
                    slot.title.hash(&mut hasher);
                    slot.features.hash(&mut hasher);
                }
            }
        }
        SkinObjectType::Bga(_) => {
            6u8.hash(&mut hasher);
            provider.is_poor_active().hash(&mut hasher);
            // Hash AssetId of each BGA image handle for change detection
            if let Some(h) = provider.bga_image() {
                h.id().hash(&mut hasher);
            }
            if let Some(h) = provider.layer_image() {
                h.id().hash(&mut hasher);
            }
            if let Some(h) = provider.poor_image() {
                h.id().hash(&mut hasher);
            }
        }
        _ => {}
    }

    hasher.finish()
}

/// Spawns child sprites for a SkinNumber.
#[allow(clippy::too_many_arguments)]
pub fn spawn_number_children(
    commands: &mut Commands,
    parent: Entity,
    num: &bms_skin::skin_number::SkinNumber,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
) {
    let value = num.ref_id.map(|id| provider.integer_value(id)).unwrap_or(0);

    // Java: (value >= 0 || mimage == null) ? this.image : mimage
    let digit_images = if value < 0 {
        num.minus_digit_sources
            .as_ref()
            .and_then(|s| s.get_images(time))
            .or_else(|| num.digit_sources.get_images(time))
    } else {
        num.digit_sources.get_images(time)
    };
    let Some(digit_images) = digit_images else {
        return;
    };

    let digit_w = if num.keta > 0 {
        rect.w / num.keta as f32
    } else {
        rect.w
    };

    let config = draw::number::NumberConfig {
        keta: num.keta,
        zero_padding: num.zero_padding,
        align: num.align,
        space: num.space,
        digit_w,
        negative: num.minus_digit_sources.is_some(),
    };

    let dst = bms_skin::skin_object::Rect::new(0.0, 0.0, rect.w, rect.h);
    let cmds = draw::number::compute_number_draw(value, &dst, config);

    for cmd in &cmds {
        let src_idx = cmd.source_index as usize;
        if src_idx >= digit_images.len() {
            continue;
        }
        let region = &digit_images[src_idx];
        let Some(entry) = tex_map.get(region.handle) else {
            continue;
        };

        let local_x = cmd.dst_rect.x + cmd.dst_rect.w / 2.0 - rect.w / 2.0;
        let local_y = -(cmd.dst_rect.y + cmd.dst_rect.h / 2.0 - rect.h / 2.0);

        let texture_rect = if region.w > 0.0 && region.h > 0.0 {
            Some(bevy::math::Rect::new(
                region.x,
                region.y,
                region.x + region.w,
                region.y + region.h,
            ))
        } else {
            None
        };

        commands.entity(parent).with_child((
            Sprite {
                image: entry.handle.clone(),
                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                color: obj_color,
                rect: texture_rect,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            MultiEntityChild,
        ));
    }
}

/// Spawns child sprites for a SkinFloat.
#[allow(clippy::too_many_arguments)]
pub fn spawn_float_children(
    commands: &mut Commands,
    parent: Entity,
    float_obj: &bms_skin::skin_float::SkinFloat,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
) {
    let value = float_obj
        .ref_id
        .map(|id| provider.float_value(id))
        .unwrap_or(0.0)
        * float_obj.gain;

    // Java: (mimage == null || v >= 0.0f) ? this.image : mimage
    let digit_images = if value < 0.0 {
        float_obj
            .minus_digit_sources
            .as_ref()
            .and_then(|s| s.get_images(time))
            .or_else(|| float_obj.digit_sources.get_images(time))
    } else {
        float_obj.digit_sources.get_images(time)
    };
    let Some(digit_images) = digit_images else {
        return;
    };

    let total_keta = float_obj.iketa + float_obj.fketa + 1; // +1 for decimal point
    let digit_w = if total_keta > 0 {
        rect.w / total_keta as f32
    } else {
        rect.w
    };

    let cmds = draw::float::compute_float_draw(value, rect, float_obj, digit_w);

    for cmd in &cmds {
        let src_idx = cmd.source_index as usize;
        if src_idx >= digit_images.len() {
            continue;
        }
        let region = &digit_images[src_idx];
        let Some(entry) = tex_map.get(region.handle) else {
            continue;
        };

        let local_x = cmd.dst_rect.x + cmd.dst_rect.w / 2.0 - rect.w / 2.0;
        let local_y = -(cmd.dst_rect.y + cmd.dst_rect.h / 2.0 - rect.h / 2.0);

        let texture_rect = if region.w > 0.0 && region.h > 0.0 {
            Some(bevy::math::Rect::new(
                region.x,
                region.y,
                region.x + region.w,
                region.y + region.h,
            ))
        } else {
            None
        };

        commands.entity(parent).with_child((
            Sprite {
                image: entry.handle.clone(),
                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                color: obj_color,
                rect: texture_rect,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            MultiEntityChild,
        ));
    }
}

/// Spawns child sprites for a SkinGauge.
#[allow(clippy::too_many_arguments)]
pub fn spawn_gauge_children(
    commands: &mut Commands,
    parent: Entity,
    gauge: &bms_skin::skin_gauge::SkinGauge,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
) {
    let gauge_value = provider.float_value(bms_skin::property_id::FloatId(107));

    let parts: Vec<_> = gauge
        .parts
        .iter()
        .map(|p| (p.part_type, p.images.clone(), p.timer, p.cycle))
        .collect();

    let dst = bms_skin::skin_object::Rect::new(0.0, 0.0, rect.w, rect.h);
    let cmds = draw::gauge::compute_gauge_draw(gauge.nodes, gauge_value, &parts, time, &dst);

    for cmd in &cmds {
        let region = &cmd.image_region;
        let Some(entry) = tex_map.get(region.handle) else {
            continue;
        };

        let local_x = cmd.dst_rect.x + cmd.dst_rect.w / 2.0 - rect.w / 2.0;
        let local_y = -(cmd.dst_rect.y + cmd.dst_rect.h / 2.0 - rect.h / 2.0);

        let texture_rect = if region.w > 0.0 && region.h > 0.0 {
            Some(bevy::math::Rect::new(
                region.x,
                region.y,
                region.x + region.w,
                region.y + region.h,
            ))
        } else {
            None
        };

        commands.entity(parent).with_child((
            Sprite {
                image: entry.handle.clone(),
                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                color: obj_color,
                rect: texture_rect,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            MultiEntityChild,
        ));
    }
}

/// Spawns child sprites for a SkinJudge.
#[allow(clippy::too_many_arguments)]
pub fn spawn_judge_children(
    commands: &mut Commands,
    parent: Entity,
    judge: &bms_skin::skin_judge::SkinJudge,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
) {
    let cmds = draw::judge::compute_judge_draw(judge, provider, tex_map, time, rect);

    for cmd in &cmds {
        let region = &cmd.image_region;
        let Some(entry) = tex_map.get(region.handle) else {
            continue;
        };

        let local_x = cmd.dst_rect.x + cmd.dst_rect.w / 2.0 - rect.w / 2.0;
        let local_y = -(cmd.dst_rect.y + cmd.dst_rect.h / 2.0 - rect.h / 2.0);

        let texture_rect = if region.w > 0.0 && region.h > 0.0 {
            Some(bevy::math::Rect::new(
                region.x,
                region.y,
                region.x + region.w,
                region.y + region.h,
            ))
        } else {
            None
        };

        commands.entity(parent).with_child((
            Sprite {
                image: entry.handle.clone(),
                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                color: obj_color,
                rect: texture_rect,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            MultiEntityChild,
        ));
    }
}

/// Spawns child sprites for a SkinDistributionGraph.
pub fn spawn_distribution_children(
    commands: &mut Commands,
    parent: Entity,
    dg: &bms_skin::skin_distribution_graph::SkinDistributionGraph,
    provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
) {
    let cmds = draw::distribution::compute_distribution_draw(dg, provider, tex_map, rect);

    for cmd in &cmds {
        let Some(entry) = tex_map.get(cmd.image_handle) else {
            continue;
        };

        let local_x = cmd.dst_rect.x + cmd.dst_rect.w / 2.0 - rect.w / 2.0;
        let local_y = -(cmd.dst_rect.y + cmd.dst_rect.h / 2.0 - rect.h / 2.0);

        commands.entity(parent).with_child((
            Sprite {
                image: entry.handle.clone(),
                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                color: obj_color,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            MultiEntityChild,
        ));
    }
}

/// Spawns child sprites for a SkinBar.
///
/// Renders the bar list for the music select screen: body images, lamps,
/// trophies, levels, labels for each of the 60 bar slots.
/// Position calculation follows Java BarRenderer.prepare()/render().
#[allow(clippy::too_many_arguments)]
pub fn spawn_bar_children(
    commands: &mut Commands,
    parent: Entity,
    bar: &bms_skin::skin_bar::SkinBar,
    bar_state: &BarScrollState,
    _provider: &dyn SkinStateProvider,
    tex_map: &TextureMap,
    time: i64,
    _rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
    screen_w: f32,
    screen_h: f32,
) {
    use crate::draw::bar::BarType;
    use bms_skin::skin_bar::BAR_COUNT;
    use bms_skin::skin_image::SkinImageSource;

    let total = bar_state.total_bars;
    if total == 0 || bar_state.slots.is_empty() {
        return;
    }

    let center = bar_state.center_bar;
    let selected = bar_state.selected_index;
    let angle_lerp = bar_state.angle_lerp.clamp(-1.0, 1.0);
    let angle = bar_state.angle;
    let slot_count = bar_state.slots.len().min(BAR_COUNT);

    let screen = crate::coord::ScreenSize {
        w: screen_w,
        h: screen_h,
    };

    // Helper: resolve image handle from SkinImage sources at given time.
    let resolve_image = |img: &bms_skin::skin_image::SkinImage,
                         source_idx: usize|
     -> Option<(
        bms_skin::image_handle::ImageHandle,
        Option<bms_skin::skin_object::Rect>,
    )> {
        let src = img.sources.get(source_idx).or(img.sources.first())?;
        match src {
            SkinImageSource::Frames { images, cycle, .. } => {
                let idx = bms_skin::skin_source::image_index(images.len(), time, *cycle);
                images.get(idx).map(|h| (*h, img.source_rect))
            }
            SkinImageSource::Reference(_) => None,
        }
    };

    // Helper: get the DST rect for a bar slot from its base destinations.
    let get_slot_dst =
        |img: &bms_skin::skin_image::SkinImage| -> Option<bms_skin::skin_object::Rect> {
            let dst = img.base.destinations.first()?;
            Some(bms_skin::skin_object::Rect::new(
                dst.region.x,
                dst.region.y,
                dst.region.w,
                dst.region.h,
            ))
        };

    // Render each bar slot
    for i in 0..slot_count {
        // Determine which data slot maps to this visual slot
        let data_idx = ((selected as i64 + total as i64 * 100 + i as i64 - center as i64)
            % total as i64) as usize;
        let slot_data = match bar_state.slots.get(data_idx) {
            Some(s) => s,
            None => continue,
        };

        let is_selected = i == center;

        // Select body image (on = selected, off = unselected)
        let body_img = if is_selected {
            bar.bar_image_on.get(i).and_then(|o| o.as_ref())
        } else {
            bar.bar_image_off.get(i).and_then(|o| o.as_ref())
        }
        .or_else(|| bar.bar_image_off.get(i).and_then(|o| o.as_ref()));

        let Some(body_img) = body_img else {
            continue;
        };

        let Some(body_dst) = get_slot_dst(body_img) else {
            continue;
        };

        // Calculate scroll interpolation
        let mut slot_x = body_dst.x;
        let mut slot_y = body_dst.y;
        let slot_w = body_dst.w;
        let slot_h = body_dst.h;

        if angle != 0 {
            // Interpolate with adjacent slot
            let adj_i = if angle > 0 {
                if i + 1 < BAR_COUNT { i + 1 } else { i }
            } else if i > 0 {
                i - 1
            } else {
                i
            };

            let adj_img = if is_selected {
                bar.bar_image_on.get(adj_i).and_then(|o| o.as_ref())
            } else {
                bar.bar_image_off.get(adj_i).and_then(|o| o.as_ref())
            }
            .or_else(|| bar.bar_image_off.get(adj_i).and_then(|o| o.as_ref()));

            if let Some(adj_img) = adj_img
                && let Some(adj_dst) = get_slot_dst(adj_img)
            {
                let lerp = angle_lerp.abs();
                slot_x += (adj_dst.x - body_dst.x) * lerp;
                slot_y += (adj_dst.y - body_dst.y) * lerp;
            }
        }

        // Spawn bar body sprite
        let bar_type_idx = match &slot_data.bar_type {
            BarType::Song { exists: true } => 0,
            BarType::Song { exists: false } => 0,
            BarType::Folder => 1,
            BarType::Grade { .. } => 2,
            BarType::Table => 3,
            BarType::Command => 4,
            BarType::Search => 5,
            BarType::Function {
                display_bar_type, ..
            } => *display_bar_type as usize,
        };

        if let Some((handle, src_rect)) = resolve_image(body_img, bar_type_idx)
            && let Some(entry) = tex_map.get(handle)
        {
            let transform = skin_to_bevy_transform(
                crate::coord::SkinRect {
                    x: slot_x,
                    y: slot_y,
                    w: slot_w,
                    h: slot_h,
                },
                screen,
                (i as f32) * 0.0001,
                crate::coord::RotationParams {
                    angle_deg: 0,
                    center_x: 0.0,
                    center_y: 0.0,
                },
            );

            let texture_rect =
                src_rect.map(|r| bevy::math::Rect::new(r.x, r.y, r.x + r.w, r.y + r.h));

            commands.entity(parent).with_child((
                Sprite {
                    image: entry.handle.clone(),
                    custom_size: Some(Vec2::new(slot_w, slot_h)),
                    color: obj_color,
                    rect: texture_rect,
                    ..default()
                },
                transform,
                MultiEntityChild,
            ));
        }

        // Spawn lamp sprite
        let lamp_id = slot_data.lamp_id as usize;
        if let Some(Some(lamp_img)) = bar.lamp.get(lamp_id)
            && let Some(lamp_dst) = get_slot_dst(lamp_img)
        {
            let lx = slot_x + lamp_dst.x;
            let ly = slot_y + lamp_dst.y;
            if let Some((handle, src_rect)) = resolve_image(lamp_img, 0)
                && let Some(entry) = tex_map.get(handle)
            {
                let transform = skin_to_bevy_transform(
                    crate::coord::SkinRect {
                        x: lx,
                        y: ly,
                        w: lamp_dst.w,
                        h: lamp_dst.h,
                    },
                    screen,
                    (i as f32) * 0.0001 + 0.00001,
                    crate::coord::RotationParams {
                        angle_deg: 0,
                        center_x: 0.0,
                        center_y: 0.0,
                    },
                );
                let texture_rect =
                    src_rect.map(|r| bevy::math::Rect::new(r.x, r.y, r.x + r.w, r.y + r.h));
                commands.entity(parent).with_child((
                    Sprite {
                        image: entry.handle.clone(),
                        custom_size: Some(Vec2::new(lamp_dst.w, lamp_dst.h)),
                        color: obj_color,
                        rect: texture_rect,
                        ..default()
                    },
                    transform,
                    MultiEntityChild,
                ));
            }
        }

        // Spawn trophy sprite (Grade bars only)
        if let BarType::Grade { .. } = &slot_data.bar_type
            && let Some(trophy_id) = slot_data.trophy_id
            && let Some(Some(trophy_img)) = bar.trophy.get(trophy_id)
            && let Some(trophy_dst) = get_slot_dst(trophy_img)
        {
            let tx = slot_x + trophy_dst.x;
            let ty = slot_y + trophy_dst.y;
            if let Some((handle, src_rect)) = resolve_image(trophy_img, 0)
                && let Some(entry) = tex_map.get(handle)
            {
                let transform = skin_to_bevy_transform(
                    crate::coord::SkinRect {
                        x: tx,
                        y: ty,
                        w: trophy_dst.w,
                        h: trophy_dst.h,
                    },
                    screen,
                    (i as f32) * 0.0001 + 0.00002,
                    crate::coord::RotationParams {
                        angle_deg: 0,
                        center_x: 0.0,
                        center_y: 0.0,
                    },
                );
                let texture_rect =
                    src_rect.map(|r| bevy::math::Rect::new(r.x, r.y, r.x + r.w, r.y + r.h));
                commands.entity(parent).with_child((
                    Sprite {
                        image: entry.handle.clone(),
                        custom_size: Some(Vec2::new(trophy_dst.w, trophy_dst.h)),
                        color: obj_color,
                        rect: texture_rect,
                        ..default()
                    },
                    transform,
                    MultiEntityChild,
                ));
            }
        }

        // Spawn level number (Song bars only)
        if matches!(slot_data.bar_type, BarType::Song { .. }) {
            let diff_idx = slot_data.difficulty.clamp(0, 6) as usize;
            if let Some(Some(level_num)) = bar.bar_level.get(diff_idx)
                && let Some(level_dst) = level_num.base.destinations.first()
            {
                let lx = slot_x + level_dst.region.x;
                let ly = slot_y + level_dst.region.y;
                let lw = level_dst.region.w;
                let lh = level_dst.region.h;

                // Render level value using the same digit rendering as SkinNumber
                let digit_images = level_num.digit_sources.get_images(time);
                if let Some(digit_images) = digit_images {
                    let digit_w = if level_num.keta > 0 {
                        lw / level_num.keta as f32
                    } else {
                        lw
                    };

                    let config = draw::number::NumberConfig {
                        keta: level_num.keta,
                        zero_padding: level_num.zero_padding,
                        align: level_num.align,
                        space: level_num.space,
                        digit_w,
                        negative: false,
                    };

                    let num_dst = bms_skin::skin_object::Rect::new(0.0, 0.0, lw, lh);
                    let cmds = draw::number::compute_number_draw(slot_data.level, &num_dst, config);

                    for cmd in &cmds {
                        let src_idx = cmd.source_index as usize;
                        if src_idx >= digit_images.len() {
                            continue;
                        }
                        let region = &digit_images[src_idx];
                        let Some(entry) = tex_map.get(region.handle) else {
                            continue;
                        };

                        // Position digit relative to level origin
                        let dx = lx + cmd.dst_rect.x;
                        let dy = ly + cmd.dst_rect.y;
                        let transform = skin_to_bevy_transform(
                            crate::coord::SkinRect {
                                x: dx,
                                y: dy,
                                w: cmd.dst_rect.w,
                                h: cmd.dst_rect.h,
                            },
                            screen,
                            (i as f32) * 0.0001 + 0.00003,
                            crate::coord::RotationParams {
                                angle_deg: 0,
                                center_x: 0.0,
                                center_y: 0.0,
                            },
                        );

                        let texture_rect = if region.w > 0.0 && region.h > 0.0 {
                            Some(bevy::math::Rect::new(
                                region.x,
                                region.y,
                                region.x + region.w,
                                region.y + region.h,
                            ))
                        } else {
                            None
                        };

                        commands.entity(parent).with_child((
                            Sprite {
                                image: entry.handle.clone(),
                                custom_size: Some(Vec2::new(cmd.dst_rect.w, cmd.dst_rect.h)),
                                color: obj_color,
                                rect: texture_rect,
                                ..default()
                            },
                            transform,
                            MultiEntityChild,
                        ));
                    }
                }
            }
        }

        // Spawn label sprites (feature flags: LN, Mine, Random, ChargeNote, HellChargeNote)
        let label_flags = [
            (crate::draw::bar::FEATURE_LN, 0usize),
            (crate::draw::bar::FEATURE_MINE, 1),
            (crate::draw::bar::FEATURE_RANDOM, 2),
            (crate::draw::bar::FEATURE_CHARGENOTE, 3),
            (crate::draw::bar::FEATURE_HELL_CHARGENOTE, 4),
        ];
        for &(flag, label_idx) in &label_flags {
            if slot_data.features & flag != 0
                && let Some(Some(label_img)) = bar.label.get(label_idx)
                && let Some(label_dst) = get_slot_dst(label_img)
            {
                let lx = slot_x + label_dst.x;
                let ly = slot_y + label_dst.y;
                if let Some((handle, src_rect)) = resolve_image(label_img, 0)
                    && let Some(entry) = tex_map.get(handle)
                {
                    let transform = skin_to_bevy_transform(
                        crate::coord::SkinRect {
                            x: lx,
                            y: ly,
                            w: label_dst.w,
                            h: label_dst.h,
                        },
                        screen,
                        (i as f32) * 0.0001 + 0.00004,
                        crate::coord::RotationParams {
                            angle_deg: 0,
                            center_x: 0.0,
                            center_y: 0.0,
                        },
                    );
                    let texture_rect =
                        src_rect.map(|r| bevy::math::Rect::new(r.x, r.y, r.x + r.w, r.y + r.h));
                    commands.entity(parent).with_child((
                        Sprite {
                            image: entry.handle.clone(),
                            custom_size: Some(Vec2::new(label_dst.w, label_dst.h)),
                            color: obj_color,
                            rect: texture_rect,
                            ..default()
                        },
                        transform,
                        MultiEntityChild,
                    ));
                }
            }
        }

        // Note: Text rendering for bar titles is complex (requires TTF/BMFont pipeline).
        // Text spawning is deferred to a future phase when SkinText rendering is
        // integrated into the bar system.
    }
}

/// Spawns child entities for a BGA skin object.
///
/// - Poor active: single Sprite child with poor_image (fallback: bga_image)
/// - Normal: Sprite child for bga_image (z=0.0001) + Mesh2d+BgaLayerMaterial
///   child for layer_image (z=0.0002) with black-key transparency
#[allow(clippy::too_many_arguments)]
pub fn spawn_bga_children(
    commands: &mut Commands,
    parent: Entity,
    provider: &dyn SkinStateProvider,
    rect: &bms_skin::skin_object::Rect,
    obj_color: bevy::prelude::Color,
    meshes: &mut Assets<Mesh>,
    bga_layer_materials: &mut Assets<BgaLayerMaterial>,
) {
    let size = Vec2::new(rect.w, rect.h);

    if provider.is_poor_active() {
        // Poor state: show poor_image, falling back to bga_image
        let image = provider.poor_image().or_else(|| provider.bga_image());
        if let Some(handle) = image {
            commands.entity(parent).with_child((
                Sprite {
                    image: handle,
                    custom_size: Some(size),
                    color: obj_color,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0001),
                MultiEntityChild,
            ));
        }
        return;
    }

    // Normal state: base image + optional layer overlay
    if let Some(base_handle) = provider.bga_image() {
        commands.entity(parent).with_child((
            Sprite {
                image: base_handle,
                custom_size: Some(size),
                color: obj_color,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0001),
            MultiEntityChild,
        ));
    }

    if let Some(layer_handle) = provider.layer_image() {
        let obj_linear: LinearRgba = obj_color.into();
        let mesh = Rectangle::new(rect.w, rect.h);
        let mesh_handle = meshes.add(mesh);
        let material = bga_layer_materials.add(BgaLayerMaterial {
            color: obj_linear,
            texture: layer_handle,
        });
        commands.entity(parent).with_child((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0002),
            MultiEntityChild,
        ));
    }
}

/// Generates pixel data for procedural texture skin objects.
///
/// `bpm_events_override` and `note_distribution_override` allow the select screen
/// to supply graph data from SongInformation, bypassing the SkinStateProvider defaults.
pub fn generate_procedural_pixels(
    object: &SkinObjectType,
    provider: &dyn SkinStateProvider,
    width: u32,
    height: u32,
    bpm_events_override: &[(i64, f64)],
    note_distribution_override: &[u32],
) -> Option<Vec<u8>> {
    match object {
        SkinObjectType::BpmGraph(_) => {
            let events = if !bpm_events_override.is_empty() {
                bpm_events_override
            } else {
                provider.bpm_events()
            };
            Some(draw::visualizer::compute_bpm_graph_pixels(
                events, width, height,
            ))
        }
        SkinObjectType::HitErrorVisualizer(_) => {
            let timings = provider.recent_judge_timings();
            Some(draw::visualizer::compute_hit_error_pixels(
                timings, width, height,
            ))
        }
        SkinObjectType::NoteDistributionGraph(_) => {
            let counts = if !note_distribution_override.is_empty() {
                note_distribution_override
            } else {
                provider.note_distribution()
            };
            Some(draw::visualizer::compute_note_distribution_pixels(
                counts, width, height,
            ))
        }
        SkinObjectType::TimingDistributionGraph(_) => {
            let counts = provider.timing_distribution();
            Some(draw::visualizer::compute_timing_distribution_pixels(
                counts, width, height,
            ))
        }
        SkinObjectType::TimingVisualizer(_) => {
            let data = provider.timing_visualizer_data();
            Some(draw::visualizer::compute_timing_visualizer_pixels(
                data, width, height,
            ))
        }
        SkinObjectType::GaugeGraph(gg) => {
            let history = provider.gauge_history();
            let gauge_type = provider.gauge_type();
            Some(draw::visualizer::compute_gauge_graph_pixels(
                history,
                gauge_type,
                &gg.colors,
                gg.line_width,
                width,
                height,
            ))
        }
        _ => None,
    }
}
