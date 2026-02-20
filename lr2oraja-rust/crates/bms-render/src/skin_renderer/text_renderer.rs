// Text rendering helpers: TTF text, TTF shadow, BMFont glyph spawning.

use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use bms_skin::skin::Skin;
use bms_skin::skin_object_type::SkinObjectType;
use bms_skin::skin_text::FontType;

use super::components::{BmFontGlyphChild, SkinRenderState};
use super::queries::{BitmapTextQuery, TtfShadowQuery, TtfTextQuery};
use crate::coord::skin_to_bevy_transform;
use crate::distance_field_material::DistanceFieldMaterial;
use crate::draw;
use crate::draw::bmfont_text::layout_bmfont_text;
use crate::eval;
use crate::font_map::FontMap;
use crate::state_provider::SkinStateProvider;

/// Renders TTF text entities each frame.
pub fn render_ttf_text(
    ttf_query: &mut TtfTextQuery,
    skin: &Skin,
    provider: &dyn SkinStateProvider,
    state: &SkinRenderState,
) {
    for (marker, mut transform, mut visibility, mut text2d, mut text_font, mut text_color) in
        ttf_query
    {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

        if !eval::check_option_conditions(base, skin, provider) {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some((rect, color, final_angle, final_alpha)) = eval::resolve_common(base, provider)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let SkinObjectType::Text(skin_text) = object {
            // Resolve text content
            let content = eval::resolve_text_content(skin_text, provider);

            // Update Text2d content
            **text2d = content;

            // Update font size
            text_font.font_size = skin_text.font_size;

            // If a TTF font is loaded, set the font handle
            if let FontType::Ttf(path) = &skin_text.font_type
                && let Some(entry) = state.font_map.get_ttf(path)
            {
                text_font.font = entry.handle.clone();
            }

            // Update color
            *text_color = TextColor(Color::srgba(color.r, color.g, color.b, final_alpha));

            // Update transform
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

            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Renders TTF shadow entities each frame.
pub fn render_ttf_shadow(
    shadow_query: &mut TtfShadowQuery,
    skin: &Skin,
    provider: &dyn SkinStateProvider,
    state: &SkinRenderState,
) {
    for (marker, mut transform, mut visibility, mut text2d, mut text_font, mut text_color) in
        shadow_query
    {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

        if !eval::check_option_conditions(base, skin, provider) {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some((rect, color, final_angle, final_alpha)) = eval::resolve_common(base, provider)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let SkinObjectType::Text(skin_text) = object
            && let Some(shadow) = &skin_text.shadow
        {
            let content = eval::resolve_text_content(skin_text, provider);
            **text2d = content;

            text_font.font_size = skin_text.font_size;
            if let FontType::Ttf(path) = &skin_text.font_type
                && let Some(entry) = state.font_map.get_ttf(path)
            {
                text_font.font = entry.handle.clone();
            }

            // Shadow color: RGB halved, same alpha (Java pattern)
            let (sr, sg, sb, sa) =
                eval::shadow_color_from_main(color.r, color.g, color.b, final_alpha);
            *text_color = TextColor(Color::srgba(sr, sg, sb, sa));

            // Shadow transform: same position + shadow offset, slightly behind main
            let shadow_z_order = idx as f32 * 0.001 - 0.0005;
            *transform = skin_to_bevy_transform(
                crate::coord::SkinRect {
                    x: rect.x + shadow.offset_x,
                    y: rect.y + shadow.offset_y,
                    w: rect.w,
                    h: rect.h,
                },
                crate::coord::ScreenSize {
                    w: skin.width,
                    h: skin.height,
                },
                shadow_z_order,
                crate::coord::RotationParams {
                    angle_deg: final_angle,
                    center_x: base.center_x,
                    center_y: base.center_y,
                },
            );
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Renders BMFont text entities each frame.
#[allow(clippy::too_many_arguments)]
pub fn render_bitmap_text(
    commands: &mut Commands,
    bitmap_query: &mut BitmapTextQuery,
    skin: &Skin,
    provider: &dyn SkinStateProvider,
    font_map: &FontMap,
    meshes: &mut Assets<Mesh>,
    df_materials: &mut Assets<DistanceFieldMaterial>,
) {
    for (entity, marker, mut transform, mut visibility, mut cached) in bitmap_query {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

        if !eval::check_option_conditions(base, skin, provider) {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some((rect, color, final_angle, final_alpha)) = eval::resolve_common(base, provider)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

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

        if let SkinObjectType::Text(skin_text) = object {
            let content = eval::resolve_text_content(skin_text, provider);

            // Only rebuild glyph children when text content changes
            if content != cached.0 {
                commands.entity(entity).despawn_descendants();

                if let FontType::Bitmap { path, bitmap_type } = &skin_text.font_type
                    && let Some(entry) = font_map.get_bitmap(path)
                {
                    let glyph_region = bms_skin::skin_object::Rect::new(0.0, 0.0, rect.w, rect.h);
                    let glyph_cmds = layout_bmfont_text(
                        &content,
                        &entry.data,
                        skin_text.font_size,
                        &glyph_region,
                        skin_text.align,
                        skin_text.overflow,
                    );
                    let glyph_color = Color::srgba(color.r, color.g, color.b, final_alpha);

                    let is_distance_field = *bitmap_type == 1 || *bitmap_type == 2;

                    if is_distance_field {
                        // Distance field glyphs: use Mesh2d + DistanceFieldMaterial
                        spawn_df_glyph_children(
                            commands,
                            entity,
                            &glyph_cmds,
                            entry,
                            skin_text,
                            glyph_color,
                            rect.w,
                            rect.h,
                            meshes,
                            df_materials,
                        );
                    } else {
                        // Standard bitmap: use Sprite children with optional shadow
                        spawn_standard_glyph_children(
                            commands,
                            entity,
                            &glyph_cmds,
                            entry,
                            skin_text,
                            glyph_color,
                            rect.w,
                            rect.h,
                        );
                    }
                }

                cached.0 = content;
            }
        }

        *visibility = Visibility::Visible;
    }
}

/// Spawns standard (bitmap_type=0) glyph sprite children with optional shadow.
#[allow(clippy::too_many_arguments)]
fn spawn_standard_glyph_children(
    commands: &mut Commands,
    parent: Entity,
    glyph_cmds: &[draw::bmfont_text::GlyphDrawCommand],
    entry: &crate::font_map::BmFontEntry,
    skin_text: &bms_skin::skin_text::SkinText,
    main_color: Color,
    region_w: f32,
    region_h: f32,
) {
    let has_shadow = skin_text
        .shadow
        .as_ref()
        .is_some_and(|s| s.offset_x != 0.0 || s.offset_y != 0.0);

    // Shadow glyphs first (rendered behind main glyphs)
    if has_shadow {
        let shadow = skin_text.shadow.as_ref().unwrap();
        let main_srgba: Srgba = main_color.into();
        let (sr, sg, sb, sa) = eval::shadow_color_from_main(
            main_srgba.red,
            main_srgba.green,
            main_srgba.blue,
            main_srgba.alpha,
        );
        let shadow_color = Color::srgba(sr, sg, sb, sa);

        for cmd in glyph_cmds {
            let page_idx = cmd.page as usize;
            let tex_handle = match entry.page_textures.get(page_idx) {
                Some(h) => h.clone(),
                None => continue,
            };

            let local_x = cmd.dst_x + cmd.dst_w / 2.0 - region_w / 2.0 + shadow.offset_x;
            let local_y = -(cmd.dst_y + cmd.dst_h / 2.0 - region_h / 2.0) - shadow.offset_y;

            commands.entity(parent).with_child((
                Sprite {
                    image: tex_handle,
                    custom_size: Some(Vec2::new(cmd.dst_w, cmd.dst_h)),
                    rect: Some(bevy::math::Rect::new(
                        cmd.src_x,
                        cmd.src_y,
                        cmd.src_x + cmd.src_w,
                        cmd.src_y + cmd.src_h,
                    )),
                    color: shadow_color,
                    ..default()
                },
                Transform::from_xyz(local_x, local_y, 0.0),
                BmFontGlyphChild,
            ));
        }
    }

    // Main glyphs
    for cmd in glyph_cmds {
        let page_idx = cmd.page as usize;
        let tex_handle = match entry.page_textures.get(page_idx) {
            Some(h) => h.clone(),
            None => continue,
        };

        let local_x = cmd.dst_x + cmd.dst_w / 2.0 - region_w / 2.0;
        let local_y = -(cmd.dst_y + cmd.dst_h / 2.0 - region_h / 2.0);

        commands.entity(parent).with_child((
            Sprite {
                image: tex_handle,
                custom_size: Some(Vec2::new(cmd.dst_w, cmd.dst_h)),
                rect: Some(bevy::math::Rect::new(
                    cmd.src_x,
                    cmd.src_y,
                    cmd.src_x + cmd.src_w,
                    cmd.src_y + cmd.src_h,
                )),
                color: main_color,
                ..default()
            },
            Transform::from_xyz(local_x, local_y, 0.0001),
            BmFontGlyphChild,
        ));
    }
}

/// Spawns distance field (bitmap_type=1,2) glyph children using Mesh2d + DistanceFieldMaterial.
/// Shadow and outline are handled entirely in the shader (no double-draw needed).
#[allow(clippy::too_many_arguments)]
fn spawn_df_glyph_children(
    commands: &mut Commands,
    parent: Entity,
    glyph_cmds: &[draw::bmfont_text::GlyphDrawCommand],
    entry: &crate::font_map::BmFontEntry,
    skin_text: &bms_skin::skin_text::SkinText,
    main_color: Color,
    region_w: f32,
    region_h: f32,
    meshes: &mut Assets<Mesh>,
    df_materials: &mut Assets<DistanceFieldMaterial>,
) {
    let main_linear: LinearRgba = main_color.into();

    // Outline parameters
    let outline_distance = if skin_text.outline_color.is_some() && skin_text.outline_width > 0.0 {
        crate::distance_field_material::compute_outline_distance(skin_text.outline_width)
    } else {
        0.5 // No outline
    };
    let outline_linear: LinearRgba = skin_text
        .outline_color
        .as_ref()
        .map(|c| Color::srgba(c.r, c.g, c.b, c.a).into())
        .unwrap_or(LinearRgba::NONE);

    // Shadow parameters
    let (shadow_color, shadow_offset, shadow_smoothing) = if let Some(shadow) = &skin_text.shadow {
        let sc: LinearRgba = Color::srgba(
            shadow.color.r,
            shadow.color.g,
            shadow.color.b,
            shadow.color.a,
        )
        .into();
        // Compute UV-space offset using the first page dimensions
        let (pw, ph) = entry.page_dimensions.first().copied().unwrap_or((1.0, 1.0));
        let offset = crate::distance_field_material::compute_shadow_offset(
            shadow.offset_x,
            shadow.offset_y,
            pw,
            ph,
        );
        let smoothing = crate::distance_field_material::compute_shadow_smoothing(shadow.smoothness);
        (sc, offset, smoothing)
    } else {
        (LinearRgba::NONE, Vec4::ZERO, 0.0)
    };

    for cmd in glyph_cmds {
        let page_idx = cmd.page as usize;
        let tex_handle = match entry.page_textures.get(page_idx) {
            Some(h) => h.clone(),
            None => continue,
        };

        let mesh = Rectangle::new(cmd.dst_w, cmd.dst_h);
        let mesh_handle = meshes.add(mesh);

        let material = df_materials.add(DistanceFieldMaterial {
            color: main_linear,
            outline_color: outline_linear,
            shadow_color,
            params: Vec4::new(outline_distance, shadow_smoothing, 0.0, 0.0),
            shadow_offset,
            texture: tex_handle,
        });

        let local_x = cmd.dst_x + cmd.dst_w / 2.0 - region_w / 2.0;
        let local_y = -(cmd.dst_y + cmd.dst_h / 2.0 - region_h / 2.0);

        commands.entity(parent).with_child((
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::from_xyz(local_x, local_y, 0.0001),
            BmFontGlyphChild,
        ));
    }
}
