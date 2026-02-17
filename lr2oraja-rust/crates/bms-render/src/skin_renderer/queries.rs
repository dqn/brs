// Query type aliases for the skin render system.

use bevy::prelude::*;

use super::components::{
    BitmapTextMarker, CachedBmFontText, CachedMultiEntityHash, MultiEntityMarker,
    ProceduralTextureMarker, ProceduralTextureState, SkinObjectEntity, TtfShadowMarker,
    TtfTextMarker,
};

pub type SpriteQuery<'w, 's> = Query<
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
        Without<MultiEntityMarker>,
        Without<ProceduralTextureMarker>,
    ),
>;

pub type TtfTextQuery<'w, 's> = Query<
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
        Without<MultiEntityMarker>,
        Without<ProceduralTextureMarker>,
    ),
>;

pub type BitmapTextQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut CachedBmFontText,
    ),
    (
        With<BitmapTextMarker>,
        Without<TtfTextMarker>,
        Without<TtfShadowMarker>,
        Without<MultiEntityMarker>,
        Without<ProceduralTextureMarker>,
    ),
>;

pub type TtfShadowQuery<'w, 's> = Query<
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
        Without<MultiEntityMarker>,
        Without<ProceduralTextureMarker>,
    ),
>;

pub type MultiEntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut CachedMultiEntityHash,
    ),
    (
        With<MultiEntityMarker>,
        Without<TtfTextMarker>,
        Without<BitmapTextMarker>,
        Without<TtfShadowMarker>,
        Without<ProceduralTextureMarker>,
    ),
>;

pub type ProceduralTextureQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SkinObjectEntity,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut Sprite,
        &'static mut ProceduralTextureState,
    ),
    (
        With<ProceduralTextureMarker>,
        Without<TtfTextMarker>,
        Without<BitmapTextMarker>,
        Without<TtfShadowMarker>,
        Without<MultiEntityMarker>,
    ),
>;
