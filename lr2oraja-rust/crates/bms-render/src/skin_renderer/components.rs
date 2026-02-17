// Marker components and resource types for skin renderer entities.

use bevy::prelude::*;

use bms_skin::skin::Skin;

use crate::draw::bar::BarScrollState;
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

/// Marker component for multi-entity skin objects (Number, Gauge, Judge, Float, DistributionGraph).
/// These objects spawn child sprite entities for rendering.
#[derive(Component)]
pub struct MultiEntityMarker;

/// Marker component for child sprites under a multi-entity skin object.
#[derive(Component)]
pub struct MultiEntityChild;

/// Caches a hash of the last rendered state to avoid unnecessary child re-spawning.
#[derive(Component, Default)]
pub struct CachedMultiEntityHash(pub u64);

/// Marker component for procedural texture skin objects (BpmGraph, HitErrorVisualizer, etc.).
/// These render CPU-generated pixel buffers as Bevy Image textures.
#[derive(Component)]
pub struct ProceduralTextureMarker;

/// Tracks the Bevy Image handle and content hash for a procedural texture.
#[derive(Component, Default)]
pub struct ProceduralTextureState {
    pub handle: Option<Handle<Image>>,
    pub hash: u64,
}

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
    /// Bar scroll state for music select screen bar rendering.
    pub bar_scroll_state: Option<BarScrollState>,
    /// BPM change events for procedural BPM graph rendering (from select screen).
    pub bpm_events: Vec<(i64, f64)>,
    /// Note distribution counts for procedural graph rendering (from select screen).
    pub note_distribution: Vec<u32>,
}
