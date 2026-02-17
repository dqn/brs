// Setup: spawn entities for each skin object.

use bevy::prelude::*;

use bms_skin::skin::Skin;
use bms_skin::skin_object_type::SkinObjectType;
use bms_skin::skin_text::FontType;

use super::components::{
    BitmapTextMarker, CachedBmFontText, CachedMultiEntityHash, MultiEntityMarker,
    ProceduralTextureMarker, ProceduralTextureState, SkinObjectEntity, SkinRenderState,
    TtfShadowMarker, TtfTextMarker,
};
use crate::font_map::FontMap;
use crate::state_provider::SkinStateProvider;
use crate::texture_map::TextureMap;

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
            // Multi-entity types: Number, Gauge, Judge, Float, DistributionGraph
            // These spawn child sprites dynamically each frame.
            SkinObjectType::Number(_)
            | SkinObjectType::Float(_)
            | SkinObjectType::Gauge(_)
            | SkinObjectType::Judge(_)
            | SkinObjectType::DistributionGraph(_)
            | SkinObjectType::Bar(_) => {
                commands.spawn((
                    Transform::default(),
                    Visibility::Hidden,
                    marker,
                    MultiEntityMarker,
                    CachedMultiEntityHash::default(),
                ));
            }
            // BGA: multi-entity for base + layer overlay rendering.
            SkinObjectType::Bga(_) => {
                commands.spawn((
                    Transform::default(),
                    Visibility::Hidden,
                    marker,
                    MultiEntityMarker,
                    CachedMultiEntityHash::default(),
                ));
            }
            // Procedural texture types: rendered from CPU pixel buffers.
            SkinObjectType::BpmGraph(_)
            | SkinObjectType::HitErrorVisualizer(_)
            | SkinObjectType::NoteDistributionGraph(_)
            | SkinObjectType::TimingDistributionGraph(_)
            | SkinObjectType::TimingVisualizer(_)
            | SkinObjectType::GaugeGraph(_) => {
                commands.spawn((
                    Sprite::default(),
                    Transform::default(),
                    Visibility::Hidden,
                    marker,
                    ProceduralTextureMarker,
                    ProceduralTextureState::default(),
                ));
            }
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
        bar_scroll_state: None,
        bpm_events: Vec::new(),
        note_distribution: Vec::new(),
    });
}
