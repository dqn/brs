// Main skin render system.
//
// Each frame, iterates over Skin.objects in order, resolves draw conditions,
// interpolates animations, applies offsets, and updates Bevy entities.

use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use bms_skin::skin::Skin;
use bms_skin::skin_object_type::SkinObjectType;
use bms_skin::skin_text::FontType;

use crate::coord::skin_to_bevy_transform;
use crate::distance_field_material::DistanceFieldMaterial;
use crate::draw;
use crate::draw::bmfont_text::layout_bmfont_text;
use crate::eval;
use crate::font_map::FontMap;
use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

// ---------------------------------------------------------------------------
// Marker components for skin object entities
// ---------------------------------------------------------------------------

/// Marker component for entities managed by the skin renderer.
#[derive(Component)]
pub struct SkinObjectEntity {
    /// Index into Skin.objects Vec.
    pub object_index: usize,
}

/// Marker component for TTF text entities (rendered via Bevy Text2d).
#[derive(Component)]
pub struct TtfTextMarker;

/// Marker component for BMFont text entities (rendered via glyph sprites).
#[derive(Component)]
pub struct BitmapTextMarker;

/// Marker component for child glyph sprites under a BMFont text entity.
#[derive(Component)]
pub struct BmFontGlyphChild;

/// Caches the last rendered text to avoid re-spawning glyph children every frame.
#[derive(Component, Default)]
pub struct CachedBmFontText(pub String);

/// Marker component for TTF shadow text entities.
#[derive(Component)]
pub struct TtfShadowMarker;

// ---------------------------------------------------------------------------
// Type aliases for complex query types
// ---------------------------------------------------------------------------

type SpriteQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut Sprite,
    ),
    (
        Without<TtfTextMarker>,
        Without<BitmapTextMarker>,
        Without<TtfShadowMarker>,
    ),
>;

type TtfTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut Text2d,
        &'static mut TextFont,
        &'static mut TextColor,
    ),
    (
        With<TtfTextMarker>,
        Without<BitmapTextMarker>,
        Without<TtfShadowMarker>,
    ),
>;

type BitmapTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut CachedBmFontText,
    ),
    (With<BitmapTextMarker>, Without<TtfTextMarker>),
>;

type TtfShadowQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut Text2d,
        &'static mut TextFont,
        &'static mut TextColor,
    ),
    (
        With<TtfShadowMarker>,
        Without<TtfTextMarker>,
        Without<BitmapTextMarker>,
    ),
>;

// ---------------------------------------------------------------------------
// Bevy Resource holding the skin render state
// ---------------------------------------------------------------------------

/// Bevy Resource holding all skin rendering state.
#[derive(Resource)]
pub struct SkinRenderState {
    pub skin: Skin,
    pub texture_map: TextureMap,
    pub font_map: FontMap,
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
    font_map: FontMap,
    state_provider: Box<dyn SkinStateProvider>,
) {
    let count = skin.objects.len();

    // Spawn one entity per skin object (initially invisible)
    for i in 0..count {
        let marker = SkinObjectEntity { object_index: i };

        match &skin.objects[i] {
            SkinObjectType::Text(text) => match &text.font_type {
                FontType::Bitmap { .. } => {
                    commands.spawn((
                        Transform::default(),
                        Visibility::Hidden,
                        marker,
                        BitmapTextMarker,
                        CachedBmFontText::default(),
                    ));
                }
                FontType::Ttf(_) | FontType::Default => {
                    // Spawn TTF shadow entity first (renders behind main text)
                    if text.shadow.is_some() {
                        commands.spawn((
                            Text2d::new(""),
                            TextFont::default(),
                            TextColor(Color::WHITE),
                            TextLayout::default(),
                            Transform::default(),
                            Visibility::Hidden,
                            SkinObjectEntity { object_index: i },
                            TtfShadowMarker,
                        ));
                    }

                    // TTF text: use Bevy Text2d for native font rendering.
                    // Text2d is spawned with a placeholder; updated each frame.
                    commands.spawn((
                        Text2d::new(""),
                        TextFont::default(),
                        TextColor(Color::WHITE),
                        TextLayout::default(),
                        Transform::default(),
                        Visibility::Hidden,
                        marker,
                        TtfTextMarker,
                    ));
                }
            },
            _ => {
                commands.spawn((
                    Sprite::default(),
                    Transform::default(),
                    Visibility::Hidden,
                    marker,
                ));
            }
        }
    }

    commands.insert_resource(SkinRenderState {
        skin,
        texture_map,
        font_map,
        state_provider,
    });
}

// ---------------------------------------------------------------------------
// Per-frame render system
// ---------------------------------------------------------------------------

/// Per-frame system that updates all skin object entities.
///
/// Uses three query sets:
/// - Sprite entities (images, sliders, graphs, etc.)
/// - TTF text entities (Text2d-based)
/// - BMFont text entities (glyph sprite children)
#[allow(clippy::too_many_arguments)]
pub fn skin_render_system(
    mut commands: Commands,
    render_state: Option<Res<SkinRenderState>>,
    mut sprite_query: SpriteQuery,
    mut ttf_query: TtfTextQuery,
    mut bitmap_query: BitmapTextQuery,
    mut shadow_query: TtfShadowQuery,
    mut meshes: ResMut<Assets<Mesh>>,
    mut df_materials: ResMut<Assets<DistanceFieldMaterial>>,
) {
    let Some(state) = render_state else {
        return;
    };

    let skin = &state.skin;
    let provider = &*state.state_provider;
    let tex_map = &state.texture_map;

    // --- Sprite entities ---
    for (marker, mut transform, mut visibility, mut sprite) in &mut sprite_query {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

        let Some((rect, color, final_angle, final_alpha)) = eval::resolve_common(base, provider)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        // Object-type-specific dispatch
        let time = eval::resolve_timer_time(base, provider).unwrap_or(0);
        let (tex_handle, src_rect_uv) = resolve_object_texture(object, provider, tex_map, time);

        // Update entity
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

    // --- TTF text entities ---
    for (marker, mut transform, mut visibility, mut text2d, mut text_font, mut text_color) in
        &mut ttf_query
    {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

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

    // --- TTF shadow entities ---
    for (marker, mut transform, mut visibility, mut text2d, mut text_font, mut text_color) in
        &mut shadow_query
    {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

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

    // --- BMFont text entities ---
    let font_map = &state.font_map;
    for (entity, marker, mut transform, mut visibility, mut cached) in &mut bitmap_query {
        let idx = marker.object_index;
        if idx >= skin.objects.len() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let object = &skin.objects[idx];
        let base = object.base();

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
                            &mut commands,
                            entity,
                            &glyph_cmds,
                            entry,
                            skin_text,
                            glyph_color,
                            rect.w,
                            rect.h,
                            &mut meshes,
                            &mut df_materials,
                        );
                    } else {
                        // Standard bitmap: use Sprite children with optional shadow
                        spawn_standard_glyph_children(
                            &mut commands,
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

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

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
        // Number, Gauge, and Visualizers need more complex multi-entity rendering.
        // Text is handled separately via TTF/BMFont queries.
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use crate::state_provider::StaticStateProvider;

    use super::*;

    #[test]
    fn skin_render_state_can_hold_static_provider() {
        let skin = Skin::new(bms_skin::skin_header::SkinHeader::default());
        let tex_map = TextureMap::new();
        let font_map = FontMap::new();
        let provider = Box::new(StaticStateProvider::default());

        let state = SkinRenderState {
            skin,
            texture_map: tex_map,
            font_map,
            state_provider: provider,
        };

        assert_eq!(state.skin.objects.len(), 0);
    }
}
