// Phase 6+ stubs - will be replaced when later phases are translated

/// Stub for beatoraja.skin.Skin
pub struct Skin;

/// Stub for beatoraja.skin.SkinObject
pub struct SkinObject;

/// Stub for beatoraja.skin.SkinImage
pub struct SkinImage;

/// Stub for beatoraja.skin.SkinSource
pub struct SkinSource;

/// Stub for beatoraja.skin.SkinSourceImage
pub struct SkinSourceImage;

/// Stub for beatoraja.ir.IRScoreData
#[derive(Clone, Debug, Default)]
pub struct IRScoreData {
    pub player: String,
    pub option: i32,
}

impl IRScoreData {
    pub fn get_exscore(&self) -> i32 {
        0
    }
}

/// Stub for beatoraja.ir.RankingData
pub struct RankingData;

impl RankingData {
    pub const NONE: i32 = 0;
    pub const ACCESS: i32 = 1;
    pub const FINISH: i32 = 2;

    pub fn get_state(&self) -> i32 {
        Self::NONE
    }

    pub fn get_total_player(&self) -> i32 {
        0
    }

    pub fn get_score(&self, _index: i32) -> IRScoreData {
        IRScoreData::default()
    }

    pub fn load(&self, _state: &dyn std::any::Any, _songdata: &dyn std::any::Any) {
        // stub
    }
}

/// Stub for beatoraja.select.ScoreDataCache
pub struct ScoreDataCache;

impl ScoreDataCache {
    pub fn read_score_data(
        &self,
        _songdata: &dyn std::any::Any,
        _lnmode: i32,
    ) -> Option<beatoraja_core::score_data::ScoreData> {
        None
    }
}

/// Stub for MainController (Phase 7+)
pub struct MainController;

/// Stub for MainState (Phase 7+)
pub trait MainStateTrait: std::any::Any {}

/// Stub for ScoreDataProperty (Phase 7+)
pub struct ScoreDataProperty;

impl ScoreDataProperty {
    pub fn update_target_score(&self, _score: i32) {}
}

/// Stub for Texture/SpriteBatch (LibGDX)
pub struct Texture;
pub struct SpriteBatch;
pub struct TextureRegion;
pub struct Pixmap;
